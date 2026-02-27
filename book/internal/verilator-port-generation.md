# Verilator Port Generation

This page details how Verilator translates Verilog interfaces into C++.

[A gist with the relevant Verilator header file code](https://gist.github.com/ethanuppal/aaf0916080343d8244a48f982912e6fd).

## Explanation

The length is computed (via `"most significant bit index" - "least significant bit index + 1"`).
If this length is `<= 64`, the appropriate primitive integer type is used.
Otherwise, a `VlWide` is used, which is an array of length `ceil(length / (sizeof(WData) * 8))`, _i.e._, the minimum number of `WData`s needed to represent the value.

## Example

The following signature
```sv
    input single_input,
    input[7:0] small_input,
    input[63:0] medium_input,
    input[127:0] big_input,
    input[256:128] big2_input,
    input[127:126] weird_input,
    output single_output,
    output[7:0] small_output,
    output[63:0] medium_output,
    output[127:0] big_output,
    output[256:128] big2_output,
    output[127:126] weird_output
```

generates this code (on `Verilator 5.042 2025-11-02 rev UNKNOWN.REV`)
```cpp
    // PORTS
    // The application code writes and reads these signals to
    // propagate new values into/out from the Verilated model.
    VL_IN8(&single_input,0,0);
    VL_IN8(&small_input,7,0);
    VL_IN8(&weird_input,127,126);
    VL_OUT8(&single_output,0,0);
    VL_OUT8(&small_output,7,0);
    VL_OUT8(&weird_output,127,126);
    VL_INW(&big_input,127,0,4);
    VL_INW(&big2_input,256,128,5);
    VL_OUTW(&big_output,127,0,4);
    VL_OUTW(&big2_output,256,128,5);
    VL_IN64(&medium_input,63,0);
    VL_OUT64(&medium_output,63,0);
```
among the public class members of the design.

The additional macro parameters to `VL_INW`/`VL_OUTW` are the number of "words" in (_i.e._, the length of) the `VlWide` array

## In Marlin

Wide values are read and written over the FFI interface via pointers to avoid ABI issues with passing around large arrays (`VlWide`).
All other values are copied.
See `build_library.rs` in the `verilator` crate for details.

The functions `compute_wdata_word_count_from_width_not_msb` and `compute_approx_width_from_wdata_word_count` help capture the wide port translation.
