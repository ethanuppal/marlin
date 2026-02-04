module very_wide_1024(
    input[1023:0] very_wide_input,
    output[1023:0] very_wide_output
);
    assign very_wide_output = very_wide_input;
endmodule
