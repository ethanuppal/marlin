# Wide Ports

Marlin supports wide (larger than 64 bits) ports in both static and dynamic bindings.
The Verilator interface is exposed through a safe Rust wrapper in either case.

## Static


```rs
pub struct WideIn<const LOW: usize, const HIGH: usize, const LENGTH: usize> {
    /* private members */
}

pub struct WideOut<const LOW: usize, const HIGH: usize, const LENGTH: usize> {
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
