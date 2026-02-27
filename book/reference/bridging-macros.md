# Bridging Macros

Marlin uses "bridging" macros to define bindings between Rust `struct`s and hardware modules:

- [`#[verilog]`](https://docs.rs/marlin/latest/marlin/verilog/prelude/attr.verilog.html)
- [`#[spade]`](https://docs.rs/marlin/latest/marlin/spade/prelude/attr.spade.html)
- [`#[veryl]`](https://docs.rs/marlin/latest/marlin/veryl/prelude/attr.veryl.html)

Under the Verilator backend, these take a common interface:

- `name = "<name>"`: The name of the module.
- `src = "<file>"`: The file where the module is defined relative to the manifest directory.

See [the relevant internal documentation](../../internal/how-it-works.md) for technical explanation.
