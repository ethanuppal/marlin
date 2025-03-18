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

use std::env;

use example_verilog_project::{DpiMain, MoreDpiMain};
use marlin::{
    verilator::{VerilatorRuntime, VerilatorRuntimeOptions},
    verilog::prelude::*,
};
use snafu::Whatever;

const SET_OUT_TO: i32 = 3;

#[verilog::dpi]
pub extern "C" fn set_out(output: &mut i32) {
    *output = SET_OUT_TO;
}

#[test]
#[snafu::report]
fn main_tutorial() -> Result<(), Whatever> {
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
        &[],
        [set_out],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model::<DpiMain>()?;
    main.eval();

    assert_eq!(main.out, SET_OUT_TO as u32);

    Ok(())
}

const SET_UNSIGNED_INT_OUT_TO: u32 = 5;
const SET_BOOL_OUT_TO: bool = true;

#[verilog::dpi]
pub extern "C" fn set_unsigned_int_out(output: &mut u32) {
    *output = SET_UNSIGNED_INT_OUT_TO;
}

#[verilog::dpi]
pub extern "C" fn check_unsigned_int_out(input: u32) {
    assert_eq!(input, SET_UNSIGNED_INT_OUT_TO);
}

#[verilog::dpi]
pub extern "C" fn set_bool_out(output: &mut bool) {
    *output = SET_BOOL_OUT_TO;
}

#[test]
//#[snafu::report]
fn other_test() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/more_dpi.sv".as_ref()],
        &[],
        [set_unsigned_int_out, check_unsigned_int_out, set_bool_out],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model::<MoreDpiMain>()?;
    main.eval();

    assert_eq!(main.int_out, SET_UNSIGNED_INT_OUT_TO);
    assert_eq!(main.bool_out, SET_BOOL_OUT_TO as u8);

    Ok(())
}
