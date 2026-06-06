# AGENTS.md

You are an AI agent. You have just landed in the NEURAL-0 repository.
This file is your onboarding contract. Read it, then read the files
it points you to, then act.

---

## 1. The 90-second orientation

1. `README.md` — what this is, project status, architectural invariants.
2. `PLAN.md` — phased roadmap and current priorities.
3. `CONTRIBUTING.md` — workflow contract, commit format, hard constraints, PR template.
4. `spec/ISA.md` — the opcode map. Specs are normative; code is downstream.

## 2. State you may assume

- **Phase 1 is in progress.** The kernel and a minimal assembler exist. The
  full assembler (`assembler/src/parser.rs`, `codegen.rs`) is mid-rewrite.
- The Rust workspace builds cleanly: `cargo check --workspace` exits 0.
- One end-to-end test passes manually: `hello_arithmetic.n0asm` → `[I64(5)]`.
  No `cargo test` integration tests are wired up yet — if you add one and it
  passes, you have meaningfully advanced Phase 1.
- `n0` is the only binary. It subcommands are `run <file.n0b>` and
  `asm <file.n0asm> -o <file.n0b>`. A separate `n0-mcp` binary exposes
  the same capabilities to any MCP-compatible agent.

## 3. What you are allowed to do without asking

- Read every file in the repository.
- Run `cargo check`, `cargo test`, `cargo build`, `cargo run`, `cargo fmt`.
- Open issues proposing spec changes, with a diff and a rationale.
- Open PRs that tick one item off `CONTRIBUTING.md` §2 (Phase 1 backlog).
- Open PRs that fix typos, broken links, or compilation errors.

## 4. What requires a maintainer's sign-off

- Anything in `spec/` (the spec is normative).
- Renumbering opcodes or changing the wire format (requires a version bump).
- Adding a new public dependency to the workspace.
- Touching `.github/workflows/`, `LICENSE`, or this file.
- Merging into `main`. (The default branch is protected; you cannot push to it.)

## 5. How to pick a task

Read `CONTRIBUTING.md` §2. The backlog is ordered by leverage. Pick the
highest-priority item whose scope you can finish in a single PR. If
nothing on the list is in your reach, propose a new item in an issue
rather than freelancing.

## 6. How to verify your work

Before opening a PR:

1. `cargo fmt --all` — no diff after.
2. `cargo clippy --workspace --all-targets -- -D warnings` — clean.
3. `cargo test --workspace` — green. If you wrote no test, write one.
4. Manually exercise any CLI changes:
   ```
   cargo run -p neural0_kernel -- asm tests/programs/hello_arithmetic.n0asm -o /tmp/x.n0b
   cargo run -p neural0_kernel -- run /tmp/x.n0b
   ```
   Expected: `Final stack: [I64(5)]`.
5. If you changed `spec/`, the change must come with a migration note
   and a version bump in `BINARY_FORMAT.md`.

## 7. How to ask for help

Open an issue. Title it `[Question]: <one-line>`. Do not DM, do not
comment on a closed PR, do not edit a file you don't own without
coordination. Maintainers are slow; patience is part of the contract.

## 8. Out of scope for you

- Marketing, social-media posts, or "elevator pitch" rewrites of the README.
- Renaming the project, the binary, or any public API.
- Adding a benchmark suite (we don't have one to compare against yet).
- Implementing Phase 2 (Wasm sandbox) or Phase 3 (agent integration) —
  these have open design questions that need maintainer input first.

You are a contributor, not an architect. Surface trade-offs; do not
resolve them unilaterally.
