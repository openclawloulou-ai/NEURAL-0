# NEURAL-0 Snapshot Engine Specification

## Overview
The NEURAL-0 snapshot mechanism allows an agent to save its entire execution state and resume later exactly where it left off. This is a fundamental primitive for long-running agent workflows, pause/resume scenarios, and checkpointing for evolutionary algorithms.

A snapshot captures:
- Program counter (next instruction to execute)
- Complete stack contents (all tagged values)
- Complete linear memory contents
- Current capability table
- Vector storage hashes (note: actual vector content is external)
- Timestamp and metadata

Snapshots are content-addressed binary blobs that can be stored anywhere (disk, database, object store) and restored by their identifier.

## Snapshot Binary Format

```
+-------------------------+
| Magic: 0x4E53           |  2 bytes ("NS" for N0-Snapshot)
| Version: u16            |  2 bytes (0x0001 for v1.0.0)
| Timestamp: u64          |  8 bytes (Unix nanoseconds since epoch)
| Module ID: u32          |  4 bytes (ID of module that executed SNAP_S)
| Program Counter: u32    |  4 bytes (offset of next instruction to execute)
| Stack Length: u32       |  4 bytes (number of tagged values on stack)
| Stack: [Tagged Value]   |  Variable (each value: u8:tag + value bytes)
| Memory Length: u32      |  4 bytes (size of linear memory in bytes)
| Memory: [u8]            |  Variable (raw memory contents)
| Capability Count: u8    |  1 byte (number of capability tokens)
| Capabilities: [Token]   |  Variable (each: 32 bytes as per CAPABILITIES.md)
| Vector Count: u8        |  1 byte (number of vector hashes stored)
| Vector Entries: [       |  Variable
|   Hash: u32             |    4 bytes (first 4 bytes of SHA-256 hash)
|   Kind: u8              |    1 byte (0x06 for VECTOR_STORE)
|   Scope Len: u8         |    1 byte
|   Scope: [bytes]        |    Variable length UTF-8
| ]*
| Reserved: u4            |  4 bytes (future use, must be 0)
| Checksum: u32           |  4 bytes (CRC32 of everything before this field)
+-------------------------+
```

### Field Details

**Magic**: Always 0x4E53 (ASCII "NS") - identifies the blob as a NEURAL-0 snapshot

**Version**: Snapshot format version (current: 0x0001)

**Timestamp**: When the snapshot was taken, in Unix nanoseconds. Provides monotonic ordering and age calculation.

**Module ID**: The ID of the currently executing module when SNAP_S was called. Helps validate restore compatibility.

**Program Counter**: The offset of the next instruction to execute (i.e., the address after the SNAP_S instruction). On restore, VM sets PC to this value.

**Stack**: 
- Stack Length: Number of tagged values currently on the stack (0 to max stack depth)
- Each Tagged Value: 
  - 1 byte: Type tag (from ISA.md)
  - Value bytes: 
    - I64: 8 bytes little-endian
    - F64: 8 bytes little-endian
    - Bytes: U16 length prefix followed by that many bytes
    - Ptr: 4 bytes little-endian offset
    - Nil: 0 bytes (just the tag)

**Memory**: Raw contents of the linear memory array. Length indicates how many bytes are currently allocated (via ALLOC). The VM grows memory to this size on restore.

**Capabilities**: Exact copy of the capability table at snapshot time. Each token is 32 bytes as defined in CAPABILITIES.md.

**Vector Entries**: Rather than snapshotting potentially large vector contents, we snapshot only the references:
- Vector Count: Number of vector entries (0-255)
- For each entry:
  - Hash: First 4 bytes of the SHA-256 hash of the vector content (used for lookup)
  - Kind: Must be 0x06 (VECTOR_STORE) - reserved for future vector types
  - Scope Len: Length of the scope string
  - Scope: The scope string used when the vector was accessed

**Reserved**: Future expansion space (must be zero in v1)

**Checksum**: CRC32 checksum of all preceding bytes (from Magic through Reserved). Used to detect corruption.

## Snapshot Semantics

### SNAP_S Instruction
When executed:
1. VM pauses normal instruction execution
2. Collects all state as described above
3. Serializes to binary format
4. Computes CRC32 checksum
5. Returns the binary blob to the host via an implementation-specific mechanism (callback, return value, etc.)
6. Resumes normal execution (the instruction after SNAP_S continues normally)

**Important**: SNAP_S does not halt or yield - it's a side-effect instruction that continues execution after taking the snapshot.

### SNAP_R Instruction
When executed with operand `u32:id`:
1. VM looks up snapshot blob by ID (mechanism is host-dependent)
2. Validates the blob:
   - Checks magic bytes
   - Checks version is supported
   - Verifies CRC32 checksum
   - Optionally validates Module ID matches current module
3. Deserializes the state:
   - Sets program counter to saved value
   - Replaces stack with saved values (after type validation)
   - Replaces linear memory with saved contents (after size validation)
   - Replaces capability table with saved tokens
   - Note: Vector storage is NOT restored - the hashes are just for reference
4. Continues execution at the restored program counter

If validation fails, the VM traps with `SNAPSHOT_INVALID`.

## Vector Storage Interaction

Snapshots do NOT capture actual vector storage contents for several reasons:
1. Vectors can be arbitrarily large (GBs or more)
2. Vector storage is often external (Redis, database, etc.)
3. The semantic meaning is in the hash, not necessarily the bytes

Instead, snapshots record the vector hashes that were accessible at snapshot time. On restore:
- The VM notes which vectors were previously accessible
- The host may choose to prefetch or validate those vectors
- Future VEC_PULL/PUSH operations use the same hash/scope mechanism
- If a vector has been deleted or modified, VEC_PULL will fail/not find it

This approach makes snapshots lightweight while preserving the semantic intent.

## Host Integration

The host is responsible for:
1. Providing storage for snapshot blobs (filesystem, S3, database, etc.)
2. Implementing the lookup mechanism for SNAP_R by ID
3. Deciding retention policy (how long to keep snapshots)
4. Optionally validating snapshots before restore (e.g., checking module compatibility)
5. Managing vector storage that backs the hashes

A simple interface:
```rust
trait SnapshotStore {
    fn store_snapshot(&self, data: &[u8]) -> SnapshotId;
    fn load_snapshot(&self, id: &SnapshotId) -> Option<Vec<u8>>;
    fn delete_snapshot(&self, id: &SnapshotId) -> bool;
}
```

Where `SnapshotId` is typically the SHA-256 hash of the snapshot blob (first 4 bytes used in the vector reference optimization).

## Security Considerations

Snapshots contain the complete execution state, including:
- Potentially sensitive data on the stack and in memory
- Capability tokens (which grant access to resources)
- Program counter (revealing execution progress)

Therefore:
- Snapshots should be treated as sensitive data
- Storage should enforce access controls
- Transmission should be encrypted if over network
- Hosts may choose to encrypt snapshots at rest
- The VM itself does not encrypt snapshots - security is host responsibility

## Usage Examples

### Simple Pause/Resume
```
; Agent does some work
PUSH_I64 42
; ... more work ...
SNAP_S          ; Take snapshot, continue execution
; ... more work ...
HALT
```
Host receives snapshot blob, stores it. Later can restore to resume from after the SNAP_S.

### Checkpoint for Evolutionary Algorithm
In the fitness evaluation loop:
```
; Load test case
TOOL_X [FILE_READ capability] "load_test_case"
; Run algorithm
; ... lots of computation ...
SNAP_S          ; Periodic checkpoint
; Check if time's up
TOOL_X [TIMER capability] "check_timeout"
JUMP_IF_NOT done_checkpoint
; ... continue ...
done_checkpoint:
HALT
```
Host stores snapshots periodically. If evaluation times out, best snapshot can be restored and continued.

### Vector Storage Interaction
```
; Compute some embedding
; ... (complex operation leaving f64 vector on stack) ...
VEC_PUSH "my_embeddings"   ; Store by hash
; Later...
VEC_PULL "my_embeddings"   ; Retrieve by same hash
```
Snapshot will record that "my_embeddings" hash was accessible, but not the actual vector bytes.