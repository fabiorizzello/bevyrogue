# Inventory — W0d sibling `tests.rs` relocate

Created: 2026-05-20
Parent workflow: `260520-3-w0c-tier-c-residue` (closed; backlog item #1)
Scope: 3 external sibling `tests.rs` files declared via `#[cfg(test)] mod tests;`, ~1,021 LOC total. Surfaced during Tier-B closeout.

## Goal

Relocate the 3 sibling test files to `tests/` so the `src/` tree contains zero test-only code. These files formally satisfy R003's "short `#[cfg(test)] mod tests;`" wording (the declaration is one line) but break its spirit — each file is 300+ LOC of test code shipped inside `src/`.

## Candidates (verified 2026-05-20 against HEAD `89728ae`)

| # | Source file | Parent decl | LOC | Tests | Target |
|---|---|---|---|---|---|
| 1 | `src/combat/runtime/builtins/tests.rs` | `src/combat/runtime/builtins.rs:330` | 329 | 7 | `tests/runtime_builtins_internals.rs` |
| 2 | `src/combat/runtime/runner/tests.rs` | `src/combat/runtime/runner.rs:415` | 389 | 6 | `tests/runtime_runner_internals.rs` |
| 3 | `src/combat/turn_system/tests.rs` | `src/combat/turn_system/mod.rs:30` | 303 | 4 | `tests/turn_system_internals.rs` |

Total: **1,021 LOC** to move out of `src/`.

## Audit finding (key result)

Pre-work assumption (from Tier-C SUMMARY) was that these files would require per-file investigation of `super::*` usage, because each is a sibling under the parent module and might reach into private items. **Audit reveals this is not the case.** Per-file symbol scan:

### `builtins/tests.rs`
- `use super::*;` at line 1, but the only `super` symbol the file actually invokes is `register_kernel_builtins` (line 20 of `builtins.rs`, **already `pub`**).
- The 16 private builtin fns in `builtins.rs` (`deal_damage`, `apply_status`, `advance_turn`, etc.) are referenced ONLY as string keys (`"core/deal_damage"`) into the hook registry — no direct function calls. Tests look them up via `regs.hooks.get(...)`.
- Pure relocate.

### `runner/tests.rs`
- `use super::*;` at line 1; consumed symbols are `BeatRunner`, `LoopFrame`, `StepOutcome`, `AwaitingCueInfo` — all `pub` on `runner.rs` (lines 28, 41, 60, 72).
- All called methods (`BeatRunner::new`, `.with_clock`, `.step`, `.resume_cue`, `.run_to_completion`) are `pub fn`.
- `runner.rs` has zero `pub(crate)` items.
- Pure relocate.

### `turn_system/tests.rs`
- `use super::*;` at line 1; consumed symbols are `resolve_action_system`, `check_victory_system`, `ActionIntent` — all re-exported `pub` (not `pub(crate)`) by `turn_system/mod.rs`.
- The `pub(crate)` re-exports (`ResolveActorsQuery`, `emit_combat_beat`, `emit_kernel_transition`, `set_phase`, `step_app`, `step_declaration`) are **not referenced** by the test file.
- Pure relocate.

## Reference pattern (from Tier-A/B/C)

Same as prior tiers, with one extra step per file:

1. Rewrite `use super::*;` as explicit `use bevyrogue::combat::...` imports for the items actually used.
2. Replace `use crate::combat::...` with `use bevyrogue::combat::...` throughout.
3. `git mv src/combat/<path>/tests.rs tests/<name>_internals.rs`.
4. Delete the `#[cfg(test)] mod tests;` line from the parent module.
5. Verify with `cargo test --test <name>_internals`.
6. Commit.

## Out of scope

- The 4 binary-private inline blocks (`src/headless.rs`, `src/windowed/{mod,render}.rs`, `src/bin/combat_cli.rs`, 131 LOC) — architectural slice, not a sibling-tests problem.
- Splitting any of the 3 test files into smaller integration files. Move first, restructure later if scope permits.

## Acceptance

- All 3 sibling `tests.rs` files removed from `src/`.
- Corresponding `tests/<name>_internals.rs` files exist with the same test bodies.
- Parent modules no longer carry `#[cfg(test)] mod tests;`.
- `cargo test --tests` green (17 tests preserved across the 3 relocated files).
- `cargo test --features windowed --tests` green.
- `cargo check --tests` and `cargo check --features windowed --tests` exit 0.
- `scripts/check_loc_cap.sh` still passes.
