// Copyright (C) 2025 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Ensures all FFI function references are consistent.

pub const DPI_INIT_CALLBACK: &str = "dpi_init_callback";

pub const TRACE_EVER_ON: &str = "ffi_Verilated_traceEverOn";

pub fn open_trace(top_module: &str) -> String {
    format!("ffi_V{top_module}_open_trace")
}

pub const TRACE_DUMP: &str = "ffi_trace_dump";
pub const TRACE_OPEN_NEXT: &str = "ffi_trace_open_next";
pub const TRACE_FLUSH: &str = "ffi_trace_flush";
pub const TRACE_CLOSE_AND_DELETE: &str = "ffi_trace_close_and_delete";

pub fn new_top(top_module: &str) -> String {
    format!(" ffi_new_V{top_module}")
}

pub fn top_eval(top_module: &str) -> String {
    format!("ffi_V{top_module}_eval")
}

pub fn delete_top(top_module: &str) -> String {
    format!("ffi_delete_V{top_module}")
}

pub fn pin_port(top_module: &str, port: &str) -> String {
    format!("ffi_V{top_module}_pin_{port}")
}

pub fn read_port(top_module: &str, port: &str) -> String {
    format!("ffi_V{top_module}_read_{port}")
}
