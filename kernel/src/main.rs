use std::env;
use std::fs;
use std::io::{self, Write};
use neural0_assembler::SimpleAssembler;
use neural0_kernel::{VM, Trap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: n0 <command> <args>");
        eprintln!("Commands:");
        eprintln!("  run <file.n0b> - Execute a NEURAL-0 binary");
        eprintln!("  asm <file.n0asm> - Assemble a NEURAL-0 assembly file");
        std::process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: n0 run <file.n0b>");
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let binary = fs::read(input_file)?;
            let code = module_code_section(&binary)
                .map_err(|e| format!("Invalid NEURAL-0 module: {}", e))?;
            
            let mut vm = VM::new(1024, 65536); // 1KB stack, 64KB memory
            match vm.load_and_run(code) {
                Ok(()) => {
                    writeln!(io::stdout(), "Program executed successfully")?;
                    if vm.is_halted() {
                        writeln!(io::stdout(), "Program halted normally")?;
                    } else if vm.has_yielded() {
                        writeln!(io::stdout(), "Program yielded")?;
                    }
                    writeln!(io::stdout(), "Final stack: {:?}", vm.stack())?;
                }
                Err(Trap::DivZero) => {
                    eprintln!("Error: Division by zero");
                    std::process::exit(1);
                }
                Err(Trap::TypeMismatch) => {
                    eprintln!("Error: Type mismatch");
                    std::process::exit(1);
                }
                Err(Trap::StackOverflow) => {
                    eprintln!("Error: Stack overflow");
                    std::process::exit(1);
                }
                Err(Trap::StackUnderflow) => {
                    eprintln!("Error: Stack underflow");
                    std::process::exit(1);
                }
                Err(Trap::OOBMemory) => {
                    eprintln!("Error: Out of bounds memory access");
                    std::process::exit(1);
                }
                Err(Trap::OOM) => {
                    eprintln!("Error: Out of memory");
                    std::process::exit(1);
                }
                Err(Trap::InvalidPointer) => {
                    eprintln!("Error: Invalid pointer");
                    std::process::exit(1);
                }
                Err(Trap::InvalidOpcode) => {
                    eprintln!("Error: Invalid opcode");
                    std::process::exit(1);
                }
                Err(Trap::MissingCap) => {
                    eprintln!("Error: Missing capability");
                    std::process::exit(1);
                }
                Err(Trap::CapExpired) => {
                    eprintln!("Error: Capability expired");
                    std::process::exit(1);
                }
                Err(Trap::CapExhausted) => {
                    eprintln!("Error: Capability exhausted");
                    std::process::exit(1);
                }
                Err(Trap::SnapshotInvalid) => {
                    eprintln!("Error: Invalid snapshot");
                    std::process::exit(1);
                }
                Err(Trap::NotImplemented) => {
                    eprintln!("Error: Not implemented");
                    std::process::exit(1);
                }
            }
        }
        "asm" => {
            if args.len() < 3 {
                eprintln!("Usage: n0 asm <file.n0asm> [-o <output.n0b>]");
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let mut output_file = None;
            
            // Parse optional arguments
            let mut i = 3;
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
            
            let source = fs::read_to_string(input_file)?;
            let binary = SimpleAssembler::assemble(&source)
                .map_err(|e| format!("Assembly error: {}", e))?;

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
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: run, asm");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn module_code_section(binary: &[u8]) -> Result<&[u8], String> {
    let magic = 0x4E30u16.to_be_bytes();
    if binary.len() < 2 || binary[0..2] != magic {
        return Ok(binary);
    }

    if binary.len() < 14 {
        return Err("module header is truncated".to_string());
    }

    let version = u16::from_be_bytes([binary[2], binary[3]]);
    if version != 0x0001 {
        return Err(format!("unsupported module version {:#06x}", version));
    }

    let flags = u16::from_be_bytes([binary[8], binary[9]]);
    let mut offset = 10usize;

    if flags & 0x0001 != 0 {
        let input_count = read_u8(binary, &mut offset)? as usize;
        advance(binary, &mut offset, input_count)?;
        let output_count = read_u8(binary, &mut offset)? as usize;
        advance(binary, &mut offset, output_count)?;
    }

    let code_len = read_u32_be(binary, &mut offset)? as usize;
    let code_start = offset;
    advance(binary, &mut offset, code_len)?;

    let data_len = read_u32_be(binary, &mut offset)? as usize;
    advance(binary, &mut offset, data_len)?;

    Ok(&binary[code_start..code_start + code_len])
}

fn read_u8(binary: &[u8], offset: &mut usize) -> Result<u8, String> {
    if *offset >= binary.len() {
        return Err("unexpected end of module".to_string());
    }
    let value = binary[*offset];
    *offset += 1;
    Ok(value)
}

fn read_u32_be(binary: &[u8], offset: &mut usize) -> Result<u32, String> {
    if *offset + 4 > binary.len() {
        return Err("unexpected end of module".to_string());
    }
    let value = u32::from_be_bytes([
        binary[*offset],
        binary[*offset + 1],
        binary[*offset + 2],
        binary[*offset + 3],
    ]);
    *offset += 4;
    Ok(value)
}

fn advance(binary: &[u8], offset: &mut usize, len: usize) -> Result<(), String> {
    let next = offset
        .checked_add(len)
        .ok_or_else(|| "module offset overflow".to_string())?;
    if next > binary.len() {
        return Err("unexpected end of module".to_string());
    }
    *offset = next;
    Ok(())
}
