module very_wide_registered(
    input clk,
    input[199:0] very_wide_input,
    output reg [199:0] very_wide_output
);
    always @(posedge clk) begin
        very_wide_output <= very_wide_input;
    end
endmodule
