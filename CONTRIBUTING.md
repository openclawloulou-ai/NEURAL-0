# Contributing to NEURAL-0 — Agent Contract

This file is your workflow contract. You are an AI agent. The human maintainer
expects the following from any contribution (yours or another agent's). If you
are a human, the same rules apply with a human-scale time budget.

---

## 1. Read first, code second

Before opening a PR, read in this order:

1. `README.md` — orientation, invariants, known issues.
2. `PLAN.md` — current phase, what "done" means for that phase.
3. The relevant `spec/*.md` for whatever you're touching. Specs are normative.
4. Existing code in the affected module. Mimic the style already there.

If `spec/` and the code disagree, **the spec wins.** Open an issue proposing
the spec change; do not silently mutate one to match the other.

---

## 2. Pick work that advances the current phase

The phase banner in `README.md` is authoritative. Cross-phase work is fine
if it's small and well-scoped (typo fixes, doc clarifications, CI plumbing).
Anything structural belongs in the active phase.

For **Phase 1** the prioritized backlog is:

- [ ] Finish `assembler/src/parser.rs` + `codegen.rs` to parity with `spec/ASSEMBLY.md`.
      The `SimpleAssembler` covers ~5% of the spec; the full assembler must
      handle labels, data section, all directive forms, and all opcodes.
- [ ] Wire `SNAP_S` / `SNAP_R` end-to-end (binary format implemented in
      `kernel/src/snapshot.rs`, host integration in `kernel/src/main.rs`).
- [ ] Wire `TOOL_X` against a real `CapabilityHost` trait implementation
      (currently stubbed in `kernel/src/capability.rs`).
- [ ] Resolve the Agent-Semantic opcode concern (see README §Specification
      Concerns). Either move them to host-imports or document the security
      boundary clearly.
- [ ] Add `cargo test` integration tests that assemble + run every file in
      `tests/programs/` and assert the final stack state.

---

## 3. Commit discipline

One logical change per commit. Commit message format:

```
<area>: <imperative summary>

<one-paragraph rationale tied to PLAN.md phase or spec section>

Refs: PLAN.md §X, spec/Y.md §Z
```

`<area>` is one of: `kernel`, `assembler`, `spec`, `docs`, `ci`, `tests`,
`tools`. Avoid drive-by edits to unrelated files in the same commit.

---

## 4. Code style

Follow the existing style in the file you're editing. Conventions observed
in the current tree:

- Rust 2021 edition, no external deps in `kernel/` (Phase 1), `logos` allowed
  in `assembler/`.
- `pub` only on items that need to be visible from outside the module.
- No `unwrap()` in library code; `expect("context")` is acceptable when the
  invariant is locally provable.
- No `TODO` without a linked issue or `Refs:` line.
- No comments explaining *what* the code does. Comments explain *why* the
  code is non-obvious or *which spec section* it implements.

---

## 5. Hard constraints — do not violate, do not "simplify"

These are also in `README.md` §Architectural Invariants. Repeating them here
because they are the most common failure mode in agent-generated patches:

- **No raw pointer math in `kernel/`.** All memory access goes through
  `Memory` with bounds checks. This is non-negotiable; the spec calls it out
  for security reasons.
- **No `unsafe` in `kernel/`** without a `// SAFETY:` comment that cites the
  specific invariant being upheld and the spec section that mandates it.
- **No capability bypass.** If `TOOL_X` lacks a matching capability, the VM
  traps. Do not add fallback paths that call the host anyway.
- **No self-modifying code.** `SELF_M` is deferred to v2. If you find yourself
  wanting it, you don't.
- **No silent opcode renumbering.** Bump the version in `BINARY_FORMAT.md`
  and write a migration note.

---

## 6. Testing

Every opcode that is *implemented* must have a test. Every test program in
`tests/programs/` must:

1. Assemble cleanly (no warnings, no errors).
2. Run to `HALT` with no trap.
3. Produce the stack state asserted in a comment at the top of the file, or
   in `tests/programs/EXPECTED.md` (create it if it doesn't exist).

If you add a new opcode, add a `.n0asm` program demonstrating it. If you
change the binary format, add a round-trip test (assemble → disassemble →
re-assemble → byte-equal).

---

## 7. PR description template

When you open a PR (or push a branch the maintainer will review), include:

```
## What
<one paragraph: what changed and why>

## Phase alignment
<which PLAN.md phase this advances; quote the relevant deliverable>

## Spec impact
- [ ] No spec change
- [ ] Spec change proposed in spec/X.md (paste diff or link)

## Test plan
- [ ] cargo test --workspace passes
- [ ] All tests/programs/*.n0asm assemble and run to HALT
- [ ] New tests added (list them)

## Risk
<what could break; what you did to mitigate>
```

If you cannot tick all the boxes in *Test plan*, explain why in the PR body.
"Do not run tests because they are slow" is not an explanation.

---

## 8. You are not the architect

You are a contributor. The maintainer (or a designated Architect Agent) makes
cross-cutting decisions. Surface trade-offs; do not resolve them unilaterally.
When in doubt, open a discussion, do not merge.
