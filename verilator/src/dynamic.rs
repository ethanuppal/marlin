// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Support for dynamic models.

use std::{collections::HashMap, ffi, fmt, slice};

use libloading::Library;
use snafu::Snafu;

use crate::{PortDirection, WideOut, types};

/// See [`types`].
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum VerilatorValue<'a> {
    CData(types::CData),
    SData(types::SData),
    IData(types::IData),
    QData(types::QData),
    WDataInP(&'a [types::WData]),
    WDataOutP(Vec<types::WData>),
}

impl VerilatorValue<'_> {
    /// The maximum number of bits this value takes up.
    pub fn width(&self) -> usize {
        match self {
            Self::CData(_) => 8,
            Self::SData(_) => 16,
            Self::IData(_) => 32,
            Self::QData(_) => 64,
            Self::WDataInP(values) => std::mem::size_of_val(*values) * 8,
            Self::WDataOutP(values) => {
                values.len() * size_of::<types::WData>() * 8
            }
        }
    }
}

impl fmt::Display for VerilatorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerilatorValue::CData(cdata) => cdata.fmt(f),
            VerilatorValue::SData(sdata) => sdata.fmt(f),
            VerilatorValue::IData(idata) => idata.fmt(f),
            VerilatorValue::QData(qdata) => qdata.fmt(f),
            Self::WDataInP(_values) => "wide (fmt is todo)".fmt(f),
            Self::WDataOutP(_values) => "wide (fmt is todo)".fmt(f),
        }
    }
}

impl From<types::CData> for VerilatorValue<'_> {
    fn from(value: types::CData) -> Self {
        Self::CData(value)
    }
}

impl From<types::SData> for VerilatorValue<'_> {
    fn from(value: types::SData) -> Self {
        Self::SData(value)
    }
}
impl From<types::IData> for VerilatorValue<'_> {
    fn from(value: types::IData) -> Self {
        Self::IData(value)
    }
}

impl From<types::QData> for VerilatorValue<'_> {
    fn from(value: types::QData) -> Self {
        Self::QData(value)
    }
}

impl<const LOW: usize, const HIGH: usize, const LENGTH: usize>
    From<WideOut<LOW, HIGH, LENGTH>> for VerilatorValue<'_>
{
    fn from(value: WideOut<LOW, HIGH, LENGTH>) -> Self {
        Self::WDataOutP(value.inner.to_vec())
    }
}

/// Access model ports at runtime.
pub trait AsDynamicVerilatedModel<'ctx>: 'ctx {
    /// If `port` is a valid port name for this model, returns the current value
    /// of the port.
    fn read(
        &self,
        port: impl Into<String>,
    ) -> Result<VerilatorValue<'_>, DynamicVerilatedModelError>;

    /// If `port` is a valid port name for this model, and the port's width is
    /// `<=` `value.into().width()`, sets the port to `value`.
    fn pin(
        &mut self,
        port: impl Into<String>,
        value: impl Into<VerilatorValue<'ctx>>,
    ) -> Result<(), DynamicVerilatedModelError>;
}

#[derive(Clone, Copy)]
pub(crate) struct DynamicPortInfo {
    pub(crate) width: usize,
    pub(crate) direction: PortDirection,
}

/// A hardware model constructed at runtime. See
/// [`super::VerilatorRuntime::create_dyn_model`].
pub struct DynamicVerilatedModel<'ctx> {
    // TODO: add the dlsyms here and remove the library field
    pub(crate) ports: HashMap<String, DynamicPortInfo>,
    pub(crate) name: String,
    pub(crate) main: *mut ffi::c_void,
    pub(crate) eval_main: extern "C" fn(*mut ffi::c_void),
    pub(crate) library: &'ctx Library,
}

impl DynamicVerilatedModel<'_> {
    /// Equivalent to the Verilator `eval` method.
    pub fn eval(&mut self) {
        (self.eval_main)(self.main);
    }
}

/// Runtime port read/write error.
#[derive(Debug, Snafu)]
pub enum DynamicVerilatedModelError {
    #[snafu(display(
        "Port {port} not found on verilated module {top_module}: did you forget to specify it in the runtime `create_dyn_model` constructor?: {source:?}"
    ))]
    NoSuchPort {
        top_module: String,
        port: String,
        #[snafu(source(false))]
        source: Option<libloading::Error>,
    },
    #[snafu(display(
        "Port {port} on verilated module {top_module} has width {width}, but used as if it was in the {attempted_lower} to {attempted_higher} width range"
    ))]
    InvalidPortWidth {
        top_module: String,
        port: String,
        width: usize,
        attempted_lower: usize,
        attempted_higher: usize,
    },
    #[snafu(display(
        "Port {port} on verilated module {top_module} is an {direction} port, but was used as an {attempted_direction} port"
    ))]
    InvalidPortDirection {
        top_module: String,
        port: String,
        direction: PortDirection,
        attempted_direction: PortDirection,
    },
}

impl<'ctx> AsDynamicVerilatedModel<'ctx> for DynamicVerilatedModel<'ctx> {
    fn read(
        &self,
        port: impl Into<String>,
    ) -> Result<VerilatorValue<'_>, DynamicVerilatedModelError> {
        let port: String = port.into();
        let DynamicPortInfo { width, direction } = *self
            .ports
            .get(&port)
            .ok_or(DynamicVerilatedModelError::NoSuchPort {
                top_module: self.name.clone(),
                port: port.clone(),
                source: None,
            })?;

        if !matches!(direction, PortDirection::Output | PortDirection::Inout,) {
            return Err(DynamicVerilatedModelError::InvalidPortDirection {
                top_module: self.name.clone(),
                port,
                direction,
                attempted_direction: PortDirection::Output,
            });
        }

        macro_rules! read_value {
            ($self:ident, $port:expr, $value_type:ty) => {{
                let symbol: libloading::Symbol<
                    extern "C" fn(*mut ffi::c_void) -> $value_type,
                > = unsafe {
                    self.library.get(
                        format!("ffi_V{}_read_{}", self.name, $port).as_bytes(),
                    )
                }
                .map_err(|source| {
                    DynamicVerilatedModelError::NoSuchPort {
                        top_module: $self.name.to_string(),
                        port: $port.clone(),
                        source: Some(source),
                    }
                })?;

                Ok((*symbol)($self.main).into())
            }};
        }

        if width <= 8 {
            read_value!(self, port, types::CData)
        } else if width <= 16 {
            read_value!(self, port, types::SData)
        } else if width <= 32 {
            read_value!(self, port, types::IData)
        } else if width <= 64 {
            read_value!(self, port, types::QData)
        } else {
            let value: types::WDataOutP =
                read_value!(self, port, types::WDataOutP)?;
            let length = width.div_ceil(types::WData::BITS as usize);
            let mut result = Vec::with_capacity(length);
            result.extend_from_slice(unsafe {
                slice::from_raw_parts(value, length)
            });
            Ok(VerilatorValue::WDataOutP(result))
        }
    }

    fn pin(
        &mut self,
        port: impl Into<String>,
        value: impl Into<VerilatorValue<'ctx>>,
    ) -> Result<(), DynamicVerilatedModelError> {
        macro_rules! pin_value {
            ($self:ident, $port:expr, $value:expr, $value_type:ty, $low:literal, $high:expr) => {{
                let symbol: libloading::Symbol<
                    extern "C" fn(*mut ffi::c_void, $value_type),
                > = unsafe {
                    self.library.get(
                        format!("ffi_V{}_pin_{}", self.name, $port).as_bytes(),
                    )
                }
                .map_err(|source| {
                    DynamicVerilatedModelError::NoSuchPort {
                        top_module: $self.name.to_string(),
                        port: $port.clone(),
                        source: Some(source),
                    }
                })?;

                let DynamicPortInfo { width, direction } = $self
                    .ports
                    .get(&$port)
                    .ok_or(DynamicVerilatedModelError::NoSuchPort {
                        top_module: $self.name.clone(),
                        port: $port.clone(),
                        source: None,
                    })?
                    .clone();

                if width > $high {
                    return Err(DynamicVerilatedModelError::InvalidPortWidth {
                        top_module: $self.name.clone(),
                        port: $port.clone(),
                        width,
                        attempted_lower: $low,
                        attempted_higher: $high,
                    });
                }

                if !matches!(
                    direction,
                    PortDirection::Input | PortDirection::Inout,
                ) {
                    return Err(
                        DynamicVerilatedModelError::InvalidPortDirection {
                            top_module: $self.name.clone(),
                            port: $port,
                            direction,
                            attempted_direction: PortDirection::Input,
                        },
                    );
                }

                (*symbol)($self.main, $value);
                Ok(())
            }};
        }

        let port: String = port.into();
        match value.into() {
            VerilatorValue::CData(cdata) => {
                pin_value!(self, port, cdata, types::CData, 0, 8)
            }
            VerilatorValue::SData(sdata) => {
                pin_value!(self, port, sdata, types::SData, 9, 16)
            }
            VerilatorValue::IData(idata) => {
                pin_value!(self, port, idata, types::IData, 17, 32)
            }
            VerilatorValue::QData(qdata) => {
                pin_value!(self, port, qdata, types::QData, 33, 64)
            }
            VerilatorValue::WDataInP(values) => {
                let values_ptr = values.as_ptr();
                pin_value!(
                    self,
                    port,
                    values_ptr,
                    types::WDataInP,
                    65,
                    usize::MAX
                )
            }
            VerilatorValue::WDataOutP(_) => {
                unreachable!("output ports should have already been caught")
            }
        }
    }
}
