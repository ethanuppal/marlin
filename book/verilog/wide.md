# Wide Ports

Marlin supports wide (larger than 64 bits) ports in both static and dynamic bindings.
The Verilator interface is exposed through a safe Rust wrapper in either case.

There is currently some overhead in using wide ports because the Rust side is entirely (and safely) isolated from the C++.
This overhead involves a `memcpy` of the entire value when reading or writing and, if using the dynamic API, an additional allocation (from `Box::from`).

## Static

If a wide value is declared in Verilog with `[MSB:LSB]`, then `LENGTH` will be `compute_wdata_word_count_from_width_not_msb(MSB + 1 - LSB)`.

```rs
pub struct WideIn<const LENGTH: usize> {
    /* private members */
}

pub struct WideOut<const LENGTH: usize> {
    /* private members */
}
```

## Dynamic

```rs
pub enum VerilatorValue<'a> {
    ...,
    WDataInP(&'a [types::WData]),
    WDataOutP(Box<types::WData>),
}
```
