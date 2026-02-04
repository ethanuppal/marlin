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

use example_verilog_project::{
    VeryWide1024, VeryWideMain, VeryWideRegistered,
};
use marlin::verilator::{
    AsDynamicVerilatedModel, PortDirection, VerilatedModelConfig,
    VerilatorRuntime, VerilatorRuntimeOptions, WideIn, dynamic::VerilatorValue,
};
use snafu::{ResultExt, Whatever};

fn make_runtime(
    artifacts_dir: &str,
    sv_path: &str,
) -> Result<VerilatorRuntime, Whatever> {
    VerilatorRuntime::new(
        artifacts_dir.into(),
        &[sv_path.as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )
}

fn run_dynamic(
    artifacts_dir: &str,
    module_name: &str,
    sv_path: &str,
    msb: u32,
    test_input: &[u32],
) -> Result<(), Whatever> {
    let runtime = make_runtime(artifacts_dir, sv_path)?;
    let mut main = runtime.create_dyn_model(
        module_name,
        sv_path,
        &[
            ("very_wide_input", msb, 0, PortDirection::Input),
            ("very_wide_output", msb, 0, PortDirection::Output),
        ],
        VerilatedModelConfig::default(),
    )?;

    main.pin("very_wide_input", test_input)
        .whatever_context("pin")?;
    main.eval();
    assert_eq!(
        main.read("very_wide_output").whatever_context("read")?,
        VerilatorValue::WDataOutP(test_input.to_vec())
    );

    Ok(())
}

#[test]
fn forwards_correctly_very_wide_static_and_dynamic() -> Result<(), Whatever> {
    let runtime =
        make_runtime("artifacts_very_wide", "src/very_wide_main.sv")?;

    let mut main = runtime.create_model_simple::<VeryWideMain>()?;

    let test_input = [
        0xDEADBEEF_u32,
        0xCAFEBABE,
        0x12345678,
        0x9ABCDEF0,
        0x11111111,
        0x22222222,
        0x55,
    ];
    main.very_wide_input = WideIn::new(test_input);
    main.eval();
    assert_eq!(main.very_wide_output.value(), &test_input);

    run_dynamic(
        "artifacts_very_wide_dyn",
        "very_wide_main",
        "src/very_wide_main.sv",
        199,
        &test_input,
    )?;

    Ok(())
}

#[test]
fn forwards_correctly_very_wide_registered_static_and_dynamic(
) -> Result<(), Whatever> {
    let runtime = make_runtime(
        "artifacts_very_wide_reg",
        "src/very_wide_registered.sv",
    )?;

    let mut main = runtime.create_model_simple::<VeryWideRegistered>()?;

    let test_input = [
        0xDEADBEEF_u32,
        0xCAFEBABE,
        0x12345678,
        0x9ABCDEF0,
        0x11111111,
        0x22222222,
        0x55,
    ];
    main.very_wide_input = WideIn::new(test_input);
    main.eval();

    main.clk = 1;
    main.eval();
    assert_eq!(main.very_wide_output.value(), &test_input);

    let runtime = make_runtime(
        "artifacts_very_wide_reg_dyn",
        "src/very_wide_registered.sv",
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

    let test_input: [u32; 7] = [
        0xDEADBEEF_u32,
        0xCAFEBABE,
        0x12345678,
        0x9ABCDEF0,
        0x11111111,
        0x22222222,
        0x55,
    ];
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

#[test]
fn forwards_correctly_very_wide_1024_static_and_dynamic(
) -> Result<(), Whatever> {
    let runtime =
        make_runtime("artifacts_very_wide_1024", "src/very_wide_1024.sv")?;

    let mut main = runtime.create_model_simple::<VeryWide1024>()?;

    let test_input: [u32; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    ];
    main.very_wide_input = WideIn::new(test_input);
    main.eval();
    assert_eq!(main.very_wide_output.value(), &test_input);

    run_dynamic(
        "artifacts_very_wide_1024_dyn",
        "very_wide_1024",
        "src/very_wide_1024.sv",
        1023,
        &test_input,
    )?;

    Ok(())
}
