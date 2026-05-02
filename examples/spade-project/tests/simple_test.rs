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

use example_spade_project::Main;
use marlin::{spade::prelude::*, verilator::verilator_version};
use snafu::Whatever;

#[test]
#[snafu::report]
fn main() -> Result<(), Whatever> {
    let runtime = SpadeRuntime::new(
        SpadeRuntimeOptions::default()
            .call_swim_build(
                true, /* warning: not thread safe! don't use if you
                      * have multiple tests */
            )
            .with_inner(|verilator_options| {
                verilator_options.allow_unsupported_verilator(Some(
                    verilator_version!(5 020),
                ))
            }),
    )?;

    let mut main = runtime.create_model_simple::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
