use clap::Parser;
use color_backtrace::install;
use core::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

fn main() {
    install();

    // parse a .asm file and optional output path from the command line with clap
    let args = Args::parse();
    let asm_file = args.asm_file;
    // let asm_file = if args.asm_file.is_empty() {
    //     "alt/fib.asm".to_string()
    // } else {
    //     args.asm_file
    // };
    //println!("Parsing file: {}", asm_file);

    let mut opcodes_handler = IsaParser::new(args.output_path);

    let asm = std::fs::read_to_string(asm_file).unwrap();
    let lines = asm.lines().map(|l| l.trim()).collect::<Vec<_>>();

    let _ = lines
        .iter()
        .map(|l| {
            let instr = opcodes_handler.handle_line(l);
            instr
        })
        .collect::<Vec<_>>();
    opcodes_handler.write();
}

#[derive(Debug)]
enum LineType {
    Instr,
    Label,
    Macro,
}

#[derive(Debug, Clone)]
struct InstrBlueprint {
    opcode: String,
    instr_format: InstrFormat,
    bin_rep: String,
    num_nops: u32,
}

impl InstrBlueprint {
    fn get_format(&self) -> InstrFormat {
        self.instr_format.clone()
    }

    fn get_cooldown(&self) -> u32 {
        self.num_nops.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InstrFormat {
    R,
    I,
    J,
    N,
}

impl InstrFormat {
    fn from_str(s: &str) -> Self {
        match s {
            "R" => InstrFormat::R,
            "I" => InstrFormat::I,
            "J" => InstrFormat::J,
            "N" => InstrFormat::N,
            _ => panic!("Invalid instruction format"),
        }
    }
}

#[derive(Parser)]
struct Args {
    /// The path to the .asm file to parse
    // #[clap(default_value = "")]
    asm_file: String,

    /// The optional path to the output file
    #[clap(short, long, default_value = "output.dat")]
    output_path: String,
}

struct IsaParser {
    available_instr: HashMap<String, InstrBlueprint>,
    instr_writer: InstrWriter,
}

impl IsaParser {
    fn new(output_path: String) -> Self {
        let instrs = IsaParser::parse_opccsv().unwrap();
        let instr_writer = InstrWriter::new(PathBuf::from(output_path));

        IsaParser {
            available_instr: instrs,
            instr_writer,
        }
    }

    fn parse_opccsv() -> io::Result<HashMap<String, InstrBlueprint>> {
        let mut opcodes = HashMap::new();
        let csv_content = include_str!("opcs.csv"); // Embed the file into the binary

        for line in csv_content.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            let opc = parts[0].to_string();
            let bin_rep = parts[1].to_string();
            let instr_format = InstrFormat::from_str(parts[2]);
            let num_operands = parts[3].parse::<u32>().unwrap();
            let instr = InstrBlueprint {
                opcode: opc.clone(),
                instr_format,
                bin_rep,
                num_nops: num_operands,
            };
            opcodes.insert(opc, instr);
        }

        Ok(opcodes)
    }

    fn write(&self) {
        self.instr_writer.write_lines();
    }

    fn get_opcode(&self, opcode: &str) -> Option<&InstrBlueprint> {
        self.available_instr.get(opcode)
    }

    fn get_type(&self, first_part: &str) -> LineType {
        if first_part.chars().next().unwrap().is_uppercase() {
            LineType::Label
        } else {
            let instr_blueprint = self.get_opcode(first_part);
            match instr_blueprint {
                Some(_) => LineType::Instr,
                None => LineType::Macro,
            }
        }
    }

    fn handle_line(&mut self, line: &str) {
        // check if the line is empty or a comment
        if line.is_empty() || line.chars().next().unwrap() == '#' {
            return;
        }
        // split at whitespace and clean ","

        let mut parts = line.split_whitespace().map(|s| s.trim_matches(','));

        let first = parts.next().unwrap();
        let rest = parts.collect::<Vec<_>>();
        let instr_type = self.get_type(first);
        match instr_type {
            LineType::Instr => {
                let opc = first;
                let args: Vec<&str> = rest.iter().map(|s| *s).collect();
                self.handle_instr(opc, args);
            }
            LineType::Label => {
                self.handle_label(first);
            }
            LineType::Macro => {
                self.handle_macro(first, rest);
            }
        }
    }

    fn handle_instr(&mut self, opcode: &str, args: Vec<&str>) {
        let instr_blueprint = (*self.get_opcode(opcode).unwrap()).clone();
        // println!("Handling instr: {} with args: {:?}", opcode, args);

        self.instr_writer.handle_instr(instr_blueprint, args);
    }

    fn handle_label(&mut self, label: &str) {
        self.instr_writer.handle_label(label);
    }

    fn handle_macro(&mut self, macro_name: &str, args: Vec<&str>) {
        match macro_name {
            "push" => {
                // decrement the stack pointer
                let sub_args: Vec<&str> = vec!["R31", "R30", "R31"]; // R31 - R30 = R31
                self.handle_instr("sub", sub_args);

                // store the register in the memory
                let stw_args: Vec<&str> = vec![args[0], "R31", "0"];
                self.handle_instr("stw", stw_args);
            }
            "pop" => {
                // load the register from the memory
                let ldw_args: Vec<&str> = vec![args[0], "R31", "0"];
                self.handle_instr("ldw", ldw_args);

                // increment the stack pointer
                let add_args: Vec<&str> = vec!["R31", "R30", "R31"]; // R31 + R30 = R31
                self.handle_instr("add", add_args);
            }
            "call" => {
                let sub_args: Vec<&str> = vec!["R31", "R30", "R31"]; // R31 - R30 = R31
                self.handle_instr("sub", sub_args);

                // store the return address in the memory with movpc
                let movpc_args: Vec<&str> = vec!["R0", "R31", "0"];
                self.handle_instr("movpc", movpc_args);

                // jump to the label
                let jmp_args: Vec<&str> = vec![args[0]];
                self.handle_instr("jmp", jmp_args);
            }
            "ret" => {
                // load the return address from the memory
                let ldw_args: Vec<&str> = vec!["R31", "R29", "0"];
                self.handle_instr("ldw", ldw_args);

                // increment the return address to get the next instruction
                let add_args: Vec<&str> = vec!["R29", "R30", "R29"]; // R29 + R30 = R29
                self.handle_instr("add", add_args);

                // increment the stack pointer
                let add_args: Vec<&str> = vec!["R31", "R30", "R31"]; // R31 + R30 = R31
                self.handle_instr("add", add_args);

                // jump to the return address
                let jmp_args: Vec<&str> = vec!["r29", "r0", "0"];
                self.handle_instr("jmpr", jmp_args);
            }
            _ => panic!("Invalid macro"),
        }
    }

    fn print_opcodes(&self) {
        println!("Opcodes:");
        println!("{:#?}", self.available_instr);
    }
}

struct InstrWriter {
    bin_lines: Vec<BinEntry>,
    output_file: PathBuf,
    linenumber: u32,
    reg_cooldown: HashMap<String, u32>,
    label_map: HashMap<String, u32>,
}

impl InstrWriter {
    fn new(output_file: PathBuf) -> Self {
        let mut reg_cooldown = HashMap::new();
        for i in 0..32 {
            reg_cooldown.insert(format!("{}", i), 0);
        }
        InstrWriter {
            bin_lines: Vec::new(),
            output_file,
            linenumber: 0,
            reg_cooldown,
            label_map: HashMap::new(),
        }
    }
    fn handle_instr(&mut self, instr: InstrBlueprint, args: Vec<&str>) {
        match instr.instr_format {
            InstrFormat::R => self.handle_r(&instr, &args),
            InstrFormat::I => self.handle_i(&instr, &args),
            InstrFormat::J => self.handle_j(&instr, &args),
            InstrFormat::N => self.handle_n(),
        }
        // set the cooldown for the second reg on I instructions and for the third reg on R instructions
        let instr_format = instr.get_format();
        if instr_format != InstrFormat::J && instr_format != InstrFormat::N {
            let reg_to_cooldown = match instr_format {
                InstrFormat::I => 1,
                InstrFormat::R => 2,
                _ => panic!("Doesn't happen"),
            };

            let reg = args[reg_to_cooldown];
            let reg_num = reg
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<u32>()
                .unwrap();

            let cooldown = instr.get_cooldown();
            self.reg_cooldown.insert(reg_num.to_string(), cooldown);
        }
    }

    fn handle_r(&mut self, instr: &InstrBlueprint, args: &Vec<&str>) {
        let mut bin_rep = instr.bin_rep.clone();
        for arg in args {
            let reg = arg;
            let reg_num = reg
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<u32>()
                .unwrap();
            let reg_cooldown = *self.reg_cooldown.get(&reg_num.to_string()).unwrap();
            if reg_cooldown > 0 {
                //insert nops
                for _ in 0..reg_cooldown {
                    self.handle_n();
                }
            }
            // append the reg_num to the bin rep
            bin_rep.push_str(&format!("{:05b}", reg_num));
        }
        // add 11 0s to the end of the bin rep FIXME:
        bin_rep.push_str("00000000000");
        let comment = format!(" -- {} {:?}", instr.opcode, args);
        self.append_line(BinEntry::WithoutLabel(bin_rep + &comment));
    }

    fn handle_i(&mut self, instr: &InstrBlueprint, args: &Vec<&str>) {
        let mut bin_rep = instr.bin_rep.clone();
        // parse the two registers and the immediate
        let (regs, imm) = args.split_at(2);

        for reg in regs {
            let reg_num = reg
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<u32>()
                .unwrap();

            let reg_cooldown = *self.reg_cooldown.get(&reg_num.to_string()).unwrap();
            if reg_cooldown > 0 {
                //insert nops
                for _ in 0..reg_cooldown {
                    self.handle_n();
                }
            }

            bin_rep.push_str(&format!("{:05b}", reg_num));
        }
        let imm_num: i32;
        // check if the immediate is a label
        if imm[0].chars().next().unwrap().is_uppercase() {
            // convert the label to a number by looking it up in the label map and subtracting the current line number
            //imm_num = *self.label_map.get(imm[0]).unwrap() as i32 - self.linenumber as i32 - 1;
            self.append_line(BinEntry::WithRelLabel(bin_rep, imm[0].to_string()));
            return;
        } else {
            imm_num = imm[0].parse::<i32>().unwrap();
        }

        let mut imm_bin_rep = format!("{:016b}", imm_num);
        let imm_len = imm_bin_rep.len();
        imm_bin_rep = imm_bin_rep.chars().skip(imm_len - 16).collect::<String>();
        bin_rep.push_str(&imm_bin_rep);
        let comment = format!(" -- {} {:?}", instr.opcode, args);
        self.append_line(BinEntry::WithoutLabel(bin_rep + &comment));
    }

    fn handle_j(&mut self, instr: &InstrBlueprint, args: &Vec<&str>) {
        let mut bin_rep = instr.bin_rep.clone();
        // check whether the argument is a label or a Reg
        if args[0].chars().next().unwrap().is_uppercase() {
            self.append_line(BinEntry::WithAbsLabel(bin_rep, args[0].to_string()));
            for _ in 0..4 {
                self.handle_n();
            }
            return;
        }
        let reg = args[0];
        let reg_num = reg
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<u32>()
            .unwrap();
        let reg_cooldown = *self.reg_cooldown.get(&reg_num.to_string()).unwrap();
        if reg_cooldown > 0 {
            //insert nops
            for _ in 0..reg_cooldown {
                self.handle_n();
            }
        }
        bin_rep.push_str(&format!("{:05b}", reg_num));
        // add 21 0s to the end of the bin rep FIXME:
        bin_rep.push_str("000000000000000000000");
        let comment = format!(" -- {} {:?}", instr.opcode, args);
        self.append_line(BinEntry::WithoutLabel(bin_rep + &comment));

        // append 4 nops maybe FIXME
        for _ in 0..4 {
            self.handle_n();
        }
    }

    fn handle_n(&mut self) {
        // append a line of 32 0s
        let line = format!("{:032b}", 0);
        let comment = " -- nop".to_string();
        self.append_line(BinEntry::WithoutLabel(line + &comment));
    }

    fn handle_label(&mut self, label: &str) {
        self.label_map.insert(label.to_string(), self.linenumber);
    }

    fn append_line(&mut self, line: BinEntry) {
        // prepend the linenumber in dec with a space
        let line_string = line.get_string();
        let line_string = format!("{} {}", self.linenumber, line_string);
        self.bin_lines.push(line.update_str(line_string));
        self.linenumber += 1;

        // decrement the reg cooldowns
        for (reg, cooldown) in self.reg_cooldown.iter_mut() {
            if *cooldown > 0 {
                *cooldown -= 1;
            }
        }
    }

    fn write_lines(&self) {
        let mut file = File::create(&self.output_file).unwrap();
        for (line_number, line) in self.bin_lines.iter().enumerate() {
            // write all the lines to the file with a newline character
            let mut bin_rep = line.get_string();
            match line {
                BinEntry::WithRelLabel(_, label) => {
                    let label_line = self.label_map.get(label).unwrap();
                    let imm = *label_line as i32 - line_number as i32;
                    let mut imm_bin_rep = format!("{:016b}", imm)
                        .chars()
                        .rev()
                        .take(16)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>();
                    bin_rep.push_str(&imm_bin_rep);
                }
                BinEntry::WithAbsLabel(_, label) => {
                    let label_line = self.label_map.get(label).unwrap();
                    let imm = *label_line as u32;
                    bin_rep.push_str(
                        &format!("{:026b}", imm)
                            .chars()
                            .rev()
                            .take(26)
                            .collect::<String>()
                            .chars()
                            .rev()
                            .collect::<String>(),
                    );
                }
                _ => {}
            }
            file.write_all(bin_rep.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
        println!(
            "Wrote {} lines to {}",
            self.bin_lines.len(),
            self.output_file.display()
        );
    }
}

#[derive(Debug, Clone)]
enum BinEntry {
    WithAbsLabel(String, String),
    WithRelLabel(String, String),
    WithoutLabel(String),
}

impl BinEntry {
    fn update_str(&self, new_str: String) -> Self {
        match self {
            BinEntry::WithRelLabel(_, label) => BinEntry::WithRelLabel(new_str, label.clone()),
            BinEntry::WithAbsLabel(_, label) => BinEntry::WithAbsLabel(new_str, label.clone()),
            BinEntry::WithoutLabel(_) => BinEntry::WithoutLabel(new_str),
        }
    }
    fn get_string(&self) -> String {
        match self {
            BinEntry::WithRelLabel(bin, _label) => bin.clone(),
            BinEntry::WithAbsLabel(bin, _label) => bin.clone(),
            BinEntry::WithoutLabel(bin) => bin.clone(),
        }
    }
}
