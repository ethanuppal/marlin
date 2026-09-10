// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::ffi;

use libloading::Library;

use crate::core::PortDirection;

/// You should not implement this `trait` manually. Instead, use a procedural
/// macro like `#[verilog(...)]` to derive it for you.
pub trait AsVerilatedModel<'ctx>: 'ctx {
    /// The source-level name of the module.
    fn name() -> &'static str;

    /// The path of the module's definition.
    fn source_path() -> &'static str;

    /// The module's interface; each element is `(port_name, port_msb, port_lsb,
    /// port_direction)`.
    fn ports() -> &'static [(&'static str, usize, usize, PortDirection)];

    #[doc(hidden)]
    fn init_from(library: &'ctx Library, tracing_enabled: bool) -> Self;

    #[doc(hidden)]
    unsafe fn model(&self) -> *mut ffi::c_void;
}
