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
use marlin::verilator::{
    VerilatedModelConfig, VerilatorRuntime, VerilatorRuntimeOptions,
};
use snafu::Whatever;

#[test]
//#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
    )?;

    let mut main = runtime.create_model::<Main>(&VerilatedModelConfig {
        enable_tracing: true,
        ..Default::default()
    })?;

    let mut vcd = main.open_vcd("foo.vcd");

    vcd.dump(0);

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    vcd.dump(1);
    vcd.dump(2);

    Ok(())
}
