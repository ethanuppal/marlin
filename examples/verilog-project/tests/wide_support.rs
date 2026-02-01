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

use example_verilog_project::WideMain;
use marlin::verilator::{
    AsDynamicVerilatedModel, PortDirection, VerilatedModelConfig,
    VerilatorRuntime, VerilatorRuntimeOptions, WideIn, dynamic::VerilatorValue,
};
use snafu::{ResultExt, Whatever};

#[test]
//#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["src/wide_main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model_simple::<WideMain>()?;

    main.wide_input = WideIn::new([u32::MAX, u32::MAX, 1]);
    println!("{:?}", main.wide_output);
    assert_eq!(main.wide_output.value(), &[0; 3]);
    main.eval();
    println!("{:?}", main.wide_output);
    assert_eq!(main.wide_output.value(), &[u32::MAX, u32::MAX, 1]);

    Ok(())
}

#[test]
//#[snafu::report]
fn forwards_correctly_dynamically() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["src/wide_main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_dyn_model(
        "wide_main",
        "src/wide_main.sv",
        &[
            ("wide_input", 64, 0, PortDirection::Input),
            ("wide_output", 64, 0, PortDirection::Output),
        ],
        VerilatedModelConfig::default(),
    )?;

    main.pin("wide_input", &[u32::MAX, u32::MAX, 1])
        .whatever_context("pin")?;
    assert_eq!(
        main.read("wide_output").whatever_context("first read")?,
        VerilatorValue::WDataOutP(vec![0, 0, 0])
    );
    main.eval();
    assert_eq!(
        main.read("wide_output").whatever_context("second read")?,
        VerilatorValue::WDataOutP(vec![u32::MAX, u32::MAX, 1])
    );

    Ok(())
}
