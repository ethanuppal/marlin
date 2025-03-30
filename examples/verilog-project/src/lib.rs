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

use marlin::verilog::prelude::*;

#[verilog(src = "src/main.sv", name = "main")]
pub struct Main;

#[verilog(src = "src/dpi.sv", name = "dpi_main")]
pub struct DpiMain;

#[verilog(src = "src/more_dpi.sv", name = "dpi_main")]
pub struct MoreDpiMain;

pub mod enclosed {
    use marlin::verilog::prelude::*;

    #[verilog(src = "src/main.sv", name = "main")]
    pub struct Main2;
}

/// Compiles if we can parse the `var` keyword.
mod parses_var_test {
    use marlin::{verilator::types, verilog::prelude::*};

    #[verilog(src = "src/parse_var.sv", name = "main")]
    pub struct ParsesVarTest;

    #[allow(unused)]
    const fn size_of_return_type<T, U>(f: impl FnOnce(T) -> U) -> usize {
        std::mem::forget(f);
        size_of::<U>()
    }

    const _: () = assert!(
        size_of_return_type::<ParsesVarTest, _>(|dut| dut.clk)
            == size_of::<types::CData>()
    );
}
