// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

use snafu::Snafu;

use crate::{
    core::{PortDirection, compute_approx_width_from_edata_word_count},
    types,
};

/// See [`types`].
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum VerilatorValue<'a> {
    CData(types::CData),
    SData(types::SData),
    IData(types::IData),
    QData(types::QData),
    WDataInP(&'a [types::EData]),
    WDataOutP(Box<[types::EData]>),
}

impl VerilatorValue<'_> {
    /// The maximum number of bits this value takes up.
    pub fn width(&self) -> usize {
        match self {
            Self::CData(_) => 8,
            Self::SData(_) => 16,
            Self::IData(_) => 32,
            Self::QData(_) => 64,
            Self::WDataInP(values) => {
                compute_approx_width_from_edata_word_count(values.len())
            }
            Self::WDataOutP(values) => {
                compute_approx_width_from_edata_word_count(values.len())
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

impl<'a, const WORDS: usize> From<&'a [types::EData; WORDS]>
    for VerilatorValue<'a>
{
    fn from(value: &'a [types::EData; WORDS]) -> Self {
        Self::WDataInP(value)
    }
}

impl<const WORDS: usize> From<[types::EData; WORDS]> for VerilatorValue<'_> {
    fn from(value: [types::EData; WORDS]) -> Self {
        Self::WDataOutP(value.into())
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

/// Access model ports at runtime.
pub trait AsDynamicVerilatedModel<'ctx>: 'ctx {
    /// Equivalent to the Verilator `eval` method.
    fn eval(&mut self);

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
