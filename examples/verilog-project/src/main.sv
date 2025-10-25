module main(
    input[31:0] medium_input,
    output[31:0] medium_output
);
    assign medium_output = medium_input;
endmodule

module wide_main(
    input[64:0] wide_input,
    output[64:0] wide_output
);
    assign wide_output = wide_input;
endmodule
