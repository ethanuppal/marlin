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

use example_verilog_project::Main;
use marlin::verilator::{VerilatorRuntime, VerilatorRuntimeOptions};
use snafu::Whatever;

macro_rules! test {
    ($name:ident) => {
        #[test]
        #[snafu::report]
        fn $name() -> Result<(), Whatever> {
            if stringify!($name) == "verbose_test" {
                if env::var("RUST_LOG").is_ok() {
                    env_logger::init();
                }
            }

            let runtime = VerilatorRuntime::new(
                "artifacts".into(),
                &["src/main.sv".as_ref()],
                &[],
                [],
                VerilatorRuntimeOptions::default_logging(),
            )?;

            let mut main = runtime.create_model_simple::<Main>()?;

            main.medium_input = u32::MAX;
            println!("{}", main.medium_output);
            assert_eq!(main.medium_output, 0);
            main.eval();
            println!("{}", main.medium_output);
            assert_eq!(main.medium_output, u32::MAX);

            Ok(())
        }
    };
}

test!(verbose_test);
test!(first_test);
test!(second_test);
test!(third_test);
test!(fourth_test);

#[test]
#[snafu::report]
fn test_preprocessor_defines() -> Result<(), Whatever> {
    use example_verilog_project::DefinesMain;

    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/defines.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default(),
    )?;

    let mut dut = runtime.create_model_simple::<DefinesMain>()?;
    dut.data_in = 0xDEAD_BEEF;
    dut.eval();
    assert_eq!(dut.data_out, 0xDEAD_BEEF);

    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/defines.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions {
            defines: vec!["INVERT_OUTPUT".into()],
            ..Default::default()
        },
    )?;

    let mut dut = runtime.create_model_simple::<DefinesMain>()?;
    dut.data_in = 0xDEAD_BEEF;
    dut.eval();
    assert_eq!(dut.data_out, !0xDEAD_BEEF);

    Ok(())
}
