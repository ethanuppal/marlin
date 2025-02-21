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

use example_verilog_project::DpiMain;
use marlin::{
    verilator::{VerilatorRuntime, VerilatorRuntimeOptions},
    verilog::prelude::*,
};
use snafu::Whatever;

#[verilog::dpi]
pub extern "C" fn three(out: &mut u32) {
    *out = 3;
}

#[test]
#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
        &[],
        [three],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

    let mut main = runtime.create_model::<DpiMain>()?;
    main.eval();
    assert_eq!(main.out, 3);

    Ok(())
}
