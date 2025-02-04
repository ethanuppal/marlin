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

use snafu::Whatever;
use verilog::{verilog, VerilatorRuntime, VerilatorRuntimeOptions};

#[verilog::dpi]
#[no_mangle]
extern "C" fn three(#[output] out: &mut u32) {
    *out = 3;
}

#[verilog(src = "sv/dpi.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["sv/dpi.sv".as_ref()],
        [three],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

    let mut main = runtime.create_model::<Main>()?;
    main.eval();
    assert_eq!(main.out, 3);

    Ok(())
}
