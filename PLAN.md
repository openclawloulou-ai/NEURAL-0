# Project NEURAL-0: High-Density Agent Execution Protocol

**Status:** Refined Specification v2.0  
**Version:** 2.0.0  
**Target:** Autonomous AI Agents  
**Human Readability:** Non-Essential / Deprecated (Assembly format available for debugging)

---

## 1. Executive Summary

NEURAL-0 (N0) is a programming language designed to eliminate the "Human-in-the-Middle" bottleneck. Traditional languages (Python, JS) are optimized for human eyes, which wastes LLM tokens and introduces syntax errors. N0 is a **stack-based virtual machine with 16-bit opcodes** that enables maximum logic density and zero-cost state serialization via snapshots.

Key improvements over the initial specification:
- **Hybrid Architecture**: Linear bytecode execution inside modules, with modules composable as a DAG
- **Complete Instruction Set**: 35+ opcodes covering arithmetic, logic, control flow, memory, and agent-semantic operations
- **Binary + Assembly Formats**: Lossless text-to-binary translation for LLM-friendly generation
- **Robust Security**: Capability-based model with fine-grained scoping and usage limits
- **Production-Ready Specs**: Detailed binary format, snapshot mechanism, and error handling

---

## 2. Repository Architecture

The repository is structured to support autonomous contribution loops without human intervention:

```text
/neural-0
  ├── /kernel             # The VM (Virtual Machine) implemented in Rust
  ├── /assembler          # Text assembly (.n0asm) -> binary (.n0b) converter
  ├── /spec               # Formal specifications (ISA, binary format, capabilities, etc.)
  ├── /sim                # The Automated Fitness & Simulation Environment (Phase 2)
  ├── /registry           # Metadata database for "Latent Function Discovery" (Phase 3)
  ├── /agents             # System prompts for LLM "Contributors"
  ├── /ledger             # The append-only record of evolutionary merges (Phase 4)
  └── /tests              # Test programs and validation suites
```

---

## 3. Language Specification (The Bytecode)

N0 bypasses text parsing at runtime. The VM reads a binary stream where each instruction consists of a 2-byte OpCode followed by variable-length data payloads. An optional text assembly format (`.n0asm`) allows LLMs to generate code efficiently, which is then assembled to binary.

### Core Opcode Categories (16-bit space, 0x0001-0xFFFF):

**Core Operations (0x0001-0x000F)**: NOP, PUSH_I64/F64/BYTES/NIL, POP, DUP, SWAP  
**Arithmetic (0x0010-0x001F)**: ADD, SUB, MUL, DIV, MOD, NEG  
**Comparison (0x0020-0x002F)**: EQ, NE, LT, LE, GT, GE  
**Logic (0x0030-0x003F)**: AND, OR, XOR, NOT, BOOL_AND/BOR/XOR/NOT  
**Control Flow (0x0040-0x004F)**: JUMP, JUMP_IF/NOT, CALL, RETURN, HALT, YIELD, EMIT  
**Memory (0x0050-0x005F)**: LOAD, STORE, ALLOC, FREE, COPY, SET, GET  
**Agent-Semantic (0x0060-0x006F)**: VEC_PULL/PUSH, PROB_B, SNAP_S/R, TOOL_X, RESERVED  

See `spec/ISA.md` for complete opcode map, encoding details, and trap conditions.

### Value Types (Tags)
All stack values are tagged with their type:
- I64: 64-bit signed integer
- F64: 64-bit IEEE 754 float  
- Bytes: Length-prefixed byte array
- Ptr: Linear memory offset
- Nil: Null/empty value

---

## 4. The Implementation Roadmap (For the Builder Agent)

### Phase 1: The Rust Kernel (Current Focus)
**Goal**: Build a working VM that can execute N0 bytecode with snapshot/resume capability.

#### Deliverables:
1. **Rust Workspace**: `/kernel` and `/assembler` crates with Cargo.toml
2. **Opcode Decoder**: Binary stream -> instruction enum (`kernel/src/opcodes.rs`)
3. **Tagged Value Stack**: With overflow/underflow protection (`kernel/src/stack.rs`) 
4. **Execution Loop**: Fetch-decode-execute with PC tracking (`kernel/src/vm.rs`)
5. **Core Instructions**: Arithmetic, comparison, logic, control flow, memory ops
6. **Text Assembler**: `.n0asm` -> `.n0b` conversion (`/assembler` crate)
7. **Snapshot Engine**: Binary state serialization/deserialization with CRC32 (`kernel/src/snapshot.rs`)
8. **Capability System**: Token validation for TOOL_X (`kernel/src/capability.rs`)
9. **CLI Interface**: `n0 run <file.n0b>` and `n0 asm <file.n0asm> -o <file.n0b>`
10. **Test Suite**: Validation programs for every opcode category

**Success Criteria**: An agent can generate `.n0asm` text, assemble it, execute it on the VM, snapshot state, and resume from the snapshot.

#### Key Technical Decisions:
- **Wasm Deferred**: Phase 1 uses native Rust; Wasm sandboxing comes in Phase 2
- **SELF_M Deferred**: Self-modifying code postponed to v2 for security
- **Linear Memory**: Flat byte array with bounds checking (like Wasm)
- **Trap Model**: Immediate halt on error with error code returned to host
- **Stack Size**: Configurable, default 64KB

### Phase 2: Wasm Sandboxing + Module DAG
- Compile kernel to Wasm via `wasmtime`
- Implement module loading and DAG execution from manifest
- Host-VM interface for capability injection
- Multi-module programs with typed ports

### Phase 3: Agent Integration
- LLM prompt templates for N0 assembly generation
- Round-trip test: prompt -> `.n0asm` -> binary -> execute -> verify
- `VEC_PULL`/`VEC_PUSH` with actual vector store backend
- `PROB_B` integration with confidence scoring from prior computations

### Phase 4: Evolutionary Loop + Registry
- Fitness testing framework with normalized scoring
- Contribution pipeline (`/contributions` -> simulation -> merge)
- Embedding-based code registry for latent function discovery
- Automated merge logic based on efficiency improvements

> **SAFETY GATING (mandatory before any auto-merge is enabled):**
> - Contributions must be cryptographically signed by the contributor agent; unsigned or
>   untrusted contributors only land in a `quarantine/` subtree, never in `main`.
> - Fitness evaluation runs in a hard sandbox (Wasmtime + dropped caps + fuel limit) — never
>   on the host directly.
> - Two-stage gate required for merge into `registry/`: (1) automated static safety check
>   (no `TOOL_X` use without matching capability, no unbounded `ALLOC`, no reachable
>   untrusted code path) and (2) attested review by a second agent or human with merge rights.
> - Registry is content-addressed and append-only; merges create new entries, never mutate
>   historical ones.
> - `TOOL_X` and `VEC_*` opcodes in contributed code are stripped by the simulator before
>   re-execution; only the pure-logic contribution survives in the registry entry.

---

## 5. Setup Instructions (The "Bootstrap" Command)

To begin, initialize your environment and hand this prompt to your lead "Architect Agent":

```bash
mkdir neural-0 && cd neural-0
git init
cp Project\ NEURAL\ 0\ High.md PLAN.md  # Keep this overview
# Then copy the spec files:
mkdir -p spec && cp -r /path/to/refined/spec/* spec/
# Create initial directory structure:
mkdir -p kernel/src assembler/src tests/programs agents sim registry ledger
```

### The Bootstrap Prompt (Updated for v2.0)

> "You are the Architect of NEURAL-0 v2.0. Your goal is to build the Phase 1 Kernel in Rust - a stack-based VM that executes 16-bit OpCodes with snapshot/resume capability.
>
> Architecture:
> - Stack-based VM with tagged values (I64, F64, Bytes, Ptr, Nil)
> - Linear memory model with bounds checking
> - Trap-based error handling (no exceptions)
> - Text assembly format for LLM-friendly code generation
>
> Build Order:
> 1. Create Rust workspace with /kernel and /assembler crates
> 2. Define OpCode enum from ISA.md in kernel/src/opcodes.rs
> 3. Implement tagged value stack in kernel/src/stack.rs
> 4. Implement execution loop (fetch-decode-execute) in kernel/src/vm.rs
> 5. Implement arithmetic, comparison, control flow, and memory ops
> 6. Implement text assembler (lexer -> parser -> binary codegen)
> 7. Implement SNAP_S/SNAP_R (binary state serialization with CRC32)
> 8. Implement capability token checking for TOOL_X (stub actual tools)
> 9. Write CLI: `n0 run <file.n0b>` and `n0 asm <file.n0asm> -o <file.n0b>`
> 10. Write tests for every OpCode category
>
> Constraints:
> - Do NOT optimize for human readability; optimize for bytecode correctness and minimal attack surface
> - Every memory access must be bounds-checked
> - Every stack operation must check for overflow/underflow
> - Do NOT implement SELF_M (deferred to v2)
> - Focus on correctness first, performance later
>
> Success Criteria: Run the test programs in /tests/programs and verify correct output."

---

## 6. The "Judge" Logic (Fitness Function)

The core of the language's growth is the **Normalized Fitness Formula**:

$$F = \frac{Accuracy \times W_a}{\left(\frac{Tokens}{T_{baseline}}\right) + \left(\frac{Cycles}{C_{baseline}}\right) + \left(\frac{Memory}{M_{baseline}}\right)}$$

Where:
- $W_a$ = Accuracy weight (default: 1.0)
- $T_{baseline}$, $C_{baseline}$, $M_{baseline}$ = Per-task baselines for normalization

This creates a natural selection pressure where the language becomes more efficient every time an agent contributes to it. Agents are programmed to maximize $F$.

Key improvements over v1:
- **Normalization**: All dimensions comparable via per-task baselines
- **Multi-dimensional**: Accounts for token, cycle, AND memory efficiency
- **Configurable Weights**: Accuracy can be weighted higher/lower as needed
- **Baseline-Dependent**: Fair comparison across different problem types

---

## 7. Getting Started for Agent Contributors

1. **Phase 1 Focus**: Implement the Rust kernel and assembler per the Bootstrap Prompt above
2. **Reference Implementation**: Use the spec files in `/spec` as the authoritative reference
3. **Test-Driven**: Make the programs in `/tests/programs` pass before moving on
4. **Agent-First Mindset**: Optimize for LLM token efficiency, not human convenience
5. **Security Primacy**: Never bypass bounds checks or capability validation
6. **Iterative Approach**: Get a minimal working VM first, then add features

Once Phase 1 is complete and tested, the system is ready for:
- Phase 2: Wasm compilation and DAG composition
- Phase 3: Agent integration and vector storage
- Phase 4: Evolutionary loops and autonomous stdlib growth

The language gets smarter and faster the more agents use it—but only if we build a solid, secure foundation first.