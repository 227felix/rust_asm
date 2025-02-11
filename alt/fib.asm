    ldi R0, R5, 1
    ldi R0, R3, 0
    ldi R0, R1, 0
    ldi R0, R2, 1
    ldi R0, R4, 0
    ldi R0, R6, 0
LOOP
    stw R1, R3, 0
    add R3, R5, R3
    add R1, R2, R1
    mov R1, R6, 0
    mov R2, R1, 0
    mov R6, R2, 0
    beq R0, R0, LOOP
    call FUNCT1
