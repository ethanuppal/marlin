# Calling Rust from Verilog

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a SystemVerilog project and test our code with
Marlin. You can find the full source code for this tutorial [here](https://github.com/ethanuppal/marlin/tree/main/examples/verilog-project) (see in particular the `dpi_tutorial.rs` file).

I'll be assuming you've read the [tutorial on testing Verilog projects](./quickstart.md); if not, read that first and come back.
In particular, I won't be reexplaining things I discussed in that tutorial,
although I will still walk through the entire setup.

## Part 1: Setup

Let's call our project "tutorial-project" (you are free to call it however you
like):
```shell
mkdir tutorial-project
cd tutorial-project
git init # optional, if using git
```

Here's what our project will look like in the end:

```
.
├── Cargo.toml
├── .gitignore
├── src
│   ├── lib.rs
│   ├── dpi.sv
└── tests
    └── dpi_test.rs
```

We'll have a simple SystemVerilog module that writes the result of `three`, a DPI function with a single integer output.

```shell
mkdir src
vi src/dpi.sv
```

```systemverilog
// file: src/dpi.sv
import "DPI-C" function void three(output int out);

module main(output logic[31:0] out);
    int a = 0;
    initial begin
        three(a);
        $display("%d", a);
        out = a;
    end
endmodule
```

## Part 2: Testing

We'll create a new Rust project:

```shell
cargo init --lib
```

Next, we'll add Marlin and other desired dependencies.

```shell
cargo add marlin --features verilog --dev
cargo add snafu --dev
```

Finally, we need the Rust file where we define the DPI function and drive the model.

```shell
mkdir tests
vi tests/dpi_test.rs
```

```rust
// file: tests/dpi_test.rs
use snafu::Whatever;
use marlin::{
    verilator::{VerilatorRuntime, VerilatorRuntimeOptions},
    verilog::prelude::*,
};

#[verilog::dpi]
pub extern "C" fn three(out: &mut u32) {
    *out = 3;
}

#[verilog(src = "src/dpi.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
        &[],
        [three],
        VerilatorRuntimeOptions::default(),
    )?;

    let mut main = runtime.create_model::<Main>()?;
    main.eval();
    assert_eq!(main.out, 3);

    Ok(())
}
```

We can `cargo run` as usual to test.

The magic happens here:

```rust
#[verilog::dpi]
pub extern fn three(out: &mut u32) {
    *out = 3;
}
```
By applying `#[verilog::dpi]`, we turn our normal Rust function into a DPI one.
We need to apply `pub` and `extern` (or `extern "C"`) so that Rust
exposes the function correctly to C. 

DPI functions cannot have a return value and only take primitive integers (for `input`) or mutable references to primitive integers (for `output`/`inout`) as
arguments. Beside that, there are no restrictions on the content --- write
whatever Rust code you want!

Then, we told the runtime about this function:
```diff
    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
        &[],
-       [],
+       [three],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

```
