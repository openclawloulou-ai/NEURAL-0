use crate::parser::{Instruction, DataItem};
use crate::opcodes::*;
use crate::value::Value;

/// Code generator that converts AST to binary NEURAL-0 format
pub struct CodeGen {
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub labels: std::collections::HashMap<String, u32>,
    pub pc: u32,
    pub data_ptr: u32,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
            labels: std::collections::HashMap::new(),
            pc: 0,
            data_ptr: 0,
        }
    }

    /// Generate binary from parsed instructions
    pub fn generate(&mut self, instructions: &[Instruction]) -> Result<Vec<u8>, String> {
        // First pass: collect labels and resolve forward references
        self.collect_labels(instructions)?;
        
        // Second pass: generate actual code
        self.generate_code(instructions)?;
        
        // Combine code and data sections with proper header
        Ok(self.build_binary())
    }

    fn collect_labels(&mut self, instructions: &[Instruction]) -> Result<(), String> {
        self.pc = 0;
        for instr in instructions {
            match instr {
                Instruction::Module(_) => {
                    // Directive, doesn't affect PC
                }
                Instruction::Ports { .. } => {
                    // Directive, doesn't affect PC
                }
                Instruction::Input(_) => {
                    // Directive, doesn't affect PC
                }
                Instruction::Output(_) => {
                    // Directive, doesn't affect PC
                }
                Instruction::Data(_) => {
                    // Directive, doesn't affect PC
                }
                _ => {
                    // Actual instruction - count its size
                    self.pc += self.instruction_size(instr)?;
                }
            }
        }
        Ok(())
    }

    fn instruction_size(&self, instr: &Instruction) -> Result<u32, String> {
        Ok(match instr {
            Instruction::Nop => 2,
            Instruction::PushI64(_) => 2 + 8,
            Instruction::PushF64(_) => 2 + 8,
            Instruction::PushBytes(ref bytes) => 2 + 2 + bytes.len() as u32,
            Instruction::Pop => 2,
            Instruction::Dup => 2,
            Instruction::Swap => 2,
            Instruction::PushNil => 2,
            Instruction::Add => 2,
            Instruction::Sub => 2,
            Instruction::Mul => 2,
            Instruction::Div => 2,
            Instruction::Mod => 2,
            Instruction::Neg => 2,
            Instruction::Eq => 2,
            Instruction::Ne => 2,
            Instruction::Lt => 2,
            Instruction::Le => 2,
            Instruction::Gt => 2,
            Instruction::Ge => 2,
            Instruction::And => 2,
            Instruction::Or => 2,
            Instruction::Xor => 2,
            Instruction::Not => 2,
            Instruction::BoolAnd => 2,
            Instruction::BoolOr => 2,
            Instruction::BoolXor => 2,
            Instruction::BoolNot => 2,
            Instruction::Jump(_) => 2 + 4,
            Instruction::JumpIf(_) => 2 + 4,
            Instruction::JumpIfNot(_) => 2 + 4,
            Instruction::Call(_, _) => 2 + 4 + 2,
            Instruction::Return => 2,
            Instruction::Halt => 2,
            Instruction::Yield => 2,
            Instruction::Emit(_) => 2 + 2,
            Instruction::Load(_) => 2 + 4,
            Instruction::Store(_) => 2 + 4,
            Instruction::Alloc(_) => 2 + 4,
            Instruction::Free => 2,
            Instruction::Copy { .. } => 2 + 4 + 4 + 4,
            Instruction::Set { .. } => 2 + 4 + 1,
            Instruction::Get(_) => 2 + 4,
            Instruction::VecPull(_) => 2 + 4,
            Instruction::VecPush(_) => 2 + 4,
            Instruction::ProbB { .. } => 2 + 4 + 4,
            Instruction::SnapS => 2,
            Instruction::SnapR(_) => 2 + 4,
            Instruction::ToolX { .. } => 2 + 4 + 2,
        })
    }

    fn generate_code(&mut self, instructions: &[Instruction]) -> Result<(), String> {
        self.pc = 0;
        for instr in instructions {
            match instr {
                Instruction::Module(id) => {
                    // Will be handled in binary header
                }
                Instruction::Ports { inputs, outputs } => {
                    // Will be handled in binary header
                }
                Instruction::Input(_) => {
                    // Will be handled in binary header
                }
                Instruction::Output(_) => {
                    // Will be handled in binary header
                }
                Instruction::Data(items) => {
                    self.generate_data(items)?;
                }
                _ => {
                    self.generate_instruction(instr)?;
                }
            }
        }
        Ok(())
    }

    fn generate_instruction(&mut self, instr: &Instruction) -> Result<(), String> {
        // Emit opcode
        let opcode = self.instruction_to_opcode(instr)?;
        self.code.extend_from_slice(&opcode.to_be_bytes());
        
        // Emit operands
        match instr {
            Instruction::Nop => {}
            Instruction::PushI64(val) => self.code.extend_from_slice(&val.to_be_bytes()),
            Instruction::PushF64(val) => self.code.extend_from_slice(&val.to_be_bytes()),
            Instruction::PushBytes(bytes) => {
                self.code.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                self.code.extend_from_slice(bytes);
            }
            Instruction::Pop => {}
            Instruction::Dup => {}
            Instruction::Swap => {}
            Instruction::PushNil => {}
            Instruction::Add => {}
            Instruction::Sub => {}
            Instruction::Mul => {}
            Instruction::Div => {}
            Instruction::Mod => {}
            Instruction::Neg => {}
            Instruction::Eq => {}
            Instruction::Ne => {}
            Instruction::Lt => {}
            Instruction::Le => {}
            Instruction::Gt => {}
            Instruction::Ge => {}
            Instruction::And => {}
            Instruction::Or => {}
            Instruction::Xor => {}
            Instruction::Not => {}
            Instruction::BoolAnd => {}
            Instruction::BoolOr => {}
            Instruction::BoolXor => {}
            Instruction::BoolNot => {}
            Instruction::Jump(label) => {
                let offset = self.resolve_label(label)? as i32;
                self.code.extend_from_slice(&(offset as u32).to_be_bytes());
            }
            Instruction::JumpIf(label) => {
                let offset = self.resolve_label(label)? as i32;
                self.code.extend_from_slice(&(offset as u32).to_be_bytes());
            }
            Instruction::JumpIfNot(label) => {
                let offset = self.resolve_label(label)? as i32;
                self.code.extend_from_slice(&(offset as u32).to_be_bytes());
            }
            Instruction::Call(module, function) => {
                self.code.extend_from_slice(&module.to_be_bytes());
                self.code.extend_from_slice(&function.to_be_bytes());
            }
            Instruction::Return => {}
            Instruction::Halt => {}
            Instruction::Yield => {}
            Instruction::Emit(event_id) => {
                self.code.extend_from_slice(&event_id.to_be_bytes());
            }
            Instruction::Load(addr) => {
                self.code.extend_from_slice(&addr.to_be_bytes());
            }
            Instruction::Store(addr) => {
                self.code.extend_from_slice(&addr.to_be_bytes());
            }
            Instruction::Alloc(size) => {
                self.code.extend_from_slice(&size.to_be_bytes());
            }
            Instruction::Free => {}
            Instruction::Copy { dst, src, len } => {
                self.code.extend_from_slice(&dst.to_be_bytes());
                self.code.extend_from_slice(&src.to_be_bytes());
                self.code.extend_from_slice(&len.to_be_bytes());
            }
            Instruction::Set { addr, value } => {
                self.code.extend_from_slice(&addr.to_be_bytes());
                self.code.push(*value);
            }
            Instruction::Get(addr) => {
                self.code.extend_from_slice(&addr.to_be_bytes());
            }
            Instruction::VecPull(hash_or_label) => {
                // For v1, treat as immediate hash (we'd need label resolution in future)
                let hash = self.parse_hash_or_label(hash_or_label)?;
                self.code.extend_from_slice(&hash.to_be_bytes());
            }
            Instruction::VecPush(hash_or_label) => {
                // For v1, treat as immediate hash
                let hash = self.parse_hash_or_label(hash_or_label)?;
                self.code.extend_from_slice(&hash.to_be_bytes());
            }
            Instruction::ProbB { threshold, offset } => {
                self.code.extend_from_slice(&((*threshold as f32).to_be_bytes()));
                let offset_val = self.resolve_label(offset)? as i32;
                self.code.extend_from_slice(&(offset_val as u32).to_be_bytes());
            }
            Instruction::SnapS => {}
            Instruction::SnapR(snapshot_id) => {
                self.code.extend_from_slice(&snapshot_id.to_be_bytes());
            }
            Instruction::ToolX { cap_token, tool_id } => {
                self.code.extend_from_slice(&cap_token.to_be_bytes());
                self.code.extend_from_slice(&tool_id.to_be_bytes());
            }
        }
        Ok(())
    }

    fn generate_data(&mut self, items: &[DataItem]) -> Result<(), String> {
        self.data_ptr = 0;
        for item in items {
            match item {
                DataItem::Bytes { label: _, data } => {
                    self.data.extend_from_slice(&(data.len() as u16).to_be_bytes());
                    self.data.extend_from_slice(data);
                    self.data_ptr += 2 + data.len() as u32;
                }
                DataItem::Quad { label: _, value } => {
                    self.data.extend_from_slice(&value.to_be_bytes());
                    self.data_ptr += 8;
                }
                DataItem::Float { label: _, value } => {
                    self.data.extend_from_slice(&value.to_be_bytes());
                    self.data_ptr += 8;
                }
                DataItem::Align(align) => {
                    let align = *align as u32;
                    let padding = (align - (self.data_ptr % align)) % align;
                    self.data.extend_from_slice(std::iter::repeat(0).take(padding as usize).collect::<Vec<u8>>());
                    self.data_ptr += padding;
                }
            }
        }
        Ok(())
    }

    fn instruction_to_opcode(&self, instr: &Instruction) -> Result<OpCode, String> {
        match instr {
            Instruction::Nop => Ok(OpCode::Nop),
            Instruction::PushI64(_) => Ok(OpCode::PUSH_I64),
            Instruction::PushF64(_) => Ok(OpCode::PUSH_F64),
            Instruction::PushBytes(_) => Ok(OpCode::PUSH_BYTES),
            Instruction::Pop => Ok(OpCode::POP),
            Instruction::Dup => Ok(OpCode::DUP),
            Instruction::Swap => Ok(OpCode::SWAP),
            Instruction::PushNil => Ok(OpCode::PUSH_NIL),
            Instruction::Add => Ok(OpCode::ADD),
            Instruction::Sub => Ok(OpCode::SUB),
            Instruction::Mul => Ok(OpCode::MUL),
            Instruction::Div => Ok(OpCode::DIV),
            Instruction::Mod => Ok(OpCode::MOD),
            Instruction::Neg => Ok(OpCode::NEG),
            Instruction::Eq => Ok(OpCode::EQ),
            Instruction::Ne => Ok(OpCode::NE),
            Instruction::Lt => Ok(OpCode::LT),
            Instruction::Le => Ok(OpCode::LE),
            Instruction::Gt => Ok(OpCode::GT),
            Instruction::Ge => Ok(OpCode::GE),
            Instruction::And => Ok(OpCode::AND),
            Instruction::Or => Ok(OpCode::OR),
            Instruction::Xor => Ok(OpCode::XOR),
            Instruction::Not => Ok(OpCode::NOT),
            Instruction::BoolAnd => Ok(OpCode::BOOL_AND),
            Instruction::BoolOr => Ok(OpCode::BOOL_OR),
            Instruction::BoolXor => Ok(OpCode::BOOL_XOR),
            Instruction::BoolNot => Ok(OpCode::BOOL_NOT),
            Instruction::Jump(_) => Ok(OpCode::JUMP),
            Instruction::JumpIf(_) => Ok(OpCode::JUMP_IF),
            Instruction::JumpIfNot(_) => Ok(OpCode::JUMP_IF_NOT),
            Instruction::Call(_, _) => Ok(OpCode::CALL),
            Instruction::Return => Ok(OpCode::RETURN),
            Instruction::Halt => Ok(OpCode::HALT),
            Instruction::Yield => Ok(OpCode::YIELD),
            Instruction::Emit(_) => Ok(OpCode::EMIT),
            Instruction::Load(_) => Ok(OpCode::LOAD),
            Instruction::Store(_) => Ok(OpCode::STORE),
            Instruction::Alloc(_) => Ok(OpCode::ALLOC),
            Instruction::Free => Ok(OpCode::FREE),
            Instruction::Copy { .. } => Ok(OpCode::COPY),
            Instruction::Set { .. } => Ok(OpCode::SET),
            Instruction::Get(_) => Ok(OpCode::GET),
            Instruction::VecPull(_) => Ok(OpCode::VEC_PULL),
            Instruction::VecPush(_) => Ok(OpCode::VEC_PUSH),
            Instruction::ProbB { .. } => Ok(OpCode::PROB_B),
            Instruction::SnapS => Ok(OpCode::SNAP_S),
            Instruction::SnapR(_) => Ok(OpCode::SNAP_R),
            Instruction::ToolX { .. } => Ok(OpCode::TOOL_X),
        }
    }

    fn resolve_label(&self, label: &str) -> Result<u32, String> {
        self.labels.get(label)
            .copied()
            .ok_or_else(|| format!("Undefined label: {}", label))
    }

    fn parse_hash_or_label(&self, s: &str) -> Result<u32, String> {
        // Try to parse as hex number first
        if s.starts_with("0x") {
            u32::from_str_radix(&s[2..], 16)
                .map_err(|_| format!("Invalid hex number: {}", s))
        } else {
            // Treat as label - in v1 we'll just hash the string for simplicity
            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            let hash = hasher.finish() as u32;
            Ok(hash)
        }
    }

    fn build_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        
        // For v1, we'll create a simple module format:
        // [Magic: 2 bytes][Version: 2 bytes][Module ID: 4 bytes][Flags: 2 bytes]
        // [Code Length: 4 bytes][Code: n bytes][Data Length: 4 bytes][Data: m bytes]
        
        // Magic: "N0"
        binary.extend_from_slice(&0x4E30u16.to_be_bytes());
        // Version: 1.0.0
        binary.extend_from_slice(&0x0001u16.to_be_bytes());
        // Module ID: 0 for now (would be hash in real implementation)
        binary.extend_from_slice(&0u32.to_be_bytes());
        // Flags: no ports, no state
        binary.extend_from_slice(&0u16.to_be_bytes());
        
        // Code length
        binary.extend_from_slice(&(self.code.len() as u32).to_be_bytes());
        // Code
        binary.extend_from_slice(&self.code);
        
        // Data length
        binary.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        // Data
        binary.extend_from_slice(&self.data);
        
        binary
    }
}