// Copyright (C) 2025 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

pub const DPI_INIT_CALLBACK: &str = "dpi_init_callback";

pub const TRACE_EVER_ON: &str = "ffi_Verilated_traceEverOn";

pub fn open_trace(top_module: &str) -> String {
    format!("ffi_V{top_module}_open_trace")
}

pub const VCD_DUMP: &str = "ffi_VerilatedVcdC_dump";
