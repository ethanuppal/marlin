# Testing a Verilog project

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a SystemVerilog project and test our code with dumbname.
You can find the full source code for this tutorial [here](../examples/verilog-project/) (in the `tutorial.rs` file).
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
├── src
│   └── main.sv
└── test
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
mkdir test
vi Cargo.toml
vi test/simple_test.rs
```

In the `Cargo.toml` generated, we'll want to add some dependencies:

```toml
# file: Cargo.toml
[package]
name = "tutorial-project"

[[bin]]
name = "simple_test"
path = "test/simple_test.rs"

[dependencies]
# other dependencies...
verilog = { git = "https://github.com/ethanuppal/dumbname" }
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

The only required package is `verilog` from dumbname; everything else is just
for fun.
It's a good idea to fix a particular revision at this stage of development (and
make sure to update it frequently insofar as it doesn't break your code!).

Finally, we'll want to actually write the code that drives our project in `simple_test.rs`:

```rust
// file: test/simple_test.rs
use snafu::Whatever;
use verilog::{verilog, VerilatorRuntime};

#[verilog(src = "src/main.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
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

Let's break down the relevant parts of what's going on here:

1. Binding at compile time:
    ```rust
    #[verilog(src = "src/main.sv", name = "main")]
    struct Main;
    ``` 

    This snippet declares that the Rust `struct Main` binds to the Verilog module `main` as
    defined in `src/main.sv` (this path is relative to the `Cargo.toml` parent directory).

2. Binding at runtime:
    ```rust
    let mut runtime = VerilatorRuntime::new(
        "artifacts".into(),
        &["src/main.sv".as_ref()],
        true,
    )?;
    ``` 
    This line creates a Verilog runtime powered by verilator, allowing you to run Verilog
    from Rust.

3. Using at runtime: 
    ```rust
    let mut main = runtime.create_model::<Main>()?;
    ``` 
    This line asks the runtime to create a new version of `Main`, that is, our `main`
    model.

I won't comment on the rest; it's just regular Rust --- including the part where
we assign to values and call `eval()` on the model object! (Yes, that is the
same as Verilator's evaluation method).

> [!TIP]
> If you are using `git`, add the `artifacts/` directory managed by the Verilator
runtime to your `.gitignore`.

Finally, we can simply use `cargo run` to drive our design!
