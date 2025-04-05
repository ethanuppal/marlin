# Model Traits

There are two main traits for Verilated models:

- `AsVerilatedModel`:

    This trait is implemented for types derived using `#[verilog]` or `#[spade]`, etc.
    It should not be manually derived.
    You can use the type-level functions provided by this trait to get information about the model.

- `AsDynamicVerilatedModel`:
    
    This trait is implemented for all models, derived and dynamic.
    It provides a safe runtime API for accessing ports by strings (instead of using the actual `struct` fields).
    For derived models, you typically won't need to use it because you'll just be able to set and read fields directly.
