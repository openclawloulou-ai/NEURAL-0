# NEURAL-0 Capability-Based Security Model

## Overview
NEURAL-0 uses a capability-based security model to ensure safe execution of agent-generated code. The VM has no inherent privileges - all access to external resources must be explicitly granted via capability tokens injected by the host before execution.

This model provides:
- **Principle of Least Privilege**: Code only gets exactly the capabilities it needs
- **No Confused Deputy Problems**: Capabilities are unforgeable references
- **Runtime Revocability**: Host can disable capabilities at any time
- **Auditability**: Clear trace of what resources a module can access

## Capability Token Structure

Each capability token is a 32-byte structure passed to the VM at initialization:

```
+-------------------+
| Kind: u8          |  1 byte (capability type identifier)
| Scope Len: u8     |  1 byte (length of scope string)
| Scope: [bytes]    |  Variable length UTF-8 string
| Flags: u8         |  1 byte (bit 0: read, bit 1: write, etc. - kind-specific)
| Uses Remaining: u32| 4 bytes (0xFFFFFFFF = unlimited)
| Expires At: u64   |  8 bytes (Unix timestamp, 0 = no expiry)
| Reserved: u4      |  4 bytes (future use, must be 0)
+-------------------+
```

### Kind Values
| Value | Name | Scope Format | Flags Meaning |
|-------|------|--------------|---------------|
| 0x01 | FILE_READ | Filesystem path (glob patterns allowed) | Bit 0: read, Bit 1: recursive |
| 0x02 | FILE_WRITE | Filesystem path (glob patterns allowed) | Bit 0: create, Bit 1: truncate, Bit 2: append |
| 0x03 | NETWORK_HOST | Hostname or IP:port | Bit 0: TCP connect, Bit 1: UDP send |
| 0x04 | NETWORK_RESOLVE | - | DNS resolution (no scope needed) |
| 0x05 | PROCESS_SPAWN | Executable path or name | Bit 0: inherit env, Bit 1: use sandbox |
| 0x06 | VECTOR_STORE | Collection name or UUID | Bit 0: read, Bit 1: write |
| 0x07 | EVENT_EMIT | Event namespace | Bit 0: emit, Bit 1: subscribe |
| 0x08-0xFF | RESERVED | - | For future capabilities |

### Scope Interpretation
- **FILE_READ/FILE_WRITE**: POSIX path, supports `*` and `?` wildcards, `**` for recursive
- **NETWORK_HOST**: Format `hostname:port` or `[ipv6]:port`, port can be `*` for any
- **VECTOR_STORE**: Logical collection name (maps to actual storage backend)
- **EVENT_EMIT**: Dot-separated namespace (e.g., "system.metrics", "agent.events")

### Flags Interpretation
Flags are capability-kind specific as shown in the table above. Unused bits must be zero.

### Uses Remaining
- `0x00000000`: Capability is exhausted (no more uses allowed)
- `0x00000001` to `0xFFFFFFFE`: Number of remaining uses
- `0xFFFFFFFF`: Unlimited uses (common for read-only capabilities)

### Expires At
- `0`: No expiration timestamp
- Non-zero: Unix epoch timestamp (seconds) when capability becomes invalid

## Capability Enforcement in the VM

When a `TOOL_X` instruction executes:

1. **Lookup**: VM looks up the capability token by `u32:cap_token` operand in its capability table
2. **Validation**:
   - If not found → Trap with `MISSING_CAP`
   - If current time > Expires At → Trap with `CAP_EXPIRED`
   - If Uses Remaining = 0 → Trap with `CAP_EXHAUSTED`
3. **Scope Check**: VM validates the requested operation against the scope:
   - For FILE operations: Checks if requested path matches scope globs
   - For NETWORK: Checks if requested host:port matches scope
   - For VECTOR_STORE: Checks if collection name matches
   - For EVENT_EMIT: Checks if event namespace matches or is sub-namespace
4. **Permission Check**: VM checks if the specific operation is allowed by flags:
   - E.g., attempting to write with a read-only FILE capability traps
5. **Execution**: If all checks pass:
   - Decrement Uses Remaining (if not unlimited)
   - Perform the operation via host callback
   - Return result to stack (or trap on failure)

## Host Interface

The host must provide callbacks for each capability kind:

```rust
trait CapabilityHost {
    fn file_read(&self, path: &str) -> Result<Vec<u8>, HostError>;
    fn file_write(&self, path: &str, data: Vec<u8>, append: bool) -> Result<(), HostError>;
    fn network_connect(&self, host: &str, port: u16) -> Result<NetworkStream, HostError>;
    fn network_resolve(&self, host: &str) -> Result<Vec<std::net::IpAddr>, HostError>;
    fn process_spawn(&self, cmd: &str, args: &[String]) -> Result<ChildProcess, HostError>;
    fn vector_read(&self, collection: &str, key: &[u8]) -> Result<Option<Vec<u8>>, HostError>;
    fn vector_write(&self, collection: &str, key: &[u8], value: Vec<u8>) -> Result<(), HostError>;
    fn event_emit(&self, namespace: &str, event: Vec<u8>) -> Result<(), HostError>;
}
```

Errors from these callbacks result in a `TOOL_FAILED` trap with the error code.

## Default Capability Policy

A secure host should:
1. Start with an empty capability table
2. Only grant capabilities explicitly requested/approved
3. Prefer read-only capabilities when possible
4. Use narrow scopes (e.g., `/tmp/workspace/` instead of `/`)
5. Set reasonable use limits and expiry times
6. Audit capability grants regularly

## Example Capability Tokens

### Read-only file access to /tmp/data/*.txt
```
Kind: 0x01 (FILE_READ)
Scope Len: 16
Scope: "/tmp/data/*.txt"
Flags: 0x01 (read only)
Uses Remaining: 0xFFFFFFFF (unlimited)
Expires At: 0 (no expiry)
```

### One-time HTTP API call to api.example.com:443
```
Kind: 0x03 (NETWORK_HOST)
Scope Len: 21
Scope: "api.example.com:443"
Flags: 0x01 (TCP connect)
Uses Remaining: 0x00000001 (one use)
Expires At: <current_time + 300> (5 minutes)
```

### Write access to vector store "agent_memory"
```
Kind: 0x06 (VECTOR_STORE)
Scope Len: 13
Scope: "agent_memory"
Flags: 0x02 (write only)
Uses Remaining: 0xFFFFFFFF (unlimited)
Expires At: 0 (no expiry)
```

## Interaction with SNAP_S/SNAP_R

When a snapshot is taken:
- The capability table is included in the snapshot binary
- Restoring a snapshot restores the exact capability set at snapshot time
- The host may choose to validate or modify capabilities upon restore
- Expires At and Uses Remaining values are preserved exactly

This ensures that snapshotted code resumes with the same authority it had when snapped.