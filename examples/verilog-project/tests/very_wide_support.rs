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

use example_verilog_project::{VeryWideMain, VeryWideRegistered};
use marlin::verilator::{
    AsDynamicVerilatedModel, PortDirection, VerilatedModelConfig,
    VerilatorRuntime, VerilatorRuntimeOptions, WideIn, dynamic::VerilatorValue,
};
use snafu::{ResultExt, Whatever};

#[test]
fn forwards_correctly_very_wide() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts_very_wide".into(),
        &["src/very_wide_main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model_simple::<VeryWideMain>()?;

    let test_input = [0xDEADBEEF_u32, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222, 0x55];
    main.very_wide_input = WideIn::new(test_input);
    assert_eq!(main.very_wide_output.value(), &[0; 7]);
    main.eval();
    assert_eq!(main.very_wide_output.value(), &test_input);

    Ok(())
}

#[test]
fn forwards_correctly_very_wide_dynamically() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts_very_wide_dyn".into(),
        &["src/very_wide_main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_dyn_model(
        "very_wide_main",
        "src/very_wide_main.sv",
        &[
            ("very_wide_input", 199, 0, PortDirection::Input),
            ("very_wide_output", 199, 0, PortDirection::Output),
        ],
        VerilatedModelConfig::default(),
    )?;

    let test_input: [u32; 7] = [0xDEADBEEF_u32, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222, 0x55];
    main.pin("very_wide_input", &test_input)
        .whatever_context("pin")?;
    assert_eq!(
        main.read("very_wide_output").whatever_context("first read")?,
        VerilatorValue::WDataOutP(vec![0, 0, 0, 0, 0, 0, 0])
    );
    main.eval();
    assert_eq!(
        main.read("very_wide_output").whatever_context("second read")?,
        VerilatorValue::WDataOutP(test_input.to_vec())
    );

    Ok(())
}

#[test]
fn forwards_correctly_very_wide_registered() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts_very_wide_reg".into(),
        &["src/very_wide_registered.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model_simple::<VeryWideRegistered>()?;

    let test_input = [0xDEADBEEF_u32, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222, 0x55];
    main.very_wide_input = WideIn::new(test_input);
    main.eval();

    main.clk = 1;
    main.eval();
    assert_eq!(main.very_wide_output.value(), &test_input);

    Ok(())
}

#[test]
fn forwards_correctly_very_wide_registered_dynamically() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts_very_wide_reg_dyn".into(),
        &["src/very_wide_registered.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_dyn_model(
        "very_wide_registered",
        "src/very_wide_registered.sv",
        &[
            ("clk", 0, 0, PortDirection::Input),
            ("very_wide_input", 199, 0, PortDirection::Input),
            ("very_wide_output", 199, 0, PortDirection::Output),
        ],
        VerilatedModelConfig::default(),
    )?;

    let test_input: [u32; 7] = [0xDEADBEEF_u32, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222, 0x55];
    main.pin("very_wide_input", &test_input)
        .whatever_context("pin")?;
    main.pin("clk", 0u8).whatever_context("pin clk")?;
    main.eval();

    main.pin("clk", 1u8).whatever_context("pin clk")?;
    main.eval();
    assert_eq!(
        main.read("very_wide_output").whatever_context("read")?,
        VerilatorValue::WDataOutP(test_input.to_vec())
    );

    Ok(())
}
