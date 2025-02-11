use clap::Parser;
use color_backtrace::install;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

fn main() {
    install();

    // parse a .asm file from the command line with clap
    let args = Args::parse();
    let asm_file = if args.asm_file.is_empty() {
        "alt\\fib.asm".to_string()
    } else {
        args.asm_file
    };
    println!("Parsing file: {}", asm_file);

    let mut opcodes_handler = IsaParser::new();

    let asm = std::fs::read_to_string(asm_file).unwrap();
    let lines = asm.lines().map(|l| l.trim()).collect::<Vec<_>>();

    let labels: Vec<(&str, u32)> = Vec::new();
    let linenumber: u32 = 0;

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

#[derive(Debug, Clone)]
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
    #[clap(default_value = "")]
    asm_file: String,
}

struct IsaParser {
    available_instr: HashMap<String, InstrBlueprint>,
    instr_writer: InstrWriter,
}
impl IsaParser {
    fn new() -> Self {
        let instrs = IsaParser::parse_opccsv("C:\\Git Repositories\\rust_asm\\opcs.csv").unwrap();
        let instr_writer = InstrWriter::new(PathBuf::from("output.dat"));
        IsaParser {
            available_instr: instrs,
            instr_writer,
        }
    }
    fn parse_opccsv(filename: &str) -> io::Result<HashMap<String, InstrBlueprint>> {
        let mut opcodes = HashMap::new();
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
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
                todo!()
            }
            LineType::Macro => {
                todo!()
            }
        }
    }

    fn handle_instr(&mut self, opcode: &str, args: Vec<&str>) {
        let instr_blueprint = (*self.get_opcode(opcode).unwrap()).clone();

        self.instr_writer.handle_instr(instr_blueprint, args);
    }

    fn print_opcodes(&self) {
        println!("Opcodes:");
        println!("{:#?}", self.available_instr);
    }
}

struct InstrWriter {
    bin_lines: Vec<String>,
    output_file: PathBuf,
    linenumber: u32,
    reg_cooldown: HashMap<String, u32>,
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
        }
    }
    fn handle_instr(&mut self, instr: InstrBlueprint, args: Vec<&str>) {
        match instr.instr_format {
            InstrFormat::R => self.handle_r(instr, args),
            InstrFormat::I => self.handle_i(instr, args),
            InstrFormat::J => self.handle_j(instr, args),
            InstrFormat::N => self.handle_n(),
        }
    }

    fn handle_r(&mut self, instr: InstrBlueprint, args: Vec<&str>) {
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
        self.append_line(bin_rep);
    }

    fn handle_i(&mut self, instr: InstrBlueprint, args: Vec<&str>) {
        let mut bin_rep = instr.bin_rep.clone();
        // parse the two registers and the immediate
        let (regs, imm) = args.split_at(2);
        let imm = imm[0].parse::<i32>().unwrap();
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
        let mut imm_bin_rep = format!("{:016b}", imm);
        let imm_len = imm_bin_rep.len();
        imm_bin_rep = imm_bin_rep.chars().skip(imm_len - 16).collect::<String>();
        bin_rep.push_str(&imm_bin_rep);
        self.append_line(bin_rep);
    }

    fn handle_j(&mut self, instr: InstrBlueprint, args: Vec<&str>) {
        let mut bin_rep = instr.bin_rep.clone();
        let imm = args[0].parse::<u32>().unwrap();
        bin_rep.push_str(&format!("{:026b}", imm));
        self.append_line(bin_rep);
    }

    fn handle_n(&mut self) {
        // append a line of 32 0s
        let line = format!("{:032b}", 0);
        self.append_line(line);
    }

    fn append_line(&mut self, line: String) {
        // prepend the linenumber in dec with a space
        let line = format!("{} {}", self.linenumber, line);
        self.bin_lines.push(line);
        self.linenumber += 1;
    }

    fn write_lines(&self) {
        let mut file = File::create(&self.output_file).unwrap();
        for line in self.bin_lines.iter() {
            // write all the lines to the file with a newline character
            file.write_all(line.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
    }
}
