use crate::capability::CapabilityTable;
use crate::memory::Memory;
use crate::stack::Stack;
use crate::trap::Trap;
use crate::value::Value;

/// Snapshot engine for saving and restoring VM state
pub struct Snapshot;

impl Snapshot {
    /// Magic number for snapshot validation
    pub const MAGIC: u32 = 0x4E53; // "NS" in ASCII
    /// Current snapshot format version
    pub const VERSION: u16 = 0x0001;
}

#[derive(Debug, Clone, Default)]
pub struct SnapshotData {
    pub timestamp: u64, // Unix nanoseconds
    pub module_id: u32,
    pub program_counter: u32,
    pub stack: Vec<Value>,
    pub memory: Vec<u8>,
    pub capabilities: CapabilityTable,
    pub vector_refs: Vec<(u32, String)>, // (hash, scope)
}

impl Snapshot {
    /// Create a snapshot from VM state
    pub fn create(
        timestamp: u64,
        module_id: u32,
        pc: u32,
        stack: &Stack,
        memory: &Memory,
        capabilities: &CapabilityTable,
        vector_refs: Vec<(u32, String)>,
    ) -> SnapshotData {
        SnapshotData {
            timestamp,
            module_id,
            program_counter: pc,
            stack: stack.data().to_vec(),
            memory: memory.data().to_vec(),
            capabilities: capabilities.clone(),
            vector_refs,
        }
    }

    /// Serialize snapshot data to binary format with CRC32 checksum
    pub fn serialize(data: &SnapshotData) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Magic and version
        buffer.extend_from_slice(&Self::MAGIC.to_be_bytes());
        buffer.extend_from_slice(&Self::VERSION.to_be_bytes());

        // Timestamp (8 bytes)
        buffer.extend_from_slice(&data.timestamp.to_be_bytes());
        // Module ID (4 bytes)
        buffer.extend_from_slice(&data.module_id.to_be_bytes());
        // Program Counter (4 bytes)
        buffer.extend_from_slice(&data.program_counter.to_be_bytes());

        // Stack length and data
        buffer.extend_from_slice(&(data.stack.len() as u32).to_be_bytes());
        for val in &data.stack {
            let bytes = val.to_bytes();
            buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            buffer.extend_from_slice(&bytes);
        }

        // Memory length and data
        buffer.extend_from_slice(&(data.memory.len() as u32).to_be_bytes());
        buffer.extend_from_slice(&data.memory);

        // Capabilities: just store count for v1 (full serialization is complex)
        let cap_count = data.capabilities.len() as u32;
        buffer.extend_from_slice(&cap_count.to_be_bytes());

        // Vector refs length and data
        buffer.extend_from_slice(&(data.vector_refs.len() as u32).to_be_bytes());
        for (hash, scope) in &data.vector_refs {
            buffer.extend_from_slice(&hash.to_be_bytes());
            let scope_bytes = scope.as_bytes();
            buffer.extend_from_slice(&(scope_bytes.len() as u32).to_be_bytes());
            buffer.extend_from_slice(scope_bytes);
        }

        // Calculate CRC32 of the payload so far
        let crc = crc32fast::hash(&buffer);
        buffer.extend_from_slice(&crc.to_be_bytes());

        buffer
    }

    /// Deserialize binary data into snapshot, verifying CRC32 checksum
    pub fn deserialize(data: &[u8]) -> Result<SnapshotData, Trap> {
        if data.len() < 20 {
            return Err(Trap::SnapshotInvalid);
        }

        // Verify CRC32 first
        let payload_len = data.len() - 4;
        let stored_crc = u32::from_be_bytes([
            data[payload_len],
            data[payload_len + 1],
            data[payload_len + 2],
            data[payload_len + 3],
        ]);
        let calculated_crc = crc32fast::hash(&data[..payload_len]);

        if stored_crc != calculated_crc {
            return Err(Trap::SnapshotInvalid);
        }

        let mut offset = 0;

        // Magic
        let magic = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        if magic != Self::MAGIC {
            return Err(Trap::SnapshotInvalid);
        }

        // Version
        let version = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        if version != Self::VERSION {
            return Err(Trap::SnapshotInvalid);
        }

        // Timestamp
        let timestamp = u64::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Module ID
        let module_id = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Program Counter
        let program_counter = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Stack
        let stack_len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let mut stack = Vec::with_capacity(stack_len);
        for _ in 0..stack_len {
            let val_len = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;
            let val_bytes = &data[offset..offset + val_len];
            offset += val_len;

            // Parse Value from tagged bytes
            if let Some((val, _)) = Value::from_tagged(val_bytes[0], &val_bytes[1..]) {
                stack.push(val);
            } else {
                return Err(Trap::SnapshotInvalid);
            }
        }

        // Memory
        let mem_len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let memory = data[offset..offset + mem_len].to_vec();
        offset += mem_len;

        // Capabilities (skip for v1 - just read the count)
        let _cap_count = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        let capabilities = CapabilityTable::new();

        // Vector refs
        let ref_count = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let mut vector_refs = Vec::with_capacity(ref_count);
        for _ in 0..ref_count {
            let hash = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            let scope_len = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;
            let scope = String::from_utf8(data[offset..offset + scope_len].to_vec())
                .map_err(|_| Trap::SnapshotInvalid)?;
            offset += scope_len;
            vector_refs.push((hash, scope));
        }

        Ok(SnapshotData {
            timestamp,
            module_id,
            program_counter,
            stack,
            memory,
            capabilities,
            vector_refs,
        })
    }

    /// Apply snapshot data to VM state
    pub fn apply_to(
        data: &SnapshotData,
        stack: &mut Stack,
        memory: &mut Memory,
        capabilities: &mut CapabilityTable,
    ) -> Result<(), Trap> {
        stack.restore_from(&data.stack)?;
        memory.restore_from(&data.memory)?;
        *capabilities = data.capabilities.clone();
        Ok(())
    }
}
