
# Using dynamic Verilog models

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll explore how to use dumbname to dynamically create
bindings to Verilog modules.
You can find the full source code for this tutorial [here](../verilog-support/example-project/) (in the `dyamic_model.rs` file).

I'll be assuming you've read the [tutorial on testing Verilog projects](./testing_verilog.md); if not, read that first and come back.
In particular, I won't be reexplaining things I discussed in that tutorial,
although I will still walk through the entire setup.

## Part 1: The Basics

Let's call our project "tutorial-project-2" (you are free to call it however you
like):
```shell
mkdir tutorial-project-2
cd tutorial-project-2
git init # optional, if using git
```

Let's use the same SystemVerilog module from the [Verilog tutorial](./testing_verilog.md).
```shell
mkdir sv
vi sv/main.sv
```

```systemverilog
// file: sv/main.sv
module main(
    input[31:0] medium_input,
    output[31:0] medium_output
);
    assign medium_output = medium_input;
endmodule
```

## Part 2: Testing

We'll create a new Rust project:
```shell
cargo init --bin .
```

Next, we'll add dumbname and other desired dependencies.
```toml
# file: Cargo.toml
[dependencies]
# other dependencies...
verilog = { git = "https://github.com/ethanuppal/dumbname" }
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

The code for dynamic models is slightly more verbose.
It's not necessarily meant for human usage, though; this API is better suited for
using dumbname as a library (e.g., writing an interpreter).

```rust
// file: src/main.rs
use snafu::Whatever;
use verilog::{VerilatorRuntime, PortDirection};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts2".into(),
        &["sv/main.sv".as_ref()],
        true,
    )?;

    let mut main = runtime.create_dyn_model(
        "main",
        "sv/sv.main",
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
```

We can `cargo run` as usual to test.

Make sure you pass in the correct filename to `create_dyn_model`.
You only need to pass in a correct _subset_ of the ports.

One current issue is that if you use multiple dynamic models, since models are
lazy-built and cached, omitting ports in the first `create_dyn_model` for a
given module means that no later model can access those omitted ports.
