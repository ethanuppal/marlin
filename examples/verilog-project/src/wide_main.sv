module wide_main(
    input logic[64:0] wide_input,
    output logic[64:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main2(
    input logic[127:0] wide_input,
    output logic[127:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main3(
    input logic[255:0] wide_input,
    output logic[255:0] wide_output
);
    assign wide_output = wide_input;
endmodule

module wide_main4(
    input logic[255:128] wide_input,
    output logic[255:128] wide_output
);
    assign wide_output = wide_input;
endmodule
