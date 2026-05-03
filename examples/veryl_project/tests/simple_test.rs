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

use example_veryl_project::{U8Counter, Wire};
use marlin::{verilator::verilator_version, veryl::prelude::*};
use snafu::Whatever;

#[test]
#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = VerylRuntime::new(
        VerylRuntimeOptions::default()
            .call_veryl_build(
                true, /* warning: not thread safe! don't use if you have
                      * multiple tests */
            )
            .with_inner(|verilator_options| {
                verilator_options.allow_unsupported_verilator(Some(
                    verilator_version!(5 020),
                ))
            }),
    )?;

    let mut main = runtime.create_model::<Wire>()?;

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    Ok(())
}

#[test]
#[snafu::report]
fn counter_resets() -> Result<(), Whatever> {
    let runtime = VerylRuntime::new(
        VerylRuntimeOptions::default()
            .call_veryl_build(
                true, /* warning: not thread safe! don't use if you have
                      * multiple tests */
            )
            .with_inner(|verilator_options| {
                verilator_options.allow_unsupported_verilator(Some(
                    verilator_version!(5 020),
                ))
            }),
    )?;

    let mut main = runtime.create_model::<U8Counter>()?;

    main.i_reset = 1;

    main.i_clk = 0;
    main.eval();
    main.i_clk = 1;
    main.eval();

    main.i_reset = 0;

    assert_eq!(main.value, 0);

    main.i_clk = 0;
    main.eval();
    main.i_clk = 1;
    main.eval();

    assert_eq!(main.value, 0);

    main.enable = 1;
    main.i_clk = 0;
    main.eval();
    main.i_clk = 1;
    main.eval();

    assert_eq!(main.value, 1);

    Ok(())
}
