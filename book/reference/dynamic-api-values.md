# Dynamic API Values

`VerilatorValue` is a dynamic version of a Verilator value: an `enum` representing all possible `marlin_verilator::types`.
A lot of the values implement `From` for an appropriate type.
For instance, a `VerilatorValue::WDataInP` can be created by `into()`ing on a `&[types::WData; LENGTH]`.
This often lets you write tests with `assert_eq!` without needing to use the `enum` at all, just an `.into()` on one argument.
Consult the documentation for all the implementations and details.
