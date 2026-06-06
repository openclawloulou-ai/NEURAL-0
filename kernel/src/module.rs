/// Module handling for DAG composition
/// 
/// In v1, we keep this simple - a module is just a blob of code
/// In v2+, this will handle imports, exports, and DAG execution

use crate::vm::VM;
use crate::trap::Trap;

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
        Self { modules: Vec::new() }
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