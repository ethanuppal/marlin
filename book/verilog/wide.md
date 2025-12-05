# Wide Ports

Marlin supports wide (larger than 64 bits) ports in both static and dynamic bindings.
The Verilator interface is exposed through a safe Rust wrapper in either case.

## Static

If a wide value is declared in Verilog with `[MSB:LSB]`, then `LENGTH` will be `compute_wdata_length_from_width_not_msb(MSB + 1)`.

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
    WDataOutP(Vec<types::WData>),
}
```
