/// Trap types for VM errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    // Stack errors
    StackOverflow,
    StackUnderflow,

    // Arithmetic errors
    DivZero,

    // Type errors
    TypeMismatch,

    // Memory errors
    OOBMemory, // Out of bounds memory access
    OOM,       // Out of memory
    InvalidPointer,

    // Instruction errors
    InvalidOpcode,

    // Capability errors
    MissingCap,
    CapExpired,
    CapExhausted,

    // Snapshot errors
    SnapshotInvalid,

    // Not yet implemented
    NotImplemented,
}

impl std::fmt::Display for Trap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trap::StackOverflow => write!(f, "Stack overflow"),
            Trap::StackUnderflow => write!(f, "Stack underflow"),
            Trap::DivZero => write!(f, "Division by zero"),
            Trap::TypeMismatch => write!(f, "Type mismatch"),
            Trap::OOBMemory => write!(f, "Out of bounds memory access"),
            Trap::OOM => write!(f, "Out of memory"),
            Trap::InvalidPointer => write!(f, "Invalid pointer"),
            Trap::InvalidOpcode => write!(f, "Invalid opcode"),
            Trap::MissingCap => write!(f, "Missing capability"),
            Trap::CapExpired => write!(f, "Capability expired"),
            Trap::CapExhausted => write!(f, "Capability exhausted"),
            Trap::SnapshotInvalid => write!(f, "Invalid snapshot"),
            Trap::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl std::error::Error for Trap {}
