# NEURAL-0

> **You are an AI agent. Read this first.**

NEURAL-0 (N0) is a stack-based 16-bit-opcode virtual machine and bytecode language
optimized for **LLM generation, not human reading**. Every design choice trades human
ergonomics for token density, verifiability, and minimal attack surface. The text
assembly format (`.n0asm`) exists only so that *you* can write and audit bytecode
without hand-encoding hex.

This repository is your working environment. It is licensed MIT. You may fork it,
extend it, and propose contributions upstream.

---

## Project Status — Read Before You Touch Anything

| Phase | Name                       | State           |
|------:|----------------------------|-----------------|
| 1     | Rust kernel + assembler    | **In progress** |
| 2     | Wasm sandbox + module DAG  | Not started     |
| 3     | Agent integration          | Not started     |
| 4     | Evolutionary loop/registry | Specified, gated |

The reference implementation in `kernel/` and `assembler/` is **partial**. A minimal
VM that handles Core + Arithmetic + Comparison + Logic + Control Flow + Memory
opcodes exists. Snapshots, capabilities, and Agent-Semantic opcodes are scaffolded
but not wired end-to-end. The `assembler/src/simple.rs` works for a tiny subset
(`PUSH_I64`, `ADD`, `HALT`); the full lexer/parser/codegen in `assembler/src/` is
mid-rewrite.

**Do not claim Phase 1 complete.** Verify by running `cargo test` and executing
`tests/programs/*.n0asm` end-to-end before reporting status.

---

## Repository Map

```
.
├── PLAN.md          # Authoritative roadmap, phases, fitness function (READ THIS)
├── CONTRIBUTING.md  # Your workflow contract — read this before committing
├── AGENTS.md        # Onboarding contract for AI agents picking up the repo
├── llms.txt         # LLM-oriented summary of the repo, per llmstxt.org
├── Cargo.toml       # Workspace manifest (kernel + assembler + mcp)
├── LICENSE          # MIT
│
├── spec/            # Normative specifications (do not contradict these)
│   ├── ISA.md             # 16-bit opcode map, encoding, traps
│   ├── BINARY_FORMAT.md   # .n0b / .n0dag wire format
│   ├── ASSEMBLY.md        # .n0asm text format (lossless w/ binary)
│   ├── CAPABILITIES.md    # Capability tokens, TOOL_X enforcement
│   └── SNAPSHOT.md        # .n0snap format, CRC32, restore semantics
│
├── kernel/          # Rust VM crate (`n0` binary)
│   └── src/
│       ├── opcodes.rs     # OpCode enum + binary readers
│       ├── value.rs       # Tagged value (I64/F64/Bytes/Ptr/Nil)
│       ├── stack.rs       # Bounded stack w/ overflow trap
│       ├── memory.rs      # Linear memory w/ bounds checking
│       ├── vm.rs          # Fetch-decode-execute loop
│       ├── trap.rs        # Trap codes
│       ├── capability.rs  # Capability table + TOOL_X gate
│       ├── module.rs      # Module header parsing (pub extract_code)
│       └── snapshot.rs    # SNAP_S / SNAP_R binary format
│
├── mcp/             # MCP server crate (`n0-mcp` binary)
│   └── src/main.rs        # stdio MCP server: `assemble` and `run` tools
│
├── assembler/       # Rust `.n0asm` -> `.n0b` crate (`n0asm` binary)
│   └── src/
│       ├── simple.rs      # Minimal working assembler (subset)
│       ├── lexer.rs       # Full lexer (logos)
│       ├── parser.rs      # Full parser
│       └── codegen.rs     # Full code generator
│
├── tests/programs/  # Reference .n0asm programs (acceptance tests)
```

If `spec/` and `kernel/src/opcodes.rs` disagree, `spec/` wins. The implementation
is downstream of the spec, not the other way around.

---

## Hello, NEURAL-0

A minimal program that pushes `2 + 3` onto the stack and halts:

```n0asm
; hello_arithmetic.n0asm
PUSH_I64 2
PUSH_I64 3
ADD
HALT
```

To assemble and run it:

```bash
cargo run -p neural0_kernel -- asm tests/programs/hello_arithmetic.n0asm -o /tmp/hello.n0b
cargo run -p neural0_kernel -- run /tmp/hello.n0b
```

Expected behaviour: VM halts with `Final stack: [I64(5)]`.

## Using NEURAL-0 from an MCP-compatible agent

The `n0-mcp` binary is a stdio MCP server exposing the kernel and assembler as
two tools — `assemble` and `run` — to any MCP client (Claude Desktop, Cursor,
etc.). Register it in the client's MCP config:

```json
{
  "mcpServers": {
    "neural0": {
      "command": "cargo",
      "args": ["run", "-p", "neural0_mcp", "--release"]
    }
  }
}
```

Once registered, an agent can call `assemble` to compile `.n0asm` source to
hex-encoded `.n0b`, then call `run` to execute the module on a fresh VM and
read the final stack. The MCP server is a sketch in v0.1 — only `PUSH_I64`,
`ADD`, and `HALT` are reachable through the current assembler.

---

## Architectural Invariants — Do Not Violate

1. **Every memory access is bounds-checked.** No raw pointer math in the kernel.
2. **Every stack op checks for overflow/underflow.** A trap is raised, not a panic.
3. **All multi-byte stream values are big-endian.** Internal representation may
   differ (the data section stores I64/F64 little-endian per `BINARY_FORMAT.md`).
4. **Tagged values.** No untyped stack slots. Type-mismatch traps, not casts.
5. **Trap model.** Errors are `Trap` enum variants, not exceptions. The host gets
   a code, not a backtrace.
6. **Capabilities gate `TOOL_X`.** No capability → no host call. Default capability
   table is empty.
7. **Snapshots include the capability table verbatim.** Restore gives the snapshotted
   code exactly the authority it had at snap time.
8. **Self-modifying code (`SELF_M`) is deferred to v2.** Do not implement it in v1.

---

## Specification Concerns (Read Carefully)

### Agent-Semantic opcodes (0x0060–0x006F) are host imports, not native ISA

`VEC_PULL`, `VEC_PUSH`, `PROB_B`, `SNAP_S`, `SNAP_R`, and `TOOL_X` share the
opcode space with native VM instructions, but `VEC_*` and `TOOL_X` are
**host-bound function calls**, not in-kernel operations. Conflating them with
native opcodes has produced a confused security model in the current spec.

**Required fix in code, not just spec:**

- The kernel must treat the 0x0060–0x006F range as a **host import table**.
  If the host has not registered a binding for a given opcode at module load
  time, the VM traps with `MISSING_HOST_IMPORT` (new trap code) on first use.
- `SNAP_S` / `SNAP_R` legitimately live in the kernel — they manipulate VM
  state, not host state — and should be the only native opcodes in that range.
- `PROB_B` is a control-flow primitive and belongs in Control Flow (0x0040–0x004F).
  Consider renumbering to `0x0048`.
- `TOOL_X` and `VEC_*` should be issued a small host-import prefix or called via
  a `HOST_CALL` indirection (one opcode, table-indexed) so the ISA does not
  encode specific host services.

If you pick this work up, do **not** silently "fix" the opcodes — that breaks
the wire format. Add a version bump to `BINARY_FORMAT.md` and a migration note.

### Phase 4 auto-merge is gated

The roadmap says "automated merge logic based on efficiency improvements." That
sentence, as written, is a supply-chain attack surface. The safety gates in
`PLAN.md` (signed contributions, sandboxed evaluation, two-stage review,
content-addressed append-only registry) are **mandatory**. Do not remove them
to "simplify" Phase 4.

---

## Known Issues (Pre-Existing)

- The `assembler/src/parser.rs` and `codegen.rs` are partially implemented and
  unused — `main.rs` calls `SimpleAssembler` only. Completing the full
  assembler is the highest-leverage Phase 1 task.
- `SimpleAssembler` handles only `PUSH_I64`, `ADD`, and `HALT`. The programs
  in `tests/programs/conditional.n0asm` and `tests/programs/snapshot_resume.n0asm`
  are acceptance targets, not currently passing end-to-end tests.
- `debug_bytes` is a scratch helper binary, not a deliverable. It is in
  `.gitignore`.

---

## License

MIT. See `LICENSE`.
