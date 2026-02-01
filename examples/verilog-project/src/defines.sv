module defines_main(
    input[31:0] data_in,
    output[31:0] data_out
);
`ifdef INVERT_OUTPUT
    assign data_out = ~data_in;
`else
    assign data_out = data_in;
`endif
endmodule
