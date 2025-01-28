import "DPI-C" function void three(output int out);

module main();
    int a = 0;
    initial begin
        three(a);
        $display("%x", a);
    end
endmodule
