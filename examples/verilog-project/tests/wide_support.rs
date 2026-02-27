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

use example_verilog_project::{WideMain, WideMain2, WideMain3, WideMain4};
use marlin::verilator::{
    AsDynamicVerilatedModel, PortDirection, VerilatedModelConfig,
    VerilatorRuntime, VerilatorRuntimeOptions, WideIn,
};
use snafu::Whatever;

#[test]
#[snafu::report]
fn all_wide_mains_forward_correctly() -> Result<(), Whatever> {
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

    let mut main2 = runtime.create_model_simple::<WideMain2>()?;

    main2.wide_input = WideIn::new([u32::MAX, u32::MAX, 1, 2]);
    println!("{:?}", main2.wide_output);
    assert_eq!(main2.wide_output.value(), &[0; 4]);
    main2.eval();
    println!("{:?}", main2.wide_output);
    assert_eq!(main2.wide_output.value(), &[u32::MAX, u32::MAX, 1, 2]);

    let mut main3 = runtime.create_model_simple::<WideMain3>()?;

    main3.wide_input = WideIn::new([u32::MAX, u32::MAX, 1, 2, 3, 4, 5, 6]);
    println!("{:?}", main3.wide_output);
    assert_eq!(main3.wide_output.value(), &[0; 8]);
    main3.eval();
    println!("{:?}", main3.wide_output);
    assert_eq!(
        main3.wide_output.value(),
        &[u32::MAX, u32::MAX, 1, 2, 3, 4, 5, 6]
    );

    let mut main4 = runtime.create_model_simple::<WideMain4>()?;

    main4.wide_input = WideIn::new([u32::MAX, u32::MAX, 1, 2]);
    println!("{:?}", main4.wide_output);
    assert_eq!(main4.wide_output.value(), &[0; 4]);
    main4.eval();
    println!("{:?}", main4.wide_output);
    assert_eq!(main4.wide_output.value(), &[u32::MAX, u32::MAX, 1, 2]);

    Ok(())
}

#[test]
#[snafu::report]
fn wide_main_forwards_correctly_dynamically() -> Result<(), Whatever> {
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

    main.pin("wide_input", &[u32::MAX, u32::MAX, 1]).unwrap();
    assert_eq!(main.read("wide_output").unwrap(), [0; 3].into());
    main.eval();
    assert_eq!(
        main.read("wide_output").unwrap(),
        [u32::MAX, u32::MAX, 1].into()
    );

    Ok(())
}

#[test]
#[snafu::report]
fn wide_main4_forwards_correctly_dynamically() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["src/wide_main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    println!("processed");

    println!("var");
    let mut main4 = runtime.create_dyn_model(
        "wide_main4",
        "src/wide_main.sv",
        &[
            ("wide_input", 255, 128, PortDirection::Input),
            ("wide_output", 255, 128, PortDirection::Output),
        ],
        VerilatedModelConfig::default(),
    )?;
    println!("foo");

    main4
        .pin("wide_input", &[u32::MAX, u32::MAX, 1, 2])
        .unwrap();
    assert_eq!(main4.read("wide_output").unwrap(), [0; 4].into());
    main4.eval();
    assert_eq!(
        main4.read("wide_output").unwrap(),
        [u32::MAX, u32::MAX, 1, 2].into()
    );

    Ok(())
}
