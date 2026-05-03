# Waveform Tracing

## Overview

(Bad) example [here](https://github.com/ethanuppal/marlin/blob/main/examples/verilog-project/tests/tracing.rs).

You can open a VCD or FST trace file for a Verilated model using the `.open_trace` function, which takes in anything that can turn into a `Path`.
The `.dump` and other functions are bridged directly to the Verilator functions and, as such, will behave as you expect (but through a safe Rust API).

The trace is automatically closed and deallocated when out of scope.
Lifetimes enforce that you cannot use the trace past the scope of the runtime whence the model you created the VCD came.

Until <https://github.com/verilator/verilator/issues/5813> gets fixed, `.open_vcd` will panic if you call it more than once.

You can consult the reference documentation for traces [here](https://docs.rs/marlin/latest/marlin/verilator/tracing/struct.Trace.html).

## Tips

You might find yourself wanting to write a function on your model (let's say you declared it as `struct Top`) to simulate a clock cycle.
You will need to remember to update the trace, just like in Verilator.
For instance:
```rs
impl Top<'_> {
    fn tick(&mut self, trace: &mut Trace<'_>, timestamp: &mut u64) {
        self.clk = 0;
        self.eval();
        *timestamp += 1;
        trace.dump(*timestamp);
        self.clk = 1;
        self.eval();
        *timestamp += 1;
        trace.dump(*timestamp);
    }
}
```

You could also wrap the `Trace` in another `struct`:

```rs
pub struct GoodTrace<'a> {
    inner: Trace<'a>,
    timestamp: u64,
}

impl<'a> From<Trace<'a>> for GoodTrace<'a> {
    fn from(inner: Trace<'a>) -> Self {
        Self {
            inner,
            timestamp: 0,
        }
    }
}
```

See the `Seq<'a, _>` type in the [`marlin-test`](https://docs.rs/marlin-test) crate for a complete example of this construction.
