// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use snafu::Whatever;
use verilog::{verilog, VerilatorRuntime, VerilatorRuntimeOptions};

#[verilog::dpi]
#[no_mangle]
extern "C" fn three(#[output] out: &mut u32) {
    *out = 3;
}

#[verilog(src = "sv/dpi.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["sv/dpi.sv".as_ref()],
        [three],
        VerilatorRuntimeOptions {
            force_verilator_rebuild: true,
            ..Default::default()
        },
        true,
    )?;

    let mut main = runtime.create_model::<Main>()?;
    main.eval();

    Ok(())
}
