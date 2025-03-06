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

use std::env;

use example_spade_project::Main;
use marlin::spade::prelude::*;
use snafu::Whatever;

#[test]
#[snafu::report]
fn main() -> Result<(), Whatever> {
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

    let mut runtime =
        SpadeRuntime::new(SpadeRuntimeOptions::default_logging())?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
