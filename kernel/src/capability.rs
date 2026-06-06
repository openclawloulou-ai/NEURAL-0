use std::collections::HashMap;

/// Capability table for managing access rights
#[derive(Debug, Clone, Default)]
pub struct CapabilityTable {
    /// Map from capability ID to capability data
    capabilities: HashMap<u32, Capability>,
    /// Next available capability ID
    next_id: u32,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub kind: u8,
    pub scope: String,
    pub flags: u8,
    pub uses_remaining: Option<u32>, // None = unlimited
    pub expires_at: Option<u64>,     // None = no expiry
}

impl CapabilityTable {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            next_id: 1, // Start at 1 to reserve 0 for invalid
        }
    }

    /// Add a capability and return its ID
    pub fn add_capability(
        &mut self,
        kind: u8,
        scope: String,
        flags: u8,
        uses_remaining: Option<u32>,
        expires_at: Option<u64>,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.capabilities.insert(
            id,
            Capability {
                kind,
                scope,
                flags,
                uses_remaining,
                expires_at,
            },
        );
        id
    }

    /// Check if a capability ID is valid
    pub fn has_capability(&self, id: u32) -> bool {
        self.capabilities.contains_key(&id)
    }

    /// Get capability data by ID
    pub fn get_capability(&self, id: u32) -> Option<&Capability> {
        self.capabilities.get(&id)
    }

    /// Consume a use of the capability (returns false if exhausted)
    pub fn use_capability(&mut self, id: u32) -> bool {
        if let Some(cap) = self.capabilities.get_mut(&id) {
            match cap.uses_remaining {
                Some(0) => false, // Already exhausted
                Some(n) => {
                    cap.uses_remaining = Some(n - 1);
                    true
                }
                None => true, // Unlimited
            }
        } else {
            false // Invalid capability
        }
    }

    /// Check if capability has expired
    pub fn is_expired(&self, id: u32, current_time: u64) -> bool {
        self.capabilities.get(&id).map_or(true, |cap| {
            cap.expires_at.map_or(false, |exp| current_time >= exp)
        })
    }

    /// Clear all capabilities
    pub fn clear(&mut self) {
        self.capabilities.clear();
        self.next_id = 1;
    }

    /// Get number of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}
