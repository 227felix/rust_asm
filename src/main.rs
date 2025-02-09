use clap::Parser;
fn main() {
    // parse a .asm file from the command line with clap
    let args = Args::parse();
    let asm_file = if args.asm_file.is_empty() {
        "alt\\fib.asm".to_string()
    } else {
        args.asm_file
    };
    println!("Parsing file: {}", asm_file);
    let asm = std::fs::read_to_string(asm_file).unwrap();
    for line in asm.lines() {
        println!("{}", line);
    }
}

#[derive(Parser)]
struct Args {
    /// The path to the .asm file to parse
    #[clap(default_value = "")]
    asm_file: String,
}
