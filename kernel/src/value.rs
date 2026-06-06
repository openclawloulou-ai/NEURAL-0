use crate::trap::Trap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bytes(Vec<u8>),
    Ptr(u32),
    Nil,
}

impl Value {
    pub fn tag(&self) -> u8 {
        match self {
            Value::I64(_) => 0x01,
            Value::F64(_) => 0x02,
            Value::Bytes(_) => 0x03,
            Value::Ptr(_) => 0x04,
            Value::Nil => 0x05,
        }
    }

    pub fn from_tagged(tag: u8, bytes: &[u8]) -> Option<(Self, usize)> {
        let mut offset = 0;
        let value = match tag {
            0x01 => {
                if bytes.len() < 8 {
                    return None;
                }
                let val = i64::from_be_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                    bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7],
                ]);
                offset += 8;
                Value::I64(val)
            }
            0x02 => {
                if bytes.len() < 8 {
                    return None;
                }
                let val = f64::from_be_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                    bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7],
                ]);
                offset += 8;
                Value::F64(val)
            }
            0x03 => {
                if bytes.len() < 2 {
                    return None;
                }
                let len = u16::from_be_bytes([bytes[offset], bytes[offset+1]]) as usize;
                offset += 2;
                if bytes.len() < offset + len {
                    return None;
                }
                let val = bytes[offset..offset+len].to_vec();
                offset += len;
                Value::Bytes(val)
            }
            0x04 => {
                if bytes.len() < 4 {
                    return None;
                }
                let val = u32::from_be_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                ]);
                offset += 4;
                Value::Ptr(val)
            }
            0x05 => {
                Value::Nil
            }
            _ => return None,
        };
        Some((value, offset))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(self.tag());
        match self {
            Value::I64(val) => result.extend_from_slice(&val.to_be_bytes()),
            Value::F64(val) => result.extend_from_slice(&val.to_be_bytes()),
            Value::Bytes(val) => {
                result.extend_from_slice(&(val.len() as u16).to_be_bytes());
                result.extend_from_slice(val);
            }
            Value::Ptr(val) => result.extend_from_slice(&val.to_be_bytes()),
            Value::Nil => {}
        }
        result
    }

    pub fn size(&self) -> usize {
        1 + match self {
            Value::I64(_) => 8,
            Value::F64(_) => 8,
            Value::Bytes(val) => 2 + val.len(),
            Value::Ptr(_) => 4,
            Value::Nil => 0,
        }
    }

    // Type checking helpers for operations
    pub fn expect_i64(self) -> Result<i64, Trap> {
        match self {
            Value::I64(val) => Ok(val),
            _ => Err(Trap::TypeMismatch),
        }
    }

    pub fn expect_f64(self) -> Result<f64, Trap> {
        match self {
            Value::F64(val) => Ok(val),
            _ => Err(Trap::TypeMismatch),
        }
    }

    pub fn expect_bytes(self) -> Result<Vec<u8>, Trap> {
        match self {
            Value::Bytes(val) => Ok(val),
            _ => Err(Trap::TypeMismatch),
        }
    }

    pub fn expect_ptr(self) -> Result<u32, Trap> {
        match self {
            Value::Ptr(val) => Ok(val),
            _ => Err(Trap::TypeMismatch),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::I64(_) | Value::F64(_))
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::I64(val) => *val != 0,
            Value::F64(val) => *val != 0.0,
            Value::Bytes(val) => !val.is_empty(),
            Value::Ptr(_) => true, // Non-null pointer is true
            Value::Nil => false,
        }
    }
}