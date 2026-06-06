use crate::trap::Trap;
/// Module handling for DAG composition
///
/// In v1, we keep this simple - a module is just a blob of code
/// In v2+, this will handle imports, exports, and DAG execution
use crate::vm::VM;

pub const MODULE_MAGIC: [u8; 2] = [0x4E, 0x30];
pub const MODULE_VERSION: u16 = 0x0001;

pub fn extract_code(binary: &[u8]) -> Result<&[u8], String> {
    if binary.len() < 2 || binary[0..2] != MODULE_MAGIC {
        return Err("missing N0 module magic (expected 0x4E30)".to_string());
    }
    if binary.len() < 10 {
        return Err("module header is truncated".to_string());
    }
    let version = u16::from_be_bytes([binary[2], binary[3]]);
    if version != MODULE_VERSION {
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

#[derive(Debug, Clone)]
pub struct Module {
    pub id: u32,
    pub code: Vec<u8>,
    // In future versions: inputs, outputs, etc.
}

impl Module {
    pub fn new(id: u32, code: Vec<u8>) -> Self {
        Self { id, code }
    }

    /// Execute this module in the given VM
    pub fn execute(&self, vm: &mut VM) -> Result<(), Trap> {
        vm.load_and_run(&self.code)
    }
}

/// A simple DAG executor for v1 (just runs modules in sequence)
/// In v2+, this will handle proper DAG topological sorting
pub struct SimpleExecutor {
    modules: Vec<Module>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }

    pub fn execute_all(&self, vm: &mut VM) -> Result<(), Trap> {
        for module in &self.modules {
            module.execute(vm)?;
            if vm.is_halted() || vm.has_yielded() {
                break;
            }
        }
        Ok(())
    }
}

impl Default for SimpleExecutor {
    fn default() -> Self {
        Self::new()
    }
}
