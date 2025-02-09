# Testing a Spade project

> [!NOTE]
> This tutorial is aimed at Unix-like systems like macOS, Linux, and WSL.

In this tutorial, we'll setup a Spade project and test our code with
Marlin. You can find the full source code for this tutorial [here](../examples/spade-project/). We won't touch on the advanced aspects or features; the goal is just to provide a simple overfiew sufficient to get started.

I'll be assuming you've read the [tutorial on testing Verilog projects](./testing_verilog.md); if not, read that first and come back.

If you don't already have Spade installed, [make sure to do that](https://docs.spade-lang.org/installation.html).
You can either integrate Marlin into an existing [Swim](https://docs.spade-lang.org/swim/index.html) project at no effort or (in this case) make a new Swim project from scratch with an eye to Marlin.

First, install `swim-marlin` from [crates.io](crates.io):

```
cargo install swim-marlin
```

Then, setup the project:

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

Then, we'll make a new crate to use Marlin:

```shell
swim marlin init -d test
```

This creates by default a `test.rs` test in the `test/` directory. The `-d`
option is optional; without it, `swim-marlin` will figure it out from your
`swim.toml` or default to `tests/`.

In the `Cargo.toml`, we'll rename the test `simple_test`:

```diff
 # file: Cargo.toml
 [[bin]]
-name = "test"
+name = "simple_test"
-path = "test/test.rs"
+path = "test/simple_test.rs"
```

Let's also rename the file to reflect this:

```shell
mv test/test.rs test/simple_test.rs
```

Our testing code will be similar to the Verilog code:

```rust
// file: test/simple_test.rs
use snafu::Whatever;
use marlin::spade::prelude::*;

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

A `swim marlin test` from the project root lets us test our Spade!

Note that, unlike the Verilog project tutorial, you don't need to add another
directory to your `.gitignore`, if you have one, because the `SpadeRuntime`
reuses the existing `build/` directory managed by Swim. Thus, you should add
that to your `.gitignore` instead. `swim init` should do that automatically,
though.
