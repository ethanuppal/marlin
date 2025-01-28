// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use snafu::Whatever;
use spade::{spade, SpadeRuntime, SpadeRuntimeOptions};

#[spade(src = "src/main.spade", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = SpadeRuntime::new(SpadeRuntimeOptions::default(), true)?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
