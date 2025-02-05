# Testing a Spade project

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a Spade project and test our code with
dumbname. You can find the full source code for this tutorial [here](../examples/spade-project/). We won't touch on the advanced aspects or features; the goal is just to provide a simple overfiew sufficient to get started.

I'll be assuming you've read the [tutorial on testing Verilog projects](./testing_verilog.md); if not, read that first and come back.

If you don't already have Spade installed, [make sure to do that](https://docs.spade-lang.org/installation.html).
You can either integrate dumbname into an existing [Swim](https://docs.spade-lang.org/swim/index.html) project at no effort or (in this case) make a new Swim project from scratch with an eye to dumbname.

```shell
swim init tutorial-project
cd tutorial-project
git init # optional, if using git
```

Here's what our project will look like at the end:
```
.
├── swim.lock
├── swim.toml
├── Cargo.toml
├── src
│   └── main.spade
└── test
    └── simple_test.rs
```

In `main.spade` we'll write some simple Spade code:

```rust
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

Then, we'll make a new crate to use dumbname:

```shell
mkdir test
vi Cargo.toml
vi test/main.rs
```

In the `Cargo.toml`, we'll indicate there's one test called `simple_test.rs` and add the `spade` dependency:

```toml
# file: Cargo.toml
[package]
name = "tutorial-project"

[[bin]]
name = "simple_test"
path = "test/simple_test.rs"

[dependencies]
# other dependencies...
spade = { git = "https://github.com/ethanuppal/dumbname" }
snafu = "0.8.5" # optional, whatever version
colog = "1.3.0" # optional, whatever version
```

Our testing code will be similar to the Verilog code:

```rust
// file: test/simple_test.rs
use snafu::Whatever;
use spade::{spade, SpadeRuntime, SpadeRuntimeOptions};

#[spade(src = "src/main.spade", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    // the second argument `true` says we want debug logging with the log crate
    let mut runtime = SpadeRuntime::new(SpadeRuntimeOptions::default(), true)?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
```

A `cargo run` from the project root lets us test our Spade!

Note that, unlike the Verilog project tutorial, you don't need to add another
directory to your `.gitignore`, if you have one, because the `SpadeRuntime`
reuses the existing `build/` directory managed by Swim. Thus, you should add
that to your `.gitignore` instead.
