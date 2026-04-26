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

use example_verilog_project::Main;
use marlin::{
    verilator::{
        VerilatedModelConfig, VerilatorRuntime, VerilatorRuntimeOptions,
    },
    verilog::prelude::*,
};
use snafu::Whatever;

#[test]
#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    // reaches
    // println!("Test 1");
    // assert!(false);

    let mut main = runtime.create_model::<Main>(&VerilatedModelConfig {
        enable_tracing: true,
        ..Default::default()
    })?;

    // reaches
    // println!("Test 2");
    // assert!(false);

    let mut vcd = main.open_vcd("foo.vcd");

    // reaches
    // println!("Test 3");
    // assert!(false);

    vcd.dump(0);

    // reaches
    // println!("Test 4");
    // assert!(false);

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();

    // reaches
    // println!("Test 5");
    // assert!(false);
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    // vcd.dump(1);

    // reaches
    println!("Test 6");
    // assert!(false);
    // vcd.dump(2);

    println!("Test 7");
    assert!(false);

    println!("Test 8");
    assert!(false);

    Ok(())
}
