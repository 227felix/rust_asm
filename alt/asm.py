import sys


NOP = "00000000000000000000000000000000"


def check_line_type(line):
    
    if line.isupper():
        return 'label'
    if line == '':
        return 'empty'
    return 'instruction'

def build_i(r1, r2, imm):

    instr = ""
    instr += bin(int(r1))[2:].zfill(5)
    instr += bin(int(r2))[2:].zfill(5)
    instr += imm
    return instr

def build_r(r1, r2, r3):

    instr = ""
    instr += bin(int(r1))[2:].zfill(5)
    instr += bin(int(r2))[2:].zfill(5)
    instr += bin(int(r3))[2:].zfill(5)
    instr += "00000000000"
    return instr

def build_j(label, labels):
    instr = ""
    instr += bin(labels.get(label))[2:].zfill(26)
    print(instr)
    return instr


def handle_macro(macro, split, g, line_count, reg_cooldown, labels):
    
    # if macro == "loop":
    #     # loop <label>
    #     instr = "001010" + build_j(split[1], labels)
    #     line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
    #     instr = "000111" + build_i("31", "31", "1111111111111111")
    #     line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
    # elif macro == "endloop":
    #     # endloop <label>
    #     instr = "001010" + build_j(split[1], labels)
    #     line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
    # elif macro == "call":
    #     # call <label>
    #     instr = "001010" + build_j(split[1], labels)
    #     line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
    # elif macro == "ret":
    #     # ret
    #     instr = "001010" + build_j("31", labels)
    #     line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
    # else:
    #     print("Unknown macro: " + macro)
    #     exit()
    # return line_count

def main():
    if len(sys.argv) < 2:
        print("Usage: python asm.py <asm_file>")
        exit()

    asm_file = sys.argv[1]
    f = open(asm_file, "r")
    input_file_name = asm_file.split(".")[0]
    output_file_name = input_file_name + ".dat"
    g = open(output_file_name, "w")



    opcodes = {
        "nop" : "000000",
        "add" : "000001",
        "sub" : "000010",
        "stw" : "001100",
        "ldi" : "001101",
        "mov" : "001110",
        "jmp" : "001010",
        "beq" : "000111",
        "bneq" : "001000",
    }

    formats = {
        "nop" : "N",
        "add" : "R",
        "sub" : "R",
        "stw" : "I",
        "ldi" : "I",
        "mov" : "I",
        "beq" : "I",
        "bneq" : "I",
        "jmp" : "J",
    }

    num_nops = {
        "nop" : 0,
        "add" : 4,
        "sub" : 4,
        "stw" : 4,
        "ldi" : 4,
        "mov" : 4,
        "jmp" : 4,
        "beq" : 4,
        "bneq" : 4,
    }
    
    macros = [
        "loop",
        "endloop",
        "call",
        "ret"]
        
    
    
    line_count = 0 # keep track of the line number for jumps
    labels = {}
    
    # do one pass to get all labels
    for (index, line) in enumerate(f.readlines()):
        line = line.strip()
        line_type = check_line_type(line)
        if line_type == 'label':
            labels.update({line: line_count})
        else:
            line_count += 1
    f.seek(0) # go back to the beginning of the file
    line_count = 0 # reset the line count
    print(labels)
        
    
    
    # keep track of the amount of nops that have to be inserted, when a register is beeing hit (decrease every instr)
    reg_cooldown = {}
    for i in range(32):
        reg_cooldown.update({i: 0})
        
    for (index, line) in enumerate(f.readlines()):
        line = line.strip()

        line_type = check_line_type(line)
        
        if line_type == 'empty':
            print("Empty line: " + str(index + 1))
            continue
        if line_type == 'label':
            labels.update({line: line_count})
            continue
        
        
        
    
        

        split = line.split(" ")
        new_split = []
        for sp in split:
            new_item = sp.replace(",", "").replace("\n", "")
            new_split.append(new_item)

        opc = split[0]
        
        
        # check if the line is a macro
        if opc in macros:
            line_count = handle_macro(opc, new_split, g, line_count, reg_cooldown, labels)
            continue
        
        
        instr = opcodes.get(opc)
            

        format = formats.get(opc)
        regs = []
        r1, r2, r3 = 0, 0, 0
        if format == 'I' or format == 'R':
            
            r1 = new_split[1]
            if r1[0] == 'R':
                r1 = r1[1:]
                regs.append(r1)
            else:
                print("Error: First operand is not a register")
                exit()
            r2 = new_split[2]
            if r2[0] == 'R':
                r2 = r2[1:]
                regs.append(r2)
            else:
                print("Error: Second operand is not a register")
                exit()
                
            if format == 'I':
                # check if there is a num or a label and get the relative address to the label
                if new_split[3].isupper():
                    imm = labels.get(new_split[3]) - line_count
                    imm = dec_to_two_complement(imm)
                else:
                    imm = new_split[3]
                    imm = dec_to_two_complement(int(imm))
                # parse the immediate value that could be a negative integer in base 10 and put it in 2's complement
                
                
                
            if format == 'R':
                r3 = new_split[3]
                if r3[0] == 'R':
                    r3 = r3[1:] 
                    regs.append(r3)
                else:
                    print("Error: Third operand is not a register")
                    exit()
                    
        for reg in regs: # FIXME es müssen nicht unbedingt alle Register cooldown haben (bspw. Quellregister werden ja nicht verändert)
            for i in range(reg_cooldown.get(int(reg))): # insert as many nops as needed
                for label in labels: # FIXME: Performance 🤯
                    if labels.get(label) > line_count:
                        labels.update({label: labels.get(label) + 1})
                        print(labels)
                line_count =  write_instr(g, NOP, line_count, reg_cooldown, labels)
                
        match format:
            case 'I':
                instr += build_i(r1, r2, imm)
            case 'R':
                instr += build_r(r1, r2, r3)
            case 'J':
                instr += build_j(new_split[1], labels)
            case 'N':
                instr += NOP

        line_count = write_instr(g, instr, line_count, reg_cooldown, labels)
        for (index, reg) in enumerate(regs):
            if index == 0:
                continue
            reg_cooldown.update({int(reg): num_nops.get(opc)})
    f.close()
    g.close()
    
    
    
def write_instr(g, instr, line_count, reg_cooldowns, labels):
    
    g.write(str(line_count) + " " + instr)
    g.write("\n")
    line_count += 1
    for reg in reg_cooldowns: # decrease the cooldown for all registers as an instruction was written
        reg_cooldowns.update({reg: reg_cooldowns.get(reg) - 1})
        
        
    return line_count

def dec_to_two_complement(dec):
    if dec < 0:
        return bin(dec & 0xFFFF)[2:]
    return bin(dec)[2:].zfill(16)

if __name__ == "__main__":
    main()


# macros
