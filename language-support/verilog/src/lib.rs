// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

pub mod __reexports {
    pub use libc;
    pub use libloading;
    pub use marlin_verilator as verilator;
}

pub use marlin_verilog_macro::dpi;

pub mod prelude {
    pub use crate as verilog;
    pub use marlin_verilog_macro::verilog;
}
