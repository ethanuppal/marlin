// Copyright (C) 2026 Ethan Uppal.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::Path;

use example_verilog_project::Mangled;
use marlin::{
    verilator::{VerilatorRuntime, VerilatorRuntimeOptions, verilator_version},
    verilog::prelude::*,
};
use snafu::Whatever;

#[test]
#[snafu::report]
fn mangled_works() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new2(
        "artifacts2",
        &["src/mangled.sv"],
        &[] as &[&Path],
        [],
        VerilatorRuntimeOptions::default()
            .allow_unsupported_verilator(Some(verilator_version!(5 020))),
    )?;

    let mut main = runtime.create_model_simple::<Mangled>()?;

    main.medium_input = 3;
    main.eval();
    assert_eq!(main.medium_output, 3);

    Ok(())
}
