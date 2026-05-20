# Plan — W0d sibling `tests.rs` relocate

Draft. Status: ready (Phase 2).
Parent: `260520-3-w0c-tier-c-residue` SUMMARY backlog #1.

## Wave breakdown

One atomic commit per file. Wave numbering bumps to `W0d-*` since this is a new tier (Tier-D) following the Tier-C closure at W0c-17.

| Wave | Source | Target | Strategy | Commit format |
|---|---|---|---|---|
| W0d-1 | `src/combat/turn_system/tests.rs` (303 LOC) | `tests/turn_system_internals.rs` | pure relocate, smallest first | `refactor(tests): W0d-1 — relocate turn_system sibling tests` |
| W0d-2 | `src/combat/runtime/builtins/tests.rs` (329 LOC) | `tests/runtime_builtins_internals.rs` | pure relocate | `refactor(tests): W0d-2 — relocate runtime builtins sibling tests` |
| W0d-3 | `src/combat/runtime/runner/tests.rs` (389 LOC) | `tests/runtime_runner_internals.rs` | pure relocate, largest last | `refactor(tests): W0d-3 — relocate runtime runner sibling tests` |

Order: smallest → largest, so any pattern friction surfaces on the cheapest file first.

## Per-wave procedure

For each wave:

1. **Read** the sibling `tests.rs` file end-to-end (already audited during inventory; no private-item access).
2. **Identify** the items consumed from `super::*` (per INVENTORY audit):
   - W0d-1: `resolve_action_system`, `check_victory_system`, `ActionIntent`
   - W0d-2: `register_kernel_builtins`
   - W0d-3: `BeatRunner`, `LoopFrame`, `StepOutcome`, `AwaitingCueInfo`
3. **Rewrite imports** at the top of the new test file:
   - Replace `use super::*;` with explicit `use bevyrogue::combat::<path>::{<items>};`
   - Replace every `use crate::` with `use bevyrogue::`
   - Replace inline `crate::` references inside test bodies with `bevyrogue::` (or keep them via re-imports)
4. **Move** the file: `git mv src/combat/<path>/tests.rs tests/<name>_internals.rs`.
5. **Delete** the `#[cfg(test)] mod tests;` line from the parent module (one line + the `#[cfg(test)]` attribute above it).
6. **Verify** before commit:
   - `cargo test --test <name>_internals` for the new file (preserves all tests)
   - `cargo check --tests` clean
7. **Commit** atomically: parent module edit + file move + import rewrite = single commit.

## Stop conditions

Halt and reassess if any of:

- A `crate::` path inside the test body resolves to a `pub(crate)` item (would have been caught during inventory but re-verify per wave).
- An import that looked `pub` at the source module turns out to be re-exported via a `pub(crate)` chain.
- A test relies on a `#[cfg(test)]`-gated helper in the parent module that wouldn't be visible from `tests/`.
- Any test changes behavior under the new visibility — re-run individually to compare.

## Final verify (Phase 3)

After all 3 waves committed:

```
cargo test --tests                       # 17 relocated tests + existing suite all green
cargo test --features windowed --tests   # all green
cargo check --tests                      # exit 0
cargo check --features windowed --tests  # exit 0
scripts/check_loc_cap.sh                 # 0 offenders
```

Re-scan `src/` for any remaining sibling `tests.rs` files and inline `#[cfg(test)] mod tests` blocks. Expected post-state: 4 binary-private inline blocks (131 LOC) remain; everything else relocated.

## Out of scope

- Splitting the relocated files into smaller per-feature integration files. Defer to a separate slice if/when files grow further.
- The 4 binary-private inline blocks — architectural slice, not Tier-D scope.

## Risk

**Low–medium.** Each file is large (300+ LOC) which raises the surface for a missed import path, but the audit confirmed all consumed symbols are `pub`. The risk is mechanical (typo in the `use bevyrogue::...` path, missing `crate::` → `bevyrogue::` substitution inside a function body); test compilation will catch any miss immediately.
