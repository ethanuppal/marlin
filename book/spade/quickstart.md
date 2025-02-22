# Spade Quickstart

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a Spade project and test our code with
Marlin. You can find the full source code for this tutorial [here](https://github.com/ethanuppal/marlin/tree/main/examples/spade-project).

I'll be assuming you've read the [tutorial on testing Verilog projects](../verilog/quickstart.md); if not, read that first and come back.

Also, make sure you have a Spade toolchain installed, although we'll only be using the Swim build tool
(follow the instructions [here](https://docs.spade-lang.org/swim/install.html)
to install it).

> [!NOTE]
> If you already have a Swim project and are looking to integrate Marlin into
> it, you don't need to read Part 1 too carefully.

## Part 1: Making a Swim Project

Let's call our project "tutorial-project" (you are free to call it however you
like):
```shell
swim init tutorial-project
cd tutorial-project
git init # optional, if using git
```

Here's what our project will look like in the end:

```
.
├── swim.toml
├── swim.lock
├── Cargo.toml
├── src
│   ├── lib.rs
│   └── main.spade
└── tests
    └── simple_test.rs
```

In `main.spade` (which should already exist after running `swim init`), we'll write some simple Spade code:

```spade
// file: src/main.spade
#[no_mangle(all)]
entity main(out: inv &int<8>) {
    set out = 42;
}}
```

You can read the [Spade book](https://docs.spade-lang.org/introduction.html) for an
introduction to Spade; this tutorial will not focus on teaching the language.
Nonetheless, the essence of the above code is to expose an inverted wire which
we pin to the value `42` (think of `assign`ing to an `output` in Verilog).
We'll write a very simple SystemVerilog module: one that forwards its inputs to
its outputs.

## Part 2: Setting up Marlin

```shell
cargo init --lib
cargo add marlin --features spade --dev
cargo add colog --dev
cargo add snafu --dev
```

In the `lib.rs`, we'll create the binding to our Spade module:

```rust
// file: src/lib.rs
use marlin::spade::prelude::*;

#[spade(src = "src/main.spade", name = "main")]
pub struct Main;
```

This tells Marlin that the `struct Main` should be linked to the `main` entity
in our Spade file.

Finally, we'll want to actually write the code that drives our hardware in `simple_test.rs`:

```shell
mkdir tests
vi tests/simple_test.rs
```

```rust
// file: tests/simple_test.rs
use tutorial_project::Main;
use marlin::spade::prelude::*;
use snafu::Whatever;

#[test]
#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = SpadeRuntime::new(
        SpadeRuntimeOptions::default_logging() // configuration 
    )?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
```

Finally, we can simply use `cargo test` to drive our design!

> [!WARNING]
> By default, `SpadeRuntime` invokes `swim build`, which is not thread-safe. If
> you have multiple tests, run `swim build` beforehand and then run `cargo test`
> after disabling the automatic `swim build` via
> `SpadeRuntimeOptions::call_swim_build`.

Note that, unlike the Verilog project tutorial, you don't need to add another
directory to your `.gitignore`, if you have one, because the `SpadeRuntime`
reuses the existing `build/` directory managed by Swim. Thus, you should add
that to your `.gitignore` instead. `swim init` should do that automatically,
though.
