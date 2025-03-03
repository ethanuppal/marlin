import "DPI-C" function void three(output int out);

module dpi_main(output logic[31:0] out);
    int a = 0;
    initial begin
        three(a);
        $display("%d", a);
        out = a;
    end
endmodule
