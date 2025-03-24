import "DPI-C" function void set_unsigned_int_out(output int unsigned out);
import "DPI-C" function void check_unsigned_int_out(input int unsigned in);
import "DPI-C" function void set_bool_out(output bit b);

module dpi_main(output logic[31:0] int_out, output logic bool_out);
    int a = 0;
    
    logic b = 0;

    initial begin
        set_unsigned_int_out(a);
        check_unsigned_int_out(a);
        $display("%d", a);

        set_bool_out(b);
        $display("%d", b);

        int_out = a;
        bool_out = b;
    end
endmodule
