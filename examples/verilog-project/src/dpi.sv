import "DPI-C" function void three(output int unsigned out);
import "DPI-C" function void check_three(input int in);
import "DPI-C" function void Bool(output bit b);

module dpi_main(output logic out);
    int a = 0;
    logic b = 0;

    initial begin
        three(a);
        check_three(a);
        $display("%d", a);

        Bool(b);
        $display("%d", b);
        out = b;
    end
    
endmodule
