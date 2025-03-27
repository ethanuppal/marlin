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
use marlin::verilator::{
    AsDynamicVerilatedModel, VerilatorRuntime, VerilatorRuntimeOptions,
};
use snafu::{ResultExt, Whatever};

#[test]
//#[snafu::report]
fn main() -> Result<(), Whatever> {
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

    let runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model_simple::<Main>()?;

    main.pin("medium_input", u32::MAX).whatever_context("pin")?;
    println!("{}", main.read("medium_output").whatever_context("read")?);
    assert_eq!(
        main.read("medium_output").whatever_context("read")?,
        0u32.into()
    );
    main.eval();
    println!("{}", main.read("medium_output").whatever_context("read")?);
    assert_eq!(
        main.read("medium_output").whatever_context("read")?,
        u32::MAX.into()
    );

    Ok(())
}
