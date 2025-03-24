// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::ffi;

pub trait DpiFunction: Sync {
    /// The Rust-declared name of the DPI function. This should be taken to be
    /// equivalent to the name given for the DPI C function in Verilog
    /// source code.
    fn name(&self) -> &'static str;

    /// A list of `(name, c_type)` pairs serving as the parameters of the
    /// generated C function and the generated function pointer type for the
    /// Rust function.
    fn signature(&self) -> &'static [(&'static str, &'static str)];

    /// The Rust function as a function pointer.
    fn pointer(&self) -> *const ffi::c_void;
}
