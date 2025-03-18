import "DPI-C" function void set_out(output int out);

module dpi_main(output logic[31:0] out);
    int a = 0;
    initial begin
        set_out(a);
        $display("%d", a);
        out = a;
    end
endmodule
