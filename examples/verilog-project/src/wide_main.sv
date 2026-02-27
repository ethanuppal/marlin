module wide_main(
    input[64:0] wide_input,
    output[64:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main2(
    input[127:0] wide_input,
    output[127:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main3(
    input[255:0] wide_input,
    output[255:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main4(
    input[255:128] wide_input,
    output[255:128] wide_output
);
    assign wide_output = wide_input;
endmodule
