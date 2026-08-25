# Adding a new HDL

Adding support for a new HDL to Marlin involves:

- Choosing a runtime (right now, only Verilator is supported)
- Making a procedural macro that lowers to an interface understood by the runtime (right now, only Verilog is supported)

Typically, you'll want to create a wrapper for the runtime, e.g., `SpadeRuntime` wraps `VerilatorRuntime`.
In your procedural macro, you'll typically call `build_verilated_struct` to handle most of the work in defining the interface.
