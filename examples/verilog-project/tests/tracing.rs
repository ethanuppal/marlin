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
        tracing::Waveform, verilator_version, VerilatedModelConfig,
        VerilatorRuntime, VerilatorRuntimeOptions,
    },
    verilog::prelude::*,
};
use snafu::Whatever;

#[test]
#[snafu::report]
fn forwards_correctly_vcd() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default()
            .allow_unsupported_verilator(Some(verilator_version!(5 020))),
    )?;

    let mut main = runtime.create_model::<Main>(
        &VerilatedModelConfig::default().enable_tracing(Some(Waveform::Vcd)),
    )?;

    let mut vcd = main.open_trace("foo.vcd");

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

#[test]
#[snafu::report]
fn forwards_correctly_fst() -> Result<(), Whatever> {
    let runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default()
            .allow_unsupported_verilator(Some(verilator_version!(5 020))),
    )?;

    let mut main = runtime.create_model::<Main>(
        &VerilatedModelConfig::default().enable_tracing(Some(Waveform::Fst)),
    )?;

    let mut fst = main.open_trace("foo.fst");

    fst.dump(0);

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    fst.dump(1);
    fst.dump(2);

    Ok(())
}
