# Calling Rust from Verilog

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll explore how to use dumbname to call Rust functions from
Verilog. Learn more about [DPI in general here](https://verilator.org/guide/latest/connecting.html#direct-programming-interface-dpi).
You can find the full source code for this tutorial [here](../examples/verilog-project/) (in the `dpi_tutorial.rs` file).

I'll be assuming you've read the [tutorial on testing Verilog projects](./testing_verilog.md); if not, read that first and come back.
In particular, I won't be reexplaining things I discussed in that tutorial, although I will still walk through the entire setup.

## Part 1: The Basics

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
├── src
│   └── dpi.sv
└── test
    └── dpi_test.rs
```

We'll have a simple SystemVerilog module that writes the result of `three`, a
DPI function with a single integer output.
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
mkdir test
vi Cargo.toml
vi test/dpi_test.rs
```

Next, we'll add dumbname and other desired dependencies.
```toml
# file: Cargo.toml
[package]
name = "tutorial-project"

[[bin]]
name = "dpi_test"
path = "test/dpi_test.rs"

[dependencies]
# other dependencies...
verilog = { git = "https://github.com/ethanuppal/dumbname" }
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

Finally, we need the Rust file where we define the DPI function and drive the
model.

```rust
// file: test/dpi_test.rs
use snafu::Whatever;
use verilog::{verilog, VerilatorRuntime, VerilatorRuntimeOptions};

#[verilog::dpi]
#[no_mangle]
extern "C" fn three(#[output] out: &mut u32) {
    *out = 3;
}

#[verilog(src = "src/dpi.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
        [three],
        VerilatorRuntimeOptions::default(),
        true,
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
#[no_mangle]
extern "C" fn three(#[output] out: &mut u32) {
    *out = 3;
}
```
By applying `#[verilog::dpi]`, we turn our normal Rust function into a DPI one.
We need to apply `#[no_mangle]` and `extern` (or `extern "C"`) so that Rust
exposes the function correctly to C. 

DPI functions cannot have a return value and only take primitive integers or mutable references to primitive integers as
arguments. Each parameter must be annotated with `#[input]`, `#[output]`, or
`#[inout]`. Moreover, their bodies can only access the standard library.

Then, we told the runtime about this function:
```diff
    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/dpi.sv".as_ref()],
-       [],
+       [three],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

```
