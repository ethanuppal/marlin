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

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "<non-string panic>".to_string()
    }
}

#[test]
//#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = make_runtime("artifacts2", "src/wide_main.sv")?;

    let mut main = runtime.create_model_simple::<WideMain>()?;

    main.wide_input = WideIn::new([u32::MAX, u32::MAX, 1]);
    main.eval();
    assert_eq!(main.wide_output.value(), &[u32::MAX, u32::MAX, 1]);

    Ok(())
}

#[test]
//#[snafu::report]
fn forwards_correctly_dynamically() -> Result<(), Whatever> {
    let runtime = make_runtime("artifacts2", "src/wide_main.sv")?;

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
    main.eval();
    assert_eq!(
        main.read("wide_output").whatever_context("second read")?,
        VerilatorValue::WDataOutP(vec![u32::MAX, u32::MAX, 1])
    );

    Ok(())
}

#[test]
fn uninitialized_wide_out_behaves_safely() -> Result<(), Whatever> {
    let runtime = make_runtime("artifacts_init_test", "src/wide_main.sv")?;
    let mut main = runtime.create_model_simple::<WideMain>()?;

    assert!(!main.wide_output.is_initialized());

    let debug_str = format!("{:?}", main.wide_output);
    assert!(debug_str.contains("<uninitialized>"));

    let value: VerilatorValue = (&main.wide_output).into();
    assert_eq!(value, VerilatorValue::NotDriven);

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = main.wide_output.value();
    }));
    let message = panic_message(panic.expect_err("expected panic"));
    assert!(message.contains("WideOut::value() called on uninitialized port"));

    main.eval();
    assert!(main.wide_output.is_initialized());

    Ok(())
}
