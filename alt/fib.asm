# reset the R31 to 65536
    ldi R0, R31, 65535
    
# load a 1 in R30 to subtract the stackp 
    ldi R0, R30, 1
#

    ldi R0, R0, 0
    ldi R0, R1, 1
LOOP
    call FIB
    jmp LOOP



FIB
    add R0, R1, R2
    mov R1, R0, 0
    mov R2, R1, 0
    ret
END
