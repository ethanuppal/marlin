
# Using dynamic Verilog models

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll explore how to use Marlin to dynamically create
bindings to Verilog modules.
You can find the full source code for this tutorial [here](../examples/verilog-project/) (in the `dyamic_model_tutorial.rs` file).

I'll be assuming you've read the [tutorial on testing Verilog projects](./testing_verilog.md); if not, read that first and come back.
In particular, I won't be reexplaining things I discussed in that tutorial,
although I will still walk through the entire setup.

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
│   └── main.sv
└── test
    └── simple_test.rs
```

Let's use the same SystemVerilog module from the [Verilog tutorial](./testing_verilog.md).
```shell
mkdir src
vi src/main.sv
```

```systemverilog
// file: src/main.sv
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
mkdir test
vi Cargo.toml
vi test/dpi_test.rs
```

Next, we'll add Marlin and other desired dependencies.
```toml
# file: Cargo.toml
[package]
name = "tutorial-project"

[[bin]]
name = "simple_test"
path = "test/simple_test.rs"

[dependencies]
# other dependencies...
marlin = "0.1.0" # no language features needed
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

The code for dynamic models is slightly more verbose.
It's not necessarily meant for human usage, though; this API is better suited for
using Marlin as a library (e.g., writing an interpreter).

```rust
// file: test/simple_test.rs
use snafu::Whatever;
use marlin::verilator::{
    PortDirection, VerilatorRuntime, VerilatorRuntimeOptions,
};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        true,
    )?;

    let mut main = runtime.create_dyn_model(
        "main",
        "src/main.sv",
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
