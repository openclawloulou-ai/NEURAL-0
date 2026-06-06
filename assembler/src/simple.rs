/// Simple assembler for v1 - just handles basic instructions
pub struct SimpleAssembler;

impl SimpleAssembler {
    pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
        let mut binary = Vec::new();

        // Magic: "N0"
        binary.extend_from_slice(&0x4E30u16.to_be_bytes());
        // Version: 1.0.0
        binary.extend_from_slice(&0x0001u16.to_be_bytes());
        // Module ID: 0 (would be hash in real implementation)
        binary.extend_from_slice(&0u32.to_be_bytes());
        // Flags: no ports, no state
        binary.extend_from_slice(&0u16.to_be_bytes());

        // Parse simple instructions
        let mut code = Vec::new();
        let data = Vec::new();

        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mnemonic = parts[0].to_uppercase();
            match mnemonic.as_str() {
                "PUSH_I64" => {
                    if parts.len() < 2 {
                        return Err("PUSH_I64 requires an operand".to_string());
                    }
                    let value = parts[1]
                        .parse::<i64>()
                        .map_err(|_| format!("Invalid integer: {}", parts[1]))?;
                    code.extend_from_slice(&0x0002u16.to_be_bytes()); // PUSH_I64 opcode
                    code.extend_from_slice(&value.to_be_bytes());
                }
                "ADD" => {
                    code.extend_from_slice(&0x0010u16.to_be_bytes()); // ADD opcode
                }
                "HALT" => {
                    code.extend_from_slice(&0x0045u16.to_be_bytes()); // HALT opcode
                }
                _ => return Err(format!("Unsupported instruction: {}", mnemonic)),
            }
        }

        // Code section
        binary.extend_from_slice(&(code.len() as u32).to_be_bytes());
        binary.extend_from_slice(&code);

        // Data section
        binary.extend_from_slice(&(data.len() as u32).to_be_bytes());
        binary.extend_from_slice(&data);

        Ok(binary)
    }

    pub fn disassemble(binary: &[u8]) -> Result<String, String> {
        if binary.len() < 18 {
            return Err("Binary too short to be a valid NEURAL-0 module".to_string());
        }

        // Check magic
        if &binary[0..2] != &0x4E30u16.to_be_bytes() {
            return Err("Invalid magic number".to_string());
        }

        // Check version
        if &binary[2..4] != &0x0001u16.to_be_bytes() {
            return Err("Unsupported version".to_string());
        }

        let flags = u16::from_be_bytes([binary[8], binary[9]]);
        if flags != 0 {
            return Err("Simple disassembler does not support module flags".to_string());
        }

        // Read code length
        let code_len =
            u32::from_be_bytes([binary[10], binary[11], binary[12], binary[13]]) as usize;

        let code_start = 14;
        let code_end = code_start + code_len;
        if code_end + 4 > binary.len() {
            return Err("Binary truncated".to_string());
        }

        let data_len = u32::from_be_bytes([
            binary[code_end],
            binary[code_end + 1],
            binary[code_end + 2],
            binary[code_end + 3],
        ]) as usize;
        let data_start = code_end + 4;
        if data_start + data_len > binary.len() {
            return Err("Binary truncated".to_string());
        }

        let code = &binary[code_start..code_end];

        // Simple disassembly
        let mut output = String::new();
        let mut pc = 0;

        while pc < code.len() {
            // Read opcode (2 bytes)
            if pc + 2 > code.len() {
                return Err("Truncated opcode".to_string());
            }
            let opcode = u16::from_be_bytes([code[pc], code[pc + 1]]);
            pc += 2;

            match opcode {
                0x0002 => {
                    // PUSH_I64
                    if pc + 8 > code.len() {
                        return Err("Truncated PUSH_I64 instruction".to_string());
                    }
                    let value = i64::from_be_bytes([
                        code[pc],
                        code[pc + 1],
                        code[pc + 2],
                        code[pc + 3],
                        code[pc + 4],
                        code[pc + 5],
                        code[pc + 6],
                        code[pc + 7],
                    ]);
                    output.push_str(&format!("PUSH_I64 {}\n", value));
                    pc += 8;
                }
                0x0010 => {
                    // ADD
                    output.push_str("ADD\n");
                }
                0x0045 => {
                    // HALT
                    output.push_str("HALT\n");
                    break;
                }
                _ => output.push_str(&format!("UNKNOWN_OPCODE {:#06x}\n", opcode)),
            }
        }

        Ok(output)
    }
}
