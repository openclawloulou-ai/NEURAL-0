# NEURAL-0 Binary Format Specification

## Module Binary Format (.n0b)

Each NEURAL-0 module is a self-contained binary blob with the following structure:

```
+-------------------+
| Magic: 0x4E30     |  2 bytes ("N0" in ASCII)
| Version: u16      |  2 bytes (0x0001 for v1.0.0)
| Module ID: u32    |  4 bytes (unique identifier, typically hash)
| Flags: u16        |  2 bytes (bit 0: has_ports, bit 1: has_state)
+-------------------+
| Port Table        |  (present if has_ports flag is set)
|   Input Count: u8 |
|   [Input Types]   |  InputCount * u8 (type tags)
|   Output Count: u8|
|   [Output Types]  |  OutputCount * u8 (type tags)
+-------------------+
| Code Length: u32   |  4 bytes (number of bytes in code section)
| Code: [u8]        |  Variable length (the actual instruction stream)
+-------------------+
| Data Length: u32   |  4 bytes (number of bytes in data/constants section)
| Data: [u8]        |  Variable length (constants pool)
+-------------------+
```

### Field Details

**Magic**: Always 0x4E30 (ASCII "N0") - identifies the file as a NEURAL-0 module

**Version**: Current version is 0x0001. Future versions will increment this.

**Module ID**: A 32-bit identifier for the module. In practice, this should be a hash of the module's contents (excluding this field) to enable content-addressed storage.

**Flags**: Bitfield:
- Bit 0 (0x01): Set if the module has input/output ports declared
- Bit 1 (0x02): Set if the module declares persistent state (for SNAP_S/SNAP_R)
- Bits 2-15: Reserved for future use (must be 0)

**Port Table**: Only present if has_ports flag is set. Declares the module's interface:
- Input Count: Number of input ports (0-255)
- Input Types: Array of type tags (from ISA.md) for each input
- Output Count: Number of output ports (0-255)
- Output Types: Array of type tags for each output

**Code Length**: Length of the instruction stream in bytes

**Code**: The actual NEURAL-0 bytecode instructions, encoded as specified in ISA.md

**Data Length**: Length of the constants/data pool in bytes

**Data**: Constants pool referenced by instructions. Format:
- U16 length prefix followed by the bytes for string/byte data
- I64 values stored directly as 8 bytes little-endian (note: internal representation is little-endian despite network byte order for multi-byte values in the stream)
- F64 values stored directly as 8 bytes little-endian
- Note: The VM is responsible for interpreting these based on how instructions reference them

## DAG Manifest Format (.n0dag)

When modules are composed into a directed acyclic graph, a manifest describes the connections:

```
+-------------------+
| Magic: 0x4E44     |  2 bytes ("ND" for N0-DAG)
| Version: u16      |  2 bytes (0x0001 for v1.0.0)
| Module Count: u16 |  2 bytes
| Edge Count: u16   |  2 bytes
| [Module Entries]  |  ModuleCount * (u32:module_id + u32:offset)
| [Edges]           |  EdgeCount * (u32:src_module + u8:src_port + u32:dst_module + u8:dst_port)
+-------------------+
```

### Field Details

**Magic**: Always 0x4E44 (ASCII "ND") - identifies the file as a NEURAL-0 DAG manifest

**Version**: Manifest format version (current: 0x0001)

**Module Count**: Number of modules in this DAG (0-65535)

**Edge Count**: Number of connections between modules (0-65535)

**Module Entries**: Array specifying where each module's binary blob is located within this manifest:
- Module ID: The module's identifier (must match the ID in the module's header)
- Offset: Byte offset from start of manifest where the module's binary blob begins

**Edges**: Array defining the dataflow connections:
- Src Module: Module ID of the source module
- Src Port: Output port index on the source module (0-255)
- Dst Module: Module ID of the destination module
- Dst Port: Input port index on the destination module (0-255)

### Important Notes

1. **Endianness**: All multi-byte values in the binary format are big-endian (network byte order) except where noted for internal representation
2. **Alignment**: No padding is used between fields - the format is tightly packed
3. **Validation**: A validating loader should:
   - Check magic bytes
   - Verify version is supported
   - Ensure module IDs in manifest match those in module headers
   - Check that port types match on connected edges
   - Verify no cycles exist in the DAG (though runtime may trap on infinite loops)
4. **Extensibility**: Future versions can add new optional sections by defining new flag bits and extending the format after the Data section
5. **Module ID Generation**: Recommended approach is SHA-256 hash of the module's binary contents (excluding the Module ID field itself) truncated to 32 bits