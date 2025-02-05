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

use snafu::{ResultExt, Whatever};
use verilog::{PortDirection, VerilatorRuntime, VerilatorRuntimeOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["sv/main.sv".as_ref()],
        [],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

    let mut main = runtime.create_dyn_model(
        "main",
        "sv/main.sv",
        &[
            ("medium_input", 31, 0, PortDirection::Input),
            ("medium_output", 31, 0, PortDirection::Output),
        ],
    )?;

    main.pin("medium_input", u32::MAX).whatever_context("pin")?;
    println!("{}", main.read("medium_output").whatever_context("read")?);
    assert_eq!(
        main.read("medium_output").whatever_context("read")?,
        0u32.into()
    );
    main.eval();
    println!("{}", main.read("medium_output").whatever_context("read")?);
    assert_eq!(
        main.read("medium_output").whatever_context("read")?,
        u32::MAX.into()
    );

    Ok(())
}
