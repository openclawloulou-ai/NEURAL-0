use crate::trap::Trap;

/// Linear memory model with bounds checking
#[derive(Debug, Clone, Default)]
pub struct Memory {
    data: Vec<u8>,
    max_size: usize,
}

impl Memory {
    /// Create a new memory with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Load a value from memory
    /// Note: This is a low-level function - callers must know the type and size
    pub fn load(&self, addr: u32, len: usize) -> Result<&[u8], Trap> {
        let addr = addr as usize;
        if addr > self.max_size {
            return Err(Trap::OOBMemory);
        }
        let end = addr + len;
        if end > self.max_size {
            return Err(Trap::OOBMemory);
        }
        Ok(&self.data[addr..end])
    }

    /// Store a value to memory
    pub fn store(&mut self, addr: u32, value: &[u8]) -> Result<(), Trap> {
        let addr = addr as usize;
        if addr > self.max_size {
            return Err(Trap::OOBMemory);
        }
        let end = addr + value.len();
        if end > self.max_size {
            return Err(Trap::OOBMemory);
        }
        self.data[addr..end].copy_from_slice(value);
        Ok(())
    }

    /// Allocate a block of memory (grow the memory slice)
    pub fn alloc(&mut self, size: u32) -> Result<u32, Trap> {
        let size = size as usize;
        if size == 0 {
            return Ok(0); // Null pointer
        }
        let ptr = self.data.len();
        // Check if we'd exceed max size
        if ptr + size > self.max_size {
            return Err(Trap::OOM);
        }
        // Resize to fit the new allocation
        self.data.resize(ptr + size, 0);
        Ok(ptr as u32)
    }

    /// Free a block of memory (in this simple model, we don't actually free,
    /// but we track the allocation for potential future use)
    pub fn free(&mut self, _ptr: u32) -> Result<(), Trap> {
        // In a real implementation, we might maintain a free list
        // For v1, we just check that the pointer is valid
        if _ptr as usize > self.data.len() {
            return Err(Trap::InvalidPointer);
        }
        // Actual freeing is omitted for simplicity in v1
        Ok(())
    }

    /// Copy memory from src to dst
    pub fn copy(&mut self, dst: u32, src: u32, len: u32) -> Result<(), Trap> {
        let src_slice = self.load(src, len as usize)?.to_vec();
        self.store(dst, &src_slice)
    }

    /// Set a single byte in memory
    pub fn set(&mut self, addr: u32, value: u8) -> Result<(), Trap> {
        let addr = addr as usize;
        if addr >= self.max_size {
            return Err(Trap::OOBMemory);
        }
        self.data[addr] = value;
        Ok(())
    }

    /// Get a single byte from memory
    pub fn get(&self, addr: u32) -> Result<u8, Trap> {
        let addr = addr as usize;
        if addr >= self.max_size {
            return Err(Trap::OOBMemory);
        }
        Ok(self.data[addr])
    }

    /// Get the current memory size (allocated bytes)
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get a reference to the underlying data (for snapshots)
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Restore memory from snapshot data
    pub fn restore_from(&mut self, data: &[u8]) -> Result<(), Trap> {
        if data.len() > self.max_size {
            return Err(Trap::OOM);
        }
        self.data = data.to_vec();
        Ok(())
    }

    /// Get the maximum memory size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}
