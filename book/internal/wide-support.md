# Wide Port Support

[A gist with the relevant Verilator header file code](https://gist.github.com/ethanuppal/aaf0916080343d8244a48f982912e6fd).

The following signature
```sv
    input single_input,
    input[7:0] small_input,
    input[63:0] medium_input,
    input[127:0] big_input,
    input[127:126] big2_input,
    output single_output,
    output[7:0] small_output,
    output[63:0] medium_output,
    output[127:0] big_output,
    output[127:126] big2_output
```
generates this code
```cpp
    // PORTS
    // The application code writes and reads these signals to
    // propagate new values into/out from the Verilated model.
    VL_IN8(&single_input,0,0);
    VL_IN8(&small_input,7,0);
    VL_IN8(&big2_input,127,126);
    VL_OUT8(&single_output,0,0);
    VL_OUT8(&small_output,7,0);
    VL_OUT8(&big2_output,127,126);
    VL_INW(&big_input,127,0,4);
    VL_OUTW(&big_output,127,0,4);
    VL_IN64(&medium_input,63,0);
    VL_OUT64(&medium_output,63,0);
```
for the public class members of the module.
