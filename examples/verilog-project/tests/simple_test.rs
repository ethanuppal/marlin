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
            if stringify!($name) == "colog_test" {
                if env::var("COLOG").is_ok() {
                    colog::init();
                }
            }

            let mut runtime = VerilatorRuntime::new(
                "artifacts".into(),
                &["src/main.sv".as_ref()],
                &[],
                [],
                VerilatorRuntimeOptions::default_logging(),
            )?;

            let mut main = runtime.create_model::<Main>()?;

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

test!(colog_test);
test!(first_test);
test!(second_test);
test!(third_test);
test!(fourth_test);
