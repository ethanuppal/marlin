# Verilog Quickstart

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a SystemVerilog project and test our code with
Marlin. You can find the full source code for this tutorial [here](https://github.com/ethanuppal/marlin/tree/main/examples/verilog-project) (see in particular the `simple_test.rs` file).
We won't touch on the advanced aspects or features; the goal is just to provide a simple overfiew sufficient to get started.

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
├── .gitignore
├── src
│   ├── lib.rs
│   ├── main.sv
└── tests
    └── simple_test.rs
```

We'll write a very simple SystemVerilog module: one that forwards its inputs to
its outputs.
```shell
mkdir src
vi src/main.sv
```
I'm using the `vi` editor here, but you can use whichever editor you prefer.

For our forwarding module, we'll just pass a medium-sized input to a
corresponding output:
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

Now that we have the setup out of the way, we can start testing our code from Rust.
We'll initialize a Rust project:

```shell
cargo init --lib
vi src/lib.rs
vi test/simple_test.rs
```

In the `Cargo.toml` generated, we'll want to add some dependencies:

```toml
# file: Cargo.toml
[dependencies]
# other dependencies...
marlin = { version = "0.1.0", features = ["verilog"] }
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

The only required crate is `marlin`, but I strongly recommend at this stage of
development to use the other two crates as well.

> [!NOTE]
> We're including the [`colog`](color) crates as a backend for the well-known [`log`][log]
> crate. That's because, if you enable verbose mode on Marlin runtimes, it will
> use the [`log`][log] API to print out information. You can use whatever logging
> backend you want; I believe the most popular is [`env_logger`][env_logger].

In the `lib.rs`, we'll create the binding to our Verilog module:

```rust
// file: src/lib.rs
use marlin::verilog::prelude::*;

#[verilog(src = "src/main.sv", name = "main")]
pub struct Main;
```

This tells Marlin that the `struct Main` should be linked to the `main` module
in our Verilog file.

Finally, we'll want to actually write the code that drives our project in `simple_test.rs`:

```rust
// file: tests/simple_test.rs
use tutorial_project::Main;
use marlin::verilator::{VerilatorRuntime, VerilatorRuntimeOptions};
use snafu::Whatever;

#[test]
#[snafu::report]
fn forwards_u32max_correctly() -> Result<(), Whatever> {
    let mut runtime = VerilatorRuntime::new(
        "build".into(),
        &["src/main.sv".as_ref()],
        &[],
        [],
        VerilatorRuntimeOptions::default(),
        true,
    )?;

    let mut main = runtime.create_model::<Main>()?;

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    Ok(())
}
```

Let's break down the relevant parts of what's going on here.

We first setup the Verilator runtime configuration. We'll use a build directory
called "build" in the local directory.
```rust
let mut runtime = VerilatorRuntime::new(
    "build".into(),                     // build directory (relative path)
    &["src/main.sv".as_ref()],          // source files
    &[],                                // include search paths
    [],                                 // DPI functions
    VerilatorRuntimeOptions::default(), // configuration
    true,                               // enable logging with the log crate
)?;
```

> [!TIP]
> Add this build directory to your `.gitignore` file if you're using `git`.

You can fill in the source files (2nd argument) by, for example, finding all `.v` files in a
source direcory with `std::fs::read_dir`. Since we only have one, we've
hardcoded it.

Then, we instantiate the model:
```rust
let mut main = runtime.create_model::<Main>()?;
``` 

I won't comment on the rest; it's just regular Rust --- including the part where
we assign to values and call `eval()` on the model object! (Yes, that is the
same as Verilator's evaluation method).

Finally, we can simply use `cargo test` to drive our design!

[colog]: https://docs.rs/colog/latest/colog/
[log]: https://docs.rs/log/latest/log/
[env_logger]: https://docs.rs/env_logger/latest/env_logger/
