use crate::trap::Trap;
use crate::value::Value;

/// Stack implementation with bounds checking
#[derive(Debug, Clone, Default)]
pub struct Stack {
    data: Vec<Value>,
    max_size: usize,
}

impl Stack {
    /// Create a new stack with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) -> Result<(), Trap> {
        if self.data.len() >= self.max_size {
            return Err(Trap::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<Value, Trap> {
        self.data.pop().ok_or(Trap::StackUnderflow)
    }

    /// Peek at the top value without popping
    pub fn peek(&self) -> Result<&Value, Trap> {
        self.data.last().ok_or(Trap::StackUnderflow)
    }

    /// Duplicate the top value
    pub fn dup(&mut self) -> Result<(), Trap> {
        let top = self.peek()?.clone();
        self.push(top)
    }

    /// Swap the top two values
    pub fn swap(&mut self) -> Result<(), Trap> {
        if self.data.len() < 2 {
            return Err(Trap::StackUnderflow);
        }
        let len = self.data.len();
        self.data.swap(len - 1, len - 2);
        Ok(())
    }

    /// Get the current stack depth
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get a reference to the underlying data (for snapshots)
    pub fn data(&self) -> &[Value] {
        &self.data
    }

    /// Restore stack from snapshot data
    pub fn restore_from(&mut self, data: &[Value]) -> Result<(), Trap> {
        if data.len() > self.max_size {
            return Err(Trap::StackOverflow);
        }
        self.data = data.to_vec();
        Ok(())
    }
}
