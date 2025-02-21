# Getting Started

## Install Verilator

The current integrations for Verilog and [Spade][spade] use a [Verilator][verilator] backend, which you need to install.
For example, on macOS:
```shell
brew install verilator
```
and on Ubuntu:
```shell
apt-get install verilator
```

Check this [list of packages](https://repology.org/project/verilator/versions) to find one for your operating system and view the [official installation instructions](https://veripool.org/guide/latest/install.html) if you need more help.

## Install Marlin

First, make sure you've installed a [Rust toolchain](https://rustup.rs), which
should come packaged with [`cargo`](https://doc.rust-lang.org/cargo/). Then,
simply run:

```shell
cargo install marlin
```

You're done! Now, it's time to [get started testing some Verilog](./verilog/quickstart.md).

[verilator]: https://www.veripool.org/verilator/
[cocotb]: https://www.cocotb.org
[spade]: https://spade-lang.org
[veryl]: https://veryl-lang.org
[calyx]: https://calyxir.org
