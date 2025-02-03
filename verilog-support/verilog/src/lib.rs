// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

pub mod __reexports {
    pub use libc;
    pub use libloading;
    pub use verilator;
}

pub use verilator::{
    dynamic::DynamicVerilatedModel, dynamic::DynamicVerilatedModelError,
    dynamic::VerilatorValue, PortDirection, VerilatorRuntime,
};
pub use verilog_macro::verilog;
