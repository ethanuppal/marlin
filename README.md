# marlin 🐟

[![CI Badge](https://github.com/ethanuppal/marlin/actions/workflows/ci.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/ci.yaml)
[![Code Style Badge](https://github.com/ethanuppal/marlin/actions/workflows/lint.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/lint.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/marlin)](https://crates.io/crates/marlin)
[![docs.rs](https://img.shields.io/docsrs/marlin)](https://docs.rs/marlin/latest/marlin)
[![Crates.io License](https://img.shields.io/crates/l/marlin)](./LICENSE)
[![Supported Spade version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2Fethanuppal%2Fmarlin%2Fraw%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=workspace.dependencies.spade-parser.version&label=Spade)](https://github.com/ethanuppal/marlin/blob/main/language-support/spade)
[![Supported Veryl version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2Fethanuppal%2Fmarlin%2Fraw%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=workspace.dependencies.veryl-parser.version&label=Veryl)](https://github.com/ethanuppal/marlin/blob/main/language-support/veryl)
[![Matrix](https://img.shields.io/matrix/marlin-project%3Amatrix.org?label=Matrix)](https://matrix.to/#/#marlin-project:matrix.org)

**[Read the documentation](https://ethanuppal.com/marlin)** | **[Read the API reference](https://docs.rs/marlin/latest/marlin)**

Marlin is a really powerful library (and API) that lets you "import" hardware
modules into Rust (or Rust functions into hardware modules!). 

No precompilation step and manual updates with `verilator` harnesses; no 
Makefiles and quirky decorators with `cocotb` testbenches. You're writing a regular Rust crate here.

Add this library to your `Cargo.toml` like any other library. Use hardware
modules as `struct`s like any other Rust `struct`. Hook them up to `tokio` or
`serde` even. `cargo test` as hard as you want.

Marlin works out of the box on macOS and Linux (verified under continuous integration).

<table>
<tr>
<td> <code>tests/demo.rs</code> </td> <td> shell </td> <td> <code>tests/u8_counter.sv</code> </td>
</tr>
<tr>
<td>

```rs
use marlin::verilog::prelude::*;
use marlin_test::prelude::*;

#[verilog(src = "tests/u8_counter.sv", name = "u8_counter")]
struct U8Counter;

#[marlin_verilog_test]
#[vcd("counter_resets.vcd")]
fn counter_resets<'a>(mut counter: Seq<'a, U8Counter<'a>>) {
    counter.reset = 1;
    counter.tick();
    counter.reset = 0;
    assert_eq!(counter.value, 0);
}
```

</td>
<td>

```
$ cargo test
   Compiling demo v0.1.0 (/project)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running tests/demo.rs (target/debug/deps/demo-9154602d5ae1a421)

running 1 test
   Compiling u8_counter#8724665540442200216 (/project/tests/u8_counter.sv)
    Finished `verilator-O0` profile [unoptimized] target in 1.17s
test counter_resets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
```

</td>
<td>

```sv
  module u8_counter(
    input logic clk,
    input logic reset,
    input logic enable,
    output reg[7:0] value
);
    always_ff @(posedge clk) begin
        if (reset)
            value <= 0;
        else
            value <= enable ? (value + 1) : value;
    end
endmodule
```

</td>
</tr>
</table>



> Example using [`marlin-test`](https://crates.io/crates/marlin-test), a set of macros replacing `#[test]`.

## Motivation

Why does hardware testing suck? Consider the ways we have to test
(System)Verilog:

- **Test natively**: Verilog is already a terrible enough language, and writing
  tests *in* Verilog is really annoying.
- **Use Verilator harnesses**: You have to first run Verilator to get the right
  headers, recompile manually every time, deal with raw pointers and C++, etc.
- **Use cocotb**: You have to use Makefiles or write your own Python runner, 
  performance isn't the greatest, you get no LSP support for ports, etc.

The problem gets worse with custom HDLs, so they've come up with some creative
solutions:

- [Calyx](https://calyxir.org): the canonical way of testing Calyx code is to
  read from JSON files representing byte arrays and write to JSON files
  representing byte arrays.
- [Spade](https://spade-lang.org): `verilator` integration involves [absurd
  macro magic](https://docs.spade-lang.org/simulation.html#verilator) and [using
  `cocotb`](https://docs.spade-lang.org/simulation.html#cocotb) requires putting the design-under-test in a code comment.
- [Veryl](https://veryl-lang.org): you literally [write inline Verilog or Python](https://doc.veryl-lang.org/book/05_language_reference/13_integrated_test.html). Yes, inside Veryl code.

Still, a lot of these are less than optimal.

## Features

- Minimal overhead over directly using `verilator`
- Works completely drop-in in your existing projects
- Safe VCD tracing support
- Declarative API for usability + Dynamic API for programmability
- DPI support in Rust: call Rust functions from (System)Verilog
- Integration with modern HDLs
- Rust. Did I say Rust?

## Requirements

- [Rust](https://rustup.rs), 2021 edition
- [`verilator`](https://verilator.org/guide/latest/install.html), 5.025 or later
  - Earlier versions may work but are not tested
- [GNU Make](https://www.gnu.org/software/make/)
- A C++ compiler that `verilator` can find; may need to [support at least C++14](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2014/n4296.pdf).

## Install

Marlin is on [crates.io], so just use `cargo add --dev marlin` to add Marlin as a
dependency for your tests (`dev-dependencies`).

## How it works

I'll write more on this once I get further in the development process.
The TLDR is procedural macros + `dlopen`.
There are some stub pages in the work-in-progress internal section of the documentation, such as [this page](https://www.ethanuppal.com/marlin/internal/how-it-works.html).

### Hardware simulation tools are slow! How does Marlin deal with that?

Simulation tools take an _absurd_ amount of time to run.
For example, when you use Marlin in a Spade project, it calls out to:

- `swim build`, which recompiles the entire Spade compiler from source
- `verilator`, which compiles and links C++ code

Marlin automatically runs them with all the right flags and arguments
--- and it caches and only invokes them when needed.

## Related

- [verilated-rs](https://github.com/djg/verilated-rs) is a super cool library
  that uses a build script to statically link in verilated bindings, but is
  unmaintained for years as of writing this.

## License & Legal

[![cargo-deny badge](https://github.com/ethanuppal/marlin/actions/workflows/cargo-deny.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/cargo-deny.yaml)

Marlin is licensed under the Mozilla Public License 2.0. This license is
similar to the Lesser GNU Public License, except that the copyleft applies only
to the source code of this library, not any library that uses it. That means you
can statically or dynamically link with unfree code (see
<https://www.mozilla.org/en-US/MPL/2.0/FAQ/#virality>).

I use [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) (see the
[`deny.toml`](./deny.toml)) to ensure no licensing violations occur. I also
check this on CI to prevent merging any new dependencies or dependency updates
that introduce incompatible licenses.

### Large Language Models

I do not permit any contributions containing output, in part or in whole, from large language models (LLMs) or other probabilistic models.

### Verilator

Verilator is licensed under the Lesser GNU General Public License 3.0. However,
Marlin will `dlopen` Verilated code, which is permitted via this clause:

> 1) Use a suitable shared library mechanism for linking with the
>   Library.  A suitable mechanism is one that (a) uses at run time
>   a copy of the Library already present on the user's computer
>   system, and (b) will operate properly with a modified version
>   of the Library that is interface-compatible with the Linked
>   Version.

Through [`VerilatorRuntimeOptions::verilator_executable`](https://docs.rs/marlin/latest/marlin/verilator/struct.VerilatorRuntimeOptions.html#structfield.verilator_executable),
you can specify your own interface-compatible Verilator wrapper, enabling (b).

[crates.io]: https://crates.io/crates/marlin

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=ethanuppal/marlin&type=date&legend=top-left)](https://www.star-history.com/#ethanuppal/marlin&type=date&legend=top-left)
