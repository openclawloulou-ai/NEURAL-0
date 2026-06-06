## What

<!-- One paragraph: what changed and why. Link the issue this closes. -->

## Phase alignment

<!-- Which item from CONTRIBUTING.md §2 does this advance? Quote the deliverable verbatim. -->

## Spec impact

- [ ] No spec change
- [ ] Spec change proposed in `spec/X.md` (paste diff or link)
- [ ] Wire format / version bump (BLOCKED — open a discussion first)

## Test plan

- [ ] `cargo fmt --all` produces no diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean
- [ ] `cargo test --workspace` is green
- [ ] Manually ran `n0 asm` + `n0 run` on `tests/programs/hello_arithmetic.n0asm` and got `Final stack: [I64(5)]`
- [ ] Added or updated a test that covers the changed behaviour (name it)

## Risk

<!-- What could break. What you did to mitigate. If unsure, say so — the reviewer needs to know. -->

## Agent contract

<!-- If you are an AI agent, confirm you have read AGENTS.md and followed §6 (verification). -->
