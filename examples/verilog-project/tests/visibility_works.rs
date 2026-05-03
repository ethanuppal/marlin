// Copyright (C) 2024 Ethan Uppal.
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

use example_verilog_project::enclosed;
use marlin::{
    verilator::{verilator_version, VerilatorRuntime, VerilatorRuntimeOptions},
    verilog::prelude::*,
};
use snafu::Whatever;

#[test]
#[snafu::report]
fn main() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts3",
        &["src/main.sv"],
        &[] as &[&Path],
        [],
        VerilatorRuntimeOptions::default()
            .allow_unsupported_verilator(Some(verilator_version!(5 020))),
    )?;

    let mut main = runtime.create_model_simple::<enclosed::Main2>()?;

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    Ok(())
}
