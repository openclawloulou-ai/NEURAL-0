use std::env;
use std::fs;
use std::io::{self, Write};
use neural0_assembler::SimpleAssembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: n0asm <input.n0asm> [-o <output.n0b>]");
        std::process::exit(1);
    }
    
    let input_file = &args[1];
    let mut output_file = None;
    
    // Parse optional arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: {} requires an argument", args[i]);
                    std::process::exit(1);
                }
                output_file = Some(args[i + 1].clone());
                i += 2;
            }
            arg => {
                eprintln!("Error: Unknown argument {}", arg);
                std::process::exit(1);
            }
        }
    }
    
    // Read input file
    let source = fs::read_to_string(input_file)?;
    
    // Assemble
    let binary = SimpleAssembler::assemble(&source)
        .map_err(|e| format!("Assembly error: {}", e))?;
    
    // Write output
    let output = output_file.unwrap_or_else(|| {
        let mut out = input_file.clone();
        if out.ends_with(".n0asm") {
            out.truncate(out.len() - 6);
        }
        out.push_str(".n0b");
        out
    });
    
    fs::write(&output, &binary)?;
    writeln!(io::stdout(), "Wrote {} bytes to {}", binary.len(), output)?;
    
    Ok(())
}