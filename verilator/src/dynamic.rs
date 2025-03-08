// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, fmt};

use libloading::Library;
use snafu::Snafu;

use crate::{PortDirection, types};

pub struct DynamicVerilatedModel<'ctx> {
    pub(crate) ports: HashMap<String, (usize, PortDirection)>,
    pub(crate) name: String,
    pub(crate) main: *mut libc::c_void,
    pub(crate) eval_main: extern "C" fn(*mut libc::c_void),
    pub(crate) delete_main: extern "C" fn(*mut libc::c_void),
    pub(crate) library: &'ctx Library,
    //cache: HashMap<String, Symbol<'ctx>>,
}

impl Drop for DynamicVerilatedModel<'_> {
    fn drop(&mut self) {
        (self.delete_main)(self.main);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum VerilatorValue {
    CData(types::CData),
    SData(types::SData),
    IData(types::IData),
    QData(types::QData),
}

impl fmt::Display for VerilatorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerilatorValue::CData(cdata) => cdata.fmt(f),
            VerilatorValue::SData(sdata) => sdata.fmt(f),
            VerilatorValue::IData(idata) => idata.fmt(f),
            VerilatorValue::QData(qdata) => qdata.fmt(f),
        }
    }
}

impl From<types::CData> for VerilatorValue {
    fn from(value: types::CData) -> Self {
        Self::CData(value)
    }
}

impl From<types::SData> for VerilatorValue {
    fn from(value: types::SData) -> Self {
        Self::SData(value)
    }
}
impl From<types::IData> for VerilatorValue {
    fn from(value: types::IData) -> Self {
        Self::IData(value)
    }
}

impl From<types::QData> for VerilatorValue {
    fn from(value: types::QData) -> Self {
        Self::QData(value)
    }
}

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

impl DynamicVerilatedModel<'_> {
    pub fn eval(&mut self) {
        (self.eval_main)(self.main);
    }

    pub fn read(
        &self,
        port: impl Into<String>,
    ) -> Result<VerilatorValue, DynamicVerilatedModelError> {
        let port: String = port.into();
        let (width, direction) = *self.ports.get(&port).ok_or(
            DynamicVerilatedModelError::NoSuchPort {
                top_module: self.name.clone(),
                port: port.clone(),
                source: None,
            },
        )?;

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
                    extern "C" fn(*mut libc::c_void) -> $value_type,
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
            unreachable!("Should have been caught in create_dyn_model")
        }
    }

    pub fn pin(
        &mut self,
        port: impl Into<String>,
        value: impl Into<VerilatorValue>,
    ) -> Result<(), DynamicVerilatedModelError> {
        macro_rules! pin_value {
            ($self:ident, $port:expr, $value:expr, $value_type:ty, $low:literal, $high:literal) => {{
                let symbol: libloading::Symbol<
                    extern "C" fn(*mut libc::c_void, $value_type),
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

                let (width, direction) = $self
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
        }
    }
}
