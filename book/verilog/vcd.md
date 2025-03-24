# Waveform Tracing

You can open a VCD for a Verilated model using the `.open_vcd` function, which takes in anything that can turn into a `Path`.
The `.dump` and other functions are bridged directly to the Verilator functions and, as such, will behave as you expect (but through a safe Rust API).

The VCD is automatically closed and deallocated when out of scope.
Lifetimes enforce that you cannot use the VCD past the scope of the runtime whence the model you created the VCD came.

Until <https://github.com/verilator/verilator/issues/5813> gets fixed, `.open_vcd` will panic if you call it more than once.

You can consult the reference documentation for VCDs [here](https://docs.rs/marlin/latest/marlin/verilator/vcd/struct.VCD.html).
