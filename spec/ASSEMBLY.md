# NEURAL-0 Assembly Language (N0ASM)

## Overview
NEURAL-0 Assembly Language is a human-readable text format for writing NEURAL-0 programs. While NEURAL-0 is designed to be agent-native (optimized for LLM token efficiency rather than human readability), the assembly format serves as an important bridge:

1. **LLM-Friendly**: Text format is easier for LLMs to generate and verify than raw binary
2. **Debuggable**: Humans can read and understand assembly when needed
3. **Lossless**: There is a 1:1 mapping between assembly instructions and binary opcodes
4. **Teachable**: Provides a gentle introduction to the ISA before tackling binary

The assembler converts `.n0asm` files to `.n0b` binary blobs. The disassembler does the reverse.

## File Format

- **Encoding**: UTF-8 text
- **Line Endings**: LF (Unix-style) preferred, but CRLF also accepted
- **Comments**: Semicolon (`;`) to end of line
- **Whitespace**: Spaces and tabs ignored except as delimiters
- **Instructions**: One per line (labels and directives may share lines)

## Instruction Syntax

Each line contains:
```
[label:] [mnemonic] [operands] [; comment]
```

### Labels
- Optional identifier followed by colon (`:`)
- Must start with letter or underscore, followed by letters, digits, underscore
- Case-sensitive
- Used as jump targets (resolved to PC offset by assembler)
- Example: `loop_start:`

### Mnemonics
- Case-insensitive (convention: UPPER_CASE)
- Must match exactly one from ISA.md
- Example: `ADD`, `jump_if`, `PUSH_I64`

### Operands
- Zero or more, separated by spaces
- Types:
  - **Immediate numbers**: Decimal (42), hexadecimal (0x2A), binary (0b101010)
  - **Float literals**: Decimal with point (3.14) or scientific (1e-5)
  - **String literals**: Double-quoted (`"hello"`) - becomes PUSH_BYTES
  - **Byte literals**: Single-quoted (`'A'`) - becomes PUSH_I64 65
  - **Labels**: Resolved to address (for JUMP, CALL) or treated as immediate (for data)
  - **Special**: `$` = current PC address

### Examples
```
; Simple addition
PUSH_I64 23
PUSH_I64 19
ADD
HALT

; With labels
start:
    PUSH_I64 10
    PUSH_I64 5
    SUB
    JUMP_IF zero_result
    PUSH_I64 1
    JUMP end
zero_result:
    PUSH_I64 0
end:
    HALT
```

## Directive Summary

While most programs are just instructions, a few directives help with organization:

### .module id
Declare the module ID (32-bit unsigned integer)
```
.module 0x1A2B3C4D
```
If omitted, assembler may generate a hash or use 0.

### .ports input_count output_count
Declare input/output port count for DAG composition
```
.ports 2 1   ; 2 inputs, 1 output
```
Requires corresponding `.input` and `.output` directives to specify types.

### .input type [, type]*
Specify types for input ports (in order)
```
.input I64, F64   ; First input I64, second input F64
```

### .output type [, type]*
Specify types for output ports (in order)
```
.output Bytes   ; Output is byte array
```

### .data
Begin data section (constants pool)
```
.data
msg:      .bytes "Hello, World!"
counter:  .quad 0
fmt:      .bytes "%d\n"
```

### .bytes "string" | hex_bytes
Define byte array data
- Quoted string: UTF-8 encoded
- Hex bytes: Space-separated hex pairs (e.g., `0xDE 0xAD 0xBE 0xEF`)
```
.label: .bytes "debug info"
.pattern: .bytes 0xFF 0x00 0xFF 0x00
```

### .quad value
Define 64-bit integer data
```
.value: .quad 123456789012345
.neg_one: .quad -1
```

### .float value
Define 64-bit float data
```
.pi: .float 3.141592653589793
.zero: .float 0.0
```

### .align n
Align next datum to n-byte boundary (power of 2)
```
.align 8   ; Align to 8-byte boundary
```

## Instruction Reference

Following the exact mnemonics and operand formats from ISA.md:

### Core
```
NOP
PUSH_I64 <integer>
PUSH_F64 <float>
PUSH_BYTES <byte_array>   ; "quoted string" or 0xHH 0xHH ...
POP
DUP
SWAP
PUSH_NIL
```

### Arithmetic
```
ADD
SUB
MUL
DIV
MOD
NEG
```

### Comparison
```
EQ
NE
LT
LE
GT
GE
```

### Logic
```
AND
OR
XOR
NOT
BOOL_AND
BOOL_OR
BOOL_XOR
BOOL_NOT
```

### Control Flow
```
JUMP <label_or_address>
JUMP_IF <label_or_address>
JUMP_IF_NOT <label_or_address>
CALL <module_id> <function_label_or_index>
RETURN
HALT
YIELD
EMIT <event_id>
```

### Memory
```
LOAD <address>
STORE <address>
ALLOC <size>
FREE
COPY <dst_addr> <src_addr> <length>
SET <addr> <byte_value>
GET <address>
```

### Agent-Semantic
```
VEC_PULL <hash_or_label>
VEC_PUSH <hash_or_label>
PROB_B <threshold_float> <offset_label_or_address>
SNAP_S
SNAP_R <snapshot_id_label_or_immediate>
TOOL_X <cap_token_immediate> <tool_id_immediate>
```

## Examples

### Hello World (via EMIT to host)
```
.start:
    PUSH_BYTES "Hello, NEURAL-0!"
    EMIT 0x01          ; Event ID 0x01 = print string
    HALT
```

### Fibonacci Iterative
```
; Computes fib(10) and leaves result on stack
    PUSH_I64 0         ; a = 0
    PUSH_I64 1         ; b = 1
    PUSH_I64 10        ; counter = 10
.loop:
    JUMP_IF .done
    DUP                ; duplicate counter
    PUSH_I64 1
    SUB                ; counter - 1
    SWAP               ; get b on top
    PUSH_I64 0         ; temp = 0
    PUSH_I64 0         ; temp = 0 (placeholder)
    ROT                ; actually need to implement ROT or use more stack ops
    ; For simplicity, showing concept - real version would use stack manipulation
    HALT
.done:
    ; Result in b
    HALT
```

### Vector Storage Usage
```
; Store a value by semantic hash
    PUSH_I64 42
    VEC_PUSH "answer_to_life"
    
    ; ... later, possibly after snapshot/restore ...
    VEC_PULL "answer_to_life"
    ; Should get 42 back on stack
```

### Probabilistic Branch
```
; Compute confidence (0.0 to 1.0) on stack, then branch if > 0.95
    ; ... confidence calculation leaves F64 on stack ...
    PROB_B 0.95 .high_confidence
    ; Low confidence path
    PUSH_I64 0
    JUMP .end
.high_confidence:
    ; High confidence path
    PUSH_I64 1
.end:
    HALT
```

## Assembler Behavior

### Numeric Literals
- Decimal: `123`, `-45`
- Hexadecimal: `0xFF`, `-0xAB`
- Binary: `0b1010`, `-0b1`
- No suffixes - type determined by instruction context

### String Literals
- Support standard escape sequences: `\n` `\t` `\\` `\"` `\'`
- Unicode UTF-8 encoding
- Length limited to 65535 bytes (U16 limit)

### Label Resolution
- Forward references allowed
- Labels must be unique within file
- Assembler resolves labels to PC offsets during second pass

### Error Handling
Assembler should report:
- Syntax errors
- Unknown mnemonics
- Operand type mismatches
- Undefined labels
- Label redefinition
- Invalid numeric literals
- String too long

## Disassembler (Binary to Text)

For completeness, the disassembler should:
- Take `.n0b` file and produce equivalent `.n0asm`
- Resolve PC-relative jumps to labels when possible
- Emit sensible labels for jump targets (e.g., `L0001`, `L0002`)
- Show data section with appropriate directives
- Preserve comments only if they existed in original (rare for binary input)