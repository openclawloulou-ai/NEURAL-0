use crate::capability::CapabilityTable;
use crate::memory::Memory;
use crate::opcodes::*;
use crate::stack::Stack;
use crate::trap::Trap;
use crate::value::Value;

/// The NEURAL-0 Virtual Machine
pub struct VM {
    stack: Stack,
    memory: Memory,
    pc: u32, // Program counter
    halted: bool,
    yield_requested: bool,
    capabilities: CapabilityTable,
    // For snapshot/restore
    snapshot_id: Option<u32>,
}

impl VM {
    /// Create a new VM with default stack and memory sizes
    pub fn new(stack_size: usize, memory_size: usize) -> Self {
        Self {
            stack: Stack::new(stack_size),
            memory: Memory::new(memory_size),
            pc: 0,
            halted: false,
            yield_requested: false,
            capabilities: CapabilityTable::new(),
            snapshot_id: None,
        }
    }

    /// Load and execute a module from binary data
    pub fn load_and_run(&mut self, code: &[u8]) -> Result<(), Trap> {
        self.pc = 0;
        self.halted = false;
        self.yield_requested = false;
        self.stack.clear();
        // Note: We don't clear memory here - preserving it allows for persistence between runs

        self.execute_block(code)
    }

    /// Execute a block of code until HALT, YIELD, or trap
    fn execute_block(&mut self, code: &[u8]) -> Result<(), Trap> {
        while !self.halted && !self.yield_requested && self.pc < code.len() as u32 {
            self.execute_instruction(code)?;
        }
        Ok(())
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, code: &[u8]) -> Result<(), Trap> {
        if self.pc as usize >= code.len() {
            return Err(Trap::InvalidOpcode);
        }

        // Read opcode (2 bytes, big-endian)
        let opcode_val = read_u16_be(code, &mut (self.pc as usize));
        self.pc += 2; // Move past opcode

        let opcode = OpCode::from_u16(opcode_val).ok_or(Trap::InvalidOpcode)?;

        // Execute the opcode
        match opcode {
            OpCode::NOP => self.op_nop(),
            OpCode::PUSH_I64 => self.op_push_i64(code),
            OpCode::PUSH_F64 => self.op_push_f64(code),
            OpCode::PUSH_BYTES => self.op_push_bytes(code),
            OpCode::POP => self.op_pop(),
            OpCode::DUP => self.op_dup(),
            OpCode::SWAP => self.op_swap(),
            OpCode::PUSH_NIL => self.op_push_nil(),
            OpCode::ADD => self.op_add(),
            OpCode::SUB => self.op_sub(),
            OpCode::MUL => self.op_mul(),
            OpCode::DIV => self.op_div(),
            OpCode::MOD => self.op_mod(),
            OpCode::NEG => self.op_neg(),
            OpCode::EQ => self.op_eq(),
            OpCode::NE => self.op_ne(),
            OpCode::LT => self.op_lt(),
            OpCode::LE => self.op_le(),
            OpCode::GT => self.op_gt(),
            OpCode::GE => self.op_ge(),
            OpCode::AND => self.op_and(),
            OpCode::OR => self.op_or(),
            OpCode::XOR => self.op_xor(),
            OpCode::NOT => self.op_not(),
            OpCode::BOOL_AND => self.op_bool_and(),
            OpCode::BOOL_OR => self.op_bool_or(),
            OpCode::BOOL_XOR => self.op_bool_xor(),
            OpCode::BOOL_NOT => self.op_bool_not(),
            OpCode::JUMP => self.op_jump(code),
            OpCode::JUMP_IF => self.op_jump_if(code),
            OpCode::JUMP_IF_NOT => self.op_jump_if_not(code),
            OpCode::CALL => self.op_call(code),
            OpCode::RETURN => self.op_return(),
            OpCode::HALT => self.op_halt(),
            OpCode::YIELD => self.op_yield(),
            OpCode::EMIT => self.op_emit(code),
            OpCode::LOAD => self.op_load(code),
            OpCode::STORE => self.op_store(code),
            OpCode::ALLOC => self.op_alloc(code),
            OpCode::FREE => self.op_free(),
            OpCode::COPY => self.op_copy(code),
            OpCode::SET => self.op_set(code),
            OpCode::GET => self.op_get(code),
            OpCode::VEC_PULL => self.op_vec_pull(code),
            OpCode::VEC_PUSH => self.op_vec_push(code),
            OpCode::PROB_B => self.op_prob_b(code),
            OpCode::SNAP_S => self.op_snap_s(),
            OpCode::SNAP_R => self.op_snap_r(code),
            OpCode::TOOL_X => self.op_tool_x(code),
            OpCode::RESERVED_66 => Err(Trap::InvalidOpcode),
            OpCode::RESERVED_67 => Err(Trap::InvalidOpcode),
        }
    }

    // === Instruction Implementations ===

    fn op_nop(&mut self) -> Result<(), Trap> {
        // No operation
        Ok(())
    }

    fn op_push_i64(&mut self, code: &[u8]) -> Result<(), Trap> {
        let value = read_i64_be(code, &mut (self.pc as usize));
        self.pc += 8;
        self.stack.push(Value::I64(value))
    }

    fn op_push_f64(&mut self, code: &[u8]) -> Result<(), Trap> {
        let value = read_f64_be(code, &mut (self.pc as usize));
        self.pc += 8;
        self.stack.push(Value::F64(value))
    }

    fn op_push_bytes(&mut self, code: &[u8]) -> Result<(), Trap> {
        let len = read_u16_be(code, &mut (self.pc as usize)) as usize;
        if self.pc as usize + len > code.len() {
            return Err(Trap::InvalidOpcode);
        }
        let value = code[self.pc as usize..self.pc as usize + len].to_vec();
        self.pc += len as u32;
        self.stack.push(Value::Bytes(value))
    }

    fn op_pop(&mut self) -> Result<(), Trap> {
        self.stack.pop()?;
        Ok(())
    }

    fn op_dup(&mut self) -> Result<(), Trap> {
        self.stack.dup()
    }

    fn op_swap(&mut self) -> Result<(), Trap> {
        self.stack.swap()
    }

    fn op_push_nil(&mut self) -> Result<(), Trap> {
        self.stack.push(Value::Nil)
    }

    fn op_add(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a.wrapping_add(b)),
            (Value::F64(a), Value::F64(b)) => Value::F64(a + b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_sub(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a.wrapping_sub(b)),
            (Value::F64(a), Value::F64(b)) => Value::F64(a - b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_mul(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a.wrapping_mul(b)),
            (Value::F64(a), Value::F64(b)) => Value::F64(a * b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_div(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(_), Value::I64(0)) => return Err(Trap::DivZero),
            (Value::F64(_), Value::F64(0.0)) => return Err(Trap::DivZero),
            (Value::I64(a), Value::I64(b)) => Value::I64(a.wrapping_div(b)),
            (Value::F64(a), Value::F64(b)) => Value::F64(a / b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_mod(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(_), Value::I64(0)) => return Err(Trap::DivZero),
            (Value::F64(_), Value::F64(0.0)) => return Err(Trap::DivZero),
            (Value::I64(a), Value::I64(b)) => Value::I64(a.wrapping_rem(b)),
            (Value::F64(a), Value::F64(b)) => Value::F64(a % b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_neg(&mut self) -> Result<(), Trap> {
        let a = self.stack.pop()?;
        let result = match a {
            Value::I64(a) => Value::I64(a.wrapping_neg()),
            Value::F64(a) => Value::F64(-a),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_eq(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a == b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a == b { 1 } else { 0 }),
            (Value::Bytes(a), Value::Bytes(b)) => Value::I64(if a == b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_ne(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a != b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a != b { 1 } else { 0 }),
            (Value::Bytes(a), Value::Bytes(b)) => Value::I64(if a != b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_lt(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a < b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a < b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_le(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a <= b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a <= b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_gt(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a > b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a > b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_ge(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(if a >= b { 1 } else { 0 }),
            (Value::F64(a), Value::F64(b)) => Value::I64(if a >= b { 1 } else { 0 }),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_and(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a & b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_or(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a | b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_xor(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a ^ b),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_not(&mut self) -> Result<(), Trap> {
        let a = self.stack.pop()?;
        let result = match a {
            Value::I64(a) => Value::I64(!a),
            _ => return Err(Trap::TypeMismatch),
        };
        self.stack.push(result)
    }

    fn op_bool_and(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = Value::I64(if a.is_true() && b.is_true() { 1 } else { 0 });
        self.stack.push(result)
    }

    fn op_bool_or(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = Value::I64(if a.is_true() || b.is_true() { 1 } else { 0 });
        self.stack.push(result)
    }

    fn op_bool_xor(&mut self) -> Result<(), Trap> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;
        let result = Value::I64(if a.is_true() != b.is_true() { 1 } else { 0 });
        self.stack.push(result)
    }

    fn op_bool_not(&mut self) -> Result<(), Trap> {
        let a = self.stack.pop()?;
        let result = Value::I64(if a.is_true() { 0 } else { 1 });
        self.stack.push(result)
    }

    fn op_jump(&mut self, code: &[u8]) -> Result<(), Trap> {
        let offset = read_i32_be(code, &mut (self.pc as usize));
        let target = self.pc.wrapping_add(offset as u32);
        self.pc = target;
        Ok(())
    }

    fn op_jump_if(&mut self, code: &[u8]) -> Result<(), Trap> {
        let offset = read_i32_be(code, &mut (self.pc as usize));
        let condition = self.stack.pop()?;
        if condition.is_true() {
            let target = self.pc.wrapping_add(offset as u32);
            self.pc = target;
        }
        Ok(())
    }

    fn op_jump_if_not(&mut self, code: &[u8]) -> Result<(), Trap> {
        let offset = read_i32_be(code, &mut (self.pc as usize));
        let condition = self.stack.pop()?;
        if !condition.is_true() {
            let target = self.pc.wrapping_add(offset as u32);
            self.pc = target;
        }
        Ok(())
    }

    fn op_call(&mut self, code: &[u8]) -> Result<(), Trap> {
        // For v1, we don't support modules/functions - treat as reserved
        // In future versions, this will handle module/function calls
        let _module = read_u32_be(code, &mut (self.pc as usize));
        let _function = read_u16_be(code, &mut (self.pc as usize));
        Err(Trap::NotImplemented) // Not implemented in v1
    }

    fn op_return(&mut self) -> Result<(), Trap> {
        // For v1, we don't support modules/functions - treat as reserved
        Err(Trap::NotImplemented) // Not implemented in v1
    }

    fn op_halt(&mut self) -> Result<(), Trap> {
        self.halted = true;
        Ok(())
    }

    fn op_yield(&mut self) -> Result<(), Trap> {
        self.yield_requested = true;
        Ok(())
    }

    fn op_emit(&mut self, code: &[u8]) -> Result<(), Trap> {
        let _event_id = read_u16_be(code, &mut (self.pc as usize));
        // In a real implementation, this would call back to the host
        // For v1, we just acknowledge it
        Ok(())
    }

    fn op_load(&mut self, code: &[u8]) -> Result<(), Trap> {
        let addr = read_u32_be(code, &mut (self.pc as usize));
        // For simplicity, we'll load a pointer-sized value (4 bytes)
        // In a more complete implementation, we'd need type information
        let bytes = self.memory.load(addr, 4)?;
        let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        self.stack.push(Value::Ptr(value))
    }

    fn op_store(&mut self, code: &[u8]) -> Result<(), Trap> {
        let addr = read_u32_be(code, &mut (self.pc as usize));
        let value = self.stack.pop()?;
        let bytes = match value {
            Value::Ptr(ptr) => ptr.to_be_bytes().to_vec(),
            Value::I64(val) => val.to_be_bytes().to_vec(),
            Value::F64(val) => {
                let mut bytes = val.to_be_bytes().to_vec();
                bytes.truncate(4); // Truncate to fit in 32-bit slot
                bytes
            }
            Value::Bytes(val) => val,
            Value::Nil => vec![0; 4],
        };
        self.memory.store(addr, &bytes)
    }

    fn op_alloc(&mut self, code: &[u8]) -> Result<(), Trap> {
        let size = read_u32_be(code, &mut (self.pc as usize));
        let ptr = self.memory.alloc(size)?;
        self.stack.push(Value::Ptr(ptr))
    }

    fn op_free(&mut self) -> Result<(), Trap> {
        let ptr = self.stack.pop()?;
        let ptr_val = ptr.expect_ptr()?;
        self.memory.free(ptr_val)
    }

    fn op_copy(&mut self, code: &[u8]) -> Result<(), Trap> {
        let dst = read_u32_be(code, &mut (self.pc as usize));
        let src = read_u32_be(code, &mut (self.pc as usize));
        let len = read_u32_be(code, &mut (self.pc as usize));
        self.memory.copy(dst, src, len)
    }

    fn op_set(&mut self, code: &[u8]) -> Result<(), Trap> {
        let addr = read_u32_be(code, &mut (self.pc as usize));
        let val = read_u8(code, &mut (self.pc as usize));
        self.memory.set(addr, val)
    }

    fn op_get(&mut self, code: &[u8]) -> Result<(), Trap> {
        let addr = read_u32_be(code, &mut (self.pc as usize));
        let val = self.memory.get(addr)?;
        self.stack.push(Value::I64(val as i64))
    }

    fn op_vec_pull(&mut self, code: &[u8]) -> Result<(), Trap> {
        // For v1, we'll treat this as a no-op that pushes Nil
        // In future versions, this will interact with vector storage
        let _hash = read_u32_be(code, &mut (self.pc as usize));
        self.stack.push(Value::Nil)
    }

    fn op_vec_push(&mut self, code: &[u8]) -> Result<(), Trap> {
        // For v1, we'll treat this as a no-op that pops and discards
        // In future versions, this will interact with vector storage
        let _hash = read_u32_be(code, &mut (self.pc as usize));
        self.stack.pop()?;
        Ok(())
    }

    fn op_prob_b(&mut self, code: &[u8]) -> Result<(), Trap> {
        let threshold = read_f32_be(code, &mut (self.pc as usize));
        let offset = read_i32_be(code, &mut (self.pc as usize));
        let prob = self.stack.pop()?;
        let prob_f64 = prob.expect_f64()?;
        if prob_f64 > threshold as f64 {
            let target = self.pc.wrapping_add(offset as u32);
            self.pc = target;
        }
        Ok(())
    }

    fn op_snap_s(&mut self) -> Result<(), Trap> {
        // Create a snapshot and return it to the host
        // For v1, we'll just return a placeholder
        // In a real implementation, this would call back to the host with the snapshot data
        self.snapshot_id = Some(0xDEADBEEF); // Placeholder
        Ok(())
    }

    fn op_snap_r(&mut self, code: &[u8]) -> Result<(), Trap> {
        let snapshot_id = read_u32_be(code, &mut (self.pc as usize));
        // For v1, we'll just acknowledge the restore request
        // In a real implementation, this would load snapshot state from host
        self.snapshot_id = Some(snapshot_id);
        Ok(())
    }

    fn op_tool_x(&mut self, code: &[u8]) -> Result<(), Trap> {
        let cap_token = read_u32_be(code, &mut (self.pc as usize));
        let _tool_id = read_u16_be(code, &mut (self.pc as usize));
        // Check capability
        if !self.capabilities.has_capability(cap_token) {
            return Err(Trap::MissingCap);
        }
        // In a real implementation, this would call back to the host to execute the tool
        // For v1, we'll just acknowledge it
        Ok(())
    }

    /// Check if the VM has halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Check if the VM has requested a yield
    pub fn has_yielded(&self) -> bool {
        self.yield_requested
    }

    /// Reset the yield flag
    pub fn reset_yield(&mut self) {
        self.yield_requested = false;
    }

    /// Get the current program counter
    pub fn pc(&self) -> u32 {
        self.pc
    }

    /// Set capabilities for this VM instance
    pub fn set_capabilities(&mut self, capabilities: CapabilityTable) {
        self.capabilities = capabilities;
    }

    /// Get a reference to the stack (for debugging/snapshots)
    pub fn stack(&self) -> &[Value] {
        self.stack.data()
    }

    /// Get a reference to the memory (for debugging/snapshots)
    pub fn memory(&self) -> &[u8] {
        self.memory.data()
    }
}

// Helper functions for reading binary data (moved from opcodes.rs for VM use)
fn read_u16_be(bytes: &[u8], offset: &mut usize) -> u16 {
    let value = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]);
    *offset += 2;
    value
}

fn read_i32_be(bytes: &[u8], offset: &mut usize) -> i32 {
    let value = i32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

fn read_u32_be(bytes: &[u8], offset: &mut usize) -> u32 {
    let value = u32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

fn read_f32_be(bytes: &[u8], offset: &mut usize) -> f32 {
    let value = f32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

fn read_f64_be(bytes: &[u8], offset: &mut usize) -> f64 {
    let value = f64::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
        bytes[*offset + 4],
        bytes[*offset + 5],
        bytes[*offset + 6],
        bytes[*offset + 7],
    ]);
    *offset += 8;
    value
}

fn read_i64_be(bytes: &[u8], offset: &mut usize) -> i64 {
    let value = i64::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
        bytes[*offset + 4],
        bytes[*offset + 5],
        bytes[*offset + 6],
        bytes[*offset + 7],
    ]);
    *offset += 8;
    value
}

fn read_u8(bytes: &[u8], offset: &mut usize) -> u8 {
    let value = bytes[*offset];
    *offset += 1;
    value
}
