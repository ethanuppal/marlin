# Veryl Quickstart

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

> [!CAUTION]
> Veryl support is still experimental.

In this tutorial, we'll setup a Veryl project and test our code with
Marlin. You can find the full source code for this tutorial [here](https://github.com/ethanuppal/marlin/tree/main/examples/veryl_project).

**I'll be assuming you've read the [tutorial on testing Verilog projects](../verilog/quickstart.md); if not, read that first and come back.**

Also, make sure you have a [Veryl toolchain installed](https://veryl-lang.org/install/).

> [!NOTE]
> If you already have a Veryl project and are looking to integrate Marlin into
> it, you don't need to read Part 1 too carefully.

## Part 1: Making a Veryl Project

Let's call our project "tutorial_project" (you are free to call it however you
like):
```shell
veryl new tutorial_project
cd tutorial-project
git init # optional, if using git
```

Here's what our project will look like in the end:

```.
├── Veryl.toml
├── Veryl.lock
├── Cargo.toml
├── Cargo.lock
├── src
│   ├── lib.rs
│   └── main.veryl
└── tests
    └── simple_test.rs
```

In `main.veryl`, we'll write some simple Veryl code:

```shell
mkdir src
vi src/main.veryl
```

```systemverilog
// file: src/main.veryl
module Wire(
    medium_input: input logic<32>,
    medium_output: output logic<32>
) {
    assign medium_output = medium_input;
}
```

You can read the [Veryl book](https://doc.veryl-lang.org/book/01_introduction.html) for an
introduction to Veryl; this tutorial will not focus on teaching the language. If
you know Verilog, the code should feel very familiar.

## Part 2: Setting up Marlin

```shell
cargo init --lib
cargo add marlin --features veryl --dev
cargo add snafu --dev
```
The only required crate is `marlin`, but I strongly recommend at this stage of
development to use `snafu`, which will display a human-readable error trace upon
`Result::Err`.

> [!CAUTION]
> Please use `snafu`! 😂

In the test file, we'll create the binding to our Veryl module:
```shell
mkdir tests
vi tests/simple_test.rs
```

```rust
// file: tests/simple_test.rs
use marlin::veryl::prelude::*;

#[veryl(src = "src/main.veryl", name = "Wire")]
pub struct Wire;
```

This tells Marlin that the `struct Wire` should be linked to the `Wire` module
in our Veryl file. You can instead put this in your `lib.rs` file if you prefer.

Finally, we'll want to actually write the code that drives our hardware in `simple_test.rs`:

```rust
// file: tests/simple_test.rs
use marlin::veryl::prelude::*;
use snafu::Whatever;

#[veryl(src = "src/main.veryl", name = "Wire")]
pub struct Wire;

#[test]
//#[snafu::report]
fn forwards_correctly() -> Result<(), Whatever> {
    let runtime = VerylRuntime::new(VerylRuntimeOptions {
        call_veryl_build: true, /* warning: not thread safe! don't use if you
                                 * have multiple tests */
        ..Default::default()
    })?;

    let mut main = runtime.create_model::<Wire>()?;

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, 0);
    main.eval();
    println!("{}", main.medium_output);
    assert_eq!(main.medium_output, u32::MAX);

    Ok(())
}
```

> [!CAUTION]
> Using `#[snafu::report]` on the function gives error messages that are
> actually useful, but sometimes breaks LSP services like code completion.
> I recommend to only apply it to your test functions when you actually
> encounter an error.

Finally, we can simply use `cargo test` to drive our design! It will take a while before it starts doing Marlin dynamic compilation because it needs to first build the Veryl project by invoking the Veryl compiler.
