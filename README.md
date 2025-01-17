# dumbname

![CI Badge](https://github.com/ethanuppal/dumbname/actions/workflows/ci.yaml/badge.svg)
![Code Style Badge](https://github.com/ethanuppal/dumbname/actions/workflows/lint.yaml/badge.svg)

dumbname is a really powerful library (and API) that lets you "import" hardware
modules into Rust. 

No precompilation step and manual updates with `verilator` harnesses; no 
Makefiles, magical comments, and quirky decorators with `cocotb`.
testbenches. You're writing a regular Rust crate here.

Add this library to your `Cargo.toml` like any other library. Use hardware
modules as `struct`s like any other Rust `struct`. Hook them up to `tokio` or
`serde` even.

## 🚀 Showcase

![Early example of using this with Spade](./assets/demo-alpha.png)

## ⚡️ Requirements

- [Rust](https://rustup.rs)
- [`verilator`](https://verilator.org/guide/latest/install.html)

## 📦 Install

dumbname is currently in development.
You can currently install the crates via `git` specifications.

## ✨ Usage

I'll write more documentation once I get further in the development process.

## 💡 How it works

I'll write more on this once I get further in the development process.
The TLDR is procedural macros + `dlopen`.
