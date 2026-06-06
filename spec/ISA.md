# NEURAL-0 Instruction Set Architecture (ISA)

## Overview
NEURAL-0 is a stack-based virtual machine with 16-bit opcodes. Each instruction consists of:
- 2-byte opcode (big-endian)
- Variable-length immediate operands (as specified per instruction)

All values on the stack are tagged with their type. The VM traps on type mismatches, stack overflow/underflow, or out-of-bounds memory access.

## Value Types (Tags)
| Tag | Type | Size | Description |
|-----|------|------|-------------|
| 0x01 | I64 | 8 bytes | 64-bit signed integer |
| 0x02 | F64 | 8 bytes | 64-bit IEEE 754 float |
| 0x03 | Bytes | variable | Byte array (length-prefixed as U16) |
| 0x04 | Ptr | 4 bytes | Linear memory offset |
| 0x05 | Nil | 0 bytes | Null/empty value |

## Opcode Map

### Core (0x0001 - 0x000F)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 0x0001 | NOP | - | No operation |
| 0x0002 | PUSH_I64 | i64 | Push 64-bit integer |
| 0x0003 | PUSH_F64 | f64 | Push 64-bit float |
| 0x0004 | PUSH_BYTES | u16:len + bytes | Push byte array |
| 0x0005 | POP | - | Pop and discard top of stack |
| 0x0006 | DUP | - | Duplicate top of stack |
| 0x0007 | SWAP | - | Swap top two stack values |
| 0x0008 | PUSH_NIL | - | Push Nil value |

### Arithmetic (0x0010 - 0x001F)
| Opcode | Mnemonic | Description | Traps On |
|--------|----------|-------------|----------|
| 0x0010 | ADD | Pop a, b; push b + a | Type mismatch (non-numeric) |
| 0x0011 | SUB | Pop a, b; push b - a | Type mismatch (non-numeric) |
| 0x0012 | MUL | Pop a, b; push b * a | Type mismatch (non-numeric) |
| 0x0013 | DIV | Pop a, b; push b / a | Type mismatch, division by zero |
| 0x0014 | MOD | Pop a, b; push b % a | Type mismatch, division by zero |
| 0x0015 | NEG | Pop a; push -a | Type mismatch (non-numeric) |

### Comparison (0x0020 - 0x002F)
| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x0020 | EQ | Pop a, b; push 1 if b == a else 0 |
| 0x0021 | NE | Pop a, b; push 1 if b != a else 0 |
| 0x0022 | LT | Pop a, b; push 1 if b < a else 0 |
| 0x0023 | LE | Pop a, b; push 1 if b <= a else 0 |
| 0x0024 | GT | Pop a, b; push 1 if b > a else 0 |
| 0x0025 | GE | Pop a, b; push 1 if b >= a else 0 |

Comparison operations work on I64 and F64 types. For mixed types, the VM traps.

### Logic (0x0030 - 0x003F)
| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x0030 | AND | Pop a, b; push b & a (bitwise) |
| 0x0031 | OR | Pop a, b; push b \| a (bitwise) |
| 0x0032 | XOR | Pop a, b; push b ^ a (bitwise) |
| 0x0033 | NOT | Pop a; push ~a (bitwise) |
| 0x0034 | BOOL_AND | Pop a, b; push 1 if b!=0 && a!=0 else 0 |
| 0x0035 | BOOL_OR | Pop a, b; push 1 if b!=0 || a!=0 else 0 |
| 0x0036 | BOOL_XOR | Pop a, b; push 1 if (b!=0) != (a!=0) else 0 |
| 0x0037 | BOOL_NOT | Pop a; push 1 if a==0 else 0 |

Bitwise operations require I64 type. Boolean operations treat any non-zero as true.

### Control Flow (0x0040 - 0x004F)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 0x0040 | JUMP | i32:offset | PC += offset (signed) |
| 0x0041 | JUMP_IF | i32:offset | If top != 0, PC += offset; else pop |
| 0x0042 | JUMP_IF_NOT | i32:offset | If top == 0, PC += offset; else pop |
| 0x0043 | CALL | u32:module + u16:function | Call function in module |
| 0x0044 | RETURN | - | Return from current function |
| 0x0045 | HALT | - | Stop execution and return to host |
| 0x0046 | YIELD | - |
### Memory (0x0050 - 0x005F)
| Opcode | Mnemonic | Operands | Description | Traps On |
|--------|----------|----------|-------------|----------|
| 0x0050 | LOAD | u32:addr | Push value at memory[addr] | OOB read |
| 0x0051 | STORE | u32:addr | Pop value, store to memory[addr] | OOB write, type mismatch (only I64/F64/Bytes) |
| 0x0052 | ALLOC | u32:size | Allocate size bytes, push pointer | OOM |
| 0x0053 | FREE | - | Pop pointer, deallocate memory | Invalid pointer, double free |
| 0x0054 | COPY | u32:dst + u32:src + u32:len | Copy len bytes from src to dst | OOB read/write |
| 0x0055 | SET | u32:addr + u8:val | Set memory[addr] = val (byte) | OOB write |
| 0x0056 | GET | u32:addr | Push memory[addr] as U8 | OOB read |

### Agent-Semantic (0x0060 - 0x006F)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 0x0060 | VEC_PULL | u32:hash | Retrieve value from vector storage by hash |
| 0x0061 | VEC_PUSH | u32:hash | Store top of stack to vector storage by hash |
| 0x0062 | PROB_B | f32:threshold + i32:offset | If pop() as F64 > threshold, PC += offset; else pop |
| 0x0063 | SNAP_S | - | Snapshot entire VM state to binary blob |
| 0x0064 | SNAP_R | u32:id | Restore VM state from snapshot ID |
| 0x0065 | TOOL_X | u32:cap_token + u16:tool_id | Execute external tool via capability |
| 0x0066 | RESERVED | - | Future use |
| 0x0067 | RESERVED | - | Future use |

### Notes
1. All multi-byte values are encoded in big-endian (network byte order)
2. Offsets in JUMP instructions are signed 32-bit integers relative to the next instruction
3. The VM begins execution at offset 0 in the code segment
4. Stack grows downward (toward lower addresses) - implementation detail
5. Linear memory starts at offset 0 and grows upward via ALLOC
6. Vector storage is external to the VM - content-addressed by SHA-256 hash of the value
