#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum OpCode {
    // Core (0x0001 - 0x000F)
    NOP = 0x0001,
    PUSH_I64 = 0x0002,
    PUSH_F64 = 0x0003,
    PUSH_BYTES = 0x0004,
    POP = 0x0005,
    DUP = 0x0006,
    SWAP = 0x0007,
    PUSH_NIL = 0x0008,

    // Arithmetic (0x0010 - 0x001F)
    ADD = 0x0010,
    SUB = 0x0011,
    MUL = 0x0012,
    DIV = 0x0013,
    MOD = 0x0014,
    NEG = 0x0015,

    // Comparison (0x0020 - 0x002F)
    EQ = 0x0020,
    NE = 0x0021,
    LT = 0x0022,
    LE = 0x0023,
    GT = 0x0024,
    GE = 0x0025,

    // Logic (0x0030 - 0x003F)
    AND = 0x0030,
    OR = 0x0031,
    XOR = 0x0032,
    NOT = 0x0033,
    BOOL_AND = 0x0034,
    BOOL_OR = 0x0035,
    BOOL_XOR = 0x0036,
    BOOL_NOT = 0x0037,

    // Control Flow (0x0040 - 0x004F)
    JUMP = 0x0040,
    JUMP_IF = 0x0041,
    JUMP_IF_NOT = 0x0042,
    CALL = 0x0043,
    RETURN = 0x0044,
    HALT = 0x0045,
    YIELD = 0x0046,
    EMIT = 0x0047,

    // Memory (0x0050 - 0x005F)
    LOAD = 0x0050,
    STORE = 0x0051,
    ALLOC = 0x0052,
    FREE = 0x0053,
    COPY = 0x0054,
    SET = 0x0055,
    GET = 0x0056,

    // Agent-Semantic (0x0060 - 0x006F)
    VEC_PULL = 0x0060,
    VEC_PUSH = 0x0061,
    PROB_B = 0x0062,
    SNAP_S = 0x0063,
    SNAP_R = 0x0064,
    TOOL_X = 0x0065,
    RESERVED_66 = 0x0066,
    RESERVED_67 = 0x0067,
}

impl OpCode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(Self::NOP),
            0x0002 => Some(Self::PUSH_I64),
            0x0003 => Some(Self::PUSH_F64),
            0x0004 => Some(Self::PUSH_BYTES),
            0x0005 => Some(Self::POP),
            0x0006 => Some(Self::DUP),
            0x0007 => Some(Self::SWAP),
            0x0008 => Some(Self::PUSH_NIL),
            0x0010 => Some(Self::ADD),
            0x0011 => Some(Self::SUB),
            0x0012 => Some(Self::MUL),
            0x0013 => Some(Self::DIV),
            0x0014 => Some(Self::MOD),
            0x0015 => Some(Self::NEG),
            0x0020 => Some(Self::EQ),
            0x0021 => Some(Self::NE),
            0x0022 => Some(Self::LT),
            0x0023 => Some(Self::LE),
            0x0024 => Some(Self::GT),
            0x0025 => Some(Self::GE),
            0x0030 => Some(Self::AND),
            0x0031 => Some(Self::OR),
            0x0032 => Some(Self::XOR),
            0x0033 => Some(Self::NOT),
            0x0034 => Some(Self::BOOL_AND),
            0x0035 => Some(Self::BOOL_OR),
            0x0036 => Some(Self::BOOL_XOR),
            0x0037 => Some(Self::BOOL_NOT),
            0x0040 => Some(Self::JUMP),
            0x0041 => Some(Self::JUMP_IF),
            0x0042 => Some(Self::JUMP_IF_NOT),
            0x0043 => Some(Self::CALL),
            0x0044 => Some(Self::RETURN),
            0x0045 => Some(Self::HALT),
            0x0046 => Some(Self::YIELD),
            0x0047 => Some(Self::EMIT),
            0x0050 => Some(Self::LOAD),
            0x0051 => Some(Self::STORE),
            0x0052 => Some(Self::ALLOC),
            0x0053 => Some(Self::FREE),
            0x0054 => Some(Self::COPY),
            0x0055 => Some(Self::SET),
            0x0056 => Some(Self::GET),
            0x0060 => Some(Self::VEC_PULL),
            0x0061 => Some(Self::VEC_PUSH),
            0x0062 => Some(Self::PROB_B),
            0x0063 => Some(Self::SNAP_S),
            0x0064 => Some(Self::SNAP_R),
            0x0065 => Some(Self::TOOL_X),
            0x0066 => Some(Self::RESERVED_66),
            0x0067 => Some(Self::RESERVED_67),
            _ => None,
        }
    }

    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

// Instruction decoding helpers
pub fn read_u16_be<'a>(bytes: &'a [u8], offset: &mut usize) -> u16 {
    let value = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]);
    *offset += 2;
    value
}

pub fn read_i32_be<'a>(bytes: &'a [u8], offset: &mut usize) -> i32 {
    let value = i32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

pub fn read_u32_be<'a>(bytes: &'a [u8], offset: &mut usize) -> u32 {
    let value = u32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

pub fn read_f32_be<'a>(bytes: &'a [u8], offset: &mut usize) -> f32 {
    let value = f32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    value
}

pub fn read_f64_be<'a>(bytes: &'a [u8], offset: &mut usize) -> f64 {
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

pub fn read_i64_be<'a>(bytes: &'a [u8], offset: &mut usize) -> i64 {
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

pub fn read_u8<'a>(bytes: &'a [u8], offset: &mut usize) -> u8 {
    let value = bytes[*offset];
    *offset += 1;
    value
}

pub fn read_bytes<'a>(bytes: &'a [u8], offset: &mut usize, length: usize) -> &'a [u8] {
    let start = *offset;
    *offset += length;
    &bytes[start..start + length]
}

pub fn read_length_prefixed_bytes<'a>(bytes: &'a [u8], offset: &mut usize) -> &'a [u8] {
    let len = read_u16_be(bytes, offset) as usize;
    read_bytes(bytes, offset, len)
}
