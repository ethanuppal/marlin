# Dynamic Bindings to Verilog

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a SystemVerilog project and test our code with
Marlin. You can find the full source code for this tutorial [here](https://github.com/ethanuppal/marlin/tree/main/examples/verilog-project) (see in particular the `dynamic_model_tutorial.rs` file).

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
│   ├── main.sv
└── tests
    └── dynamic_test.rs
```

Let's use the same SystemVerilog module from the [Verilog quickstart](./quickstart.md).
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
cargo init --lib
```

Next, we'll add Marlin and other desired dependencies.

```shell
cargo add marlin --dev # no features sneeded
cargo add colog --dev
cargo add snafu --dev
```

We will illustrate using dynamic models by implementing the exact same test as
we did in the [Verilog quickstart](./quickstart.md).

The code for dynamic models is slightly more verbose.
It's not necessarily meant for human usage, though; this API is better suited for
using Marlin as a library (e.g., writing an interpreter).

```shell
mkdir tests
vi tests/dynamic_test.rs
```

```rust
// file: tests/simple_test.rs
use snafu::Whatever;
use marlin::verilator::{
    PortDirection, VerilatorRuntime, VerilatorRuntimeOptions,
};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "build2".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default_logging(),
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

We can `cargo test` as usual to test.
If you're using `git`, remember to add `build2/` to your `.gitignore`.

Make sure you pass in the correct filename to `create_dyn_model`.
You only need to pass in a correct _subset_ of the ports.

> [!WARNING]
> One current issue is that if you use multiple dynamic models, since models are
> lazy-built and cached, omitting ports in the first `create_dyn_model` for a
> given module means that no later model can access those omitted ports.
