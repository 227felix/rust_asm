# reset the R31 to 65536
    ldi R0, R31, 65535
    
# load a 1 in R30 to subtract the stackp 
    ldi R0, R30, 1

    ldi R0, R0, 2
    call ADD
    ldi R0, R1, 10
    call ADD
    jmp END



ADD
    add R0, R0, R0
    ret
END