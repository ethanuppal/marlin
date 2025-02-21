# Marlin Handbook 🐟

[![CI Badge](https://github.com/ethanuppal/marlin/actions/workflows/ci.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/ci.yaml)
[![Code Style Badge](https://github.com/ethanuppal/marlin/actions/workflows/lint.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/lint.yaml)
[![cargo-deny badge](https://github.com/ethanuppal/marlin/actions/workflows/cargo-deny.yaml/badge.svg)](https://github.com/ethanuppal/marlin/blob/main/.github/workflows/cargo-deny.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/marlin)](https://crates.io/crates/marlin)
[![docs.rs](https://img.shields.io/docsrs/marlin)](https://docs.rs/marlin/latest/marlin)
[![Crates.io License](https://img.shields.io/crates/l/marlin)](./LICENSE)

Marlin is a hardware testing framework that _just works_.
It comes as a normal Rust crate, so you don't need build scripts or preprocessing commands.
That means:

- **Unlike [Verilator][verilator] harnesses**, then, you don't need to run a command
  to generate header and C++ files that you then connect to/run via a Makefile,
  Ninja, or other build system.
- **Unlike [cocotb] tests**, you don't have to use Makefiles or write Python driver
  code, use fancy decorators on the tests, and lose LSP information for hardware
  model ports.

Moreover, Marlin core is thread-safe, meaning you can use `cargo test`/`#[test]`
for your tests! It integrates perfectly and completely noninvasively into the 
existing Rust ecosystem.

## Features

Marlin comes with prebuilt integration for (System)Verilog and [Spade][spade], and offers:

- A declarative API for writing tests in plain Rust, treating the hardware
  models as normal `struct`s with fields and member functions ([learn more](./verilog/quickstart.md)).
- A safe procedural API for dynamically constructing bindings to Verilog and
  interacting with opaque hardware models at runtime ([learn more](./verilog/dynamic.md)).
- A library for any hardware description language (HDL) that compiles to Verilog
  to get all of the above.

You can also [call Rust functions](./verilog/dpi.md) from Verilog.

## Future Work

Planned features include:

- Ports wider than 64 bits ([#7](https://github.com/ethanuppal/marlin/issues/7))
- Static linking + build scripts as an option ([#23](https://github.com/ethanuppal/marlin/issues/23))
- Supporting the [Calyx intermediate language][calyx] ([#8](https://github.com/ethanuppal/marlin/issues/8))
- Supporting the [Veryl HDL][veryl] ([#8](https://github.com/ethanuppal/marlin/issues/6))

[verilator]: https://www.veripool.org/verilator/
[cocotb]: https://www.cocotb.org
[spade]: https://spade-lang.org
[veryl]: https://veryl-lang.org
[calyx]: https://calyxir.org
