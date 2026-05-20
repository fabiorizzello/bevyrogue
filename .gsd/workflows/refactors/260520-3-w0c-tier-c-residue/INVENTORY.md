# Inventory — W0c Tier-C inline `mod tests` residue

Created: 2026-05-20
Parent workflow: `260520-2-w0c-tier-b-inline-tests` (closed; backlog item #1)
Scope: 6 inline `#[cfg(test)] mod tests` blocks, 20–30 LOC each, all well below R003 hard cap. Mechanical tail of the W0c sweep.

## Goal

Close out the remaining inline-test backlog identified in the Tier-B SUMMARY. All targets are pure-relocate candidates against `pub` symbols, with the exception of 2 binary-private files that must be skipped (same constraint that excluded W0c-7/W0c-11).

## Candidates (verified 2026-05-20 against HEAD `f42a5f3`)

| # | Source file | Inline block lines | LOC | Crate | Disposition |
|---|---|---|---|---|---|
| 1 | `src/headless.rs` | 401–430 | 30 | **binary** (`src/main.rs:4 mod headless;`) | **SKIP** — bin-private |
| 2 | `src/combat/mechanics/buffs.rs` | 51–79 | 29 | library | relocate (3 tests, `DrBag`/`sum_dr` all `pub`) |
| 3 | `src/combat/mechanics/stun.rs` | 18–45 | 28 | library | relocate (3 tests, `Stunned`/`tick` all `pub`) |
| 4 | `src/combat/encounter/bootstrap.rs` | 242–268 | 27 | library | relocate (2 tests, `spawn_unit_from_def`/`taichi_def` both `pub`) |
| 5 | `src/bin/combat_cli.rs` | 243–267 | 25 | **binary** (`src/bin/`) | **SKIP** — bin-private |
| 6 | `src/combat/observability/log.rs` | 57–76 | 20 | library | relocate (1 test, `ActionLog`/`LogEntry` both `pub`) |

Totals: **104 LOC relocatable** (4 files), **55 LOC binary-skip** (2 files), **159 LOC** overall — matches Tier-B SUMMARY's projection.

## Course correction from Tier-B planning

Tier-B SUMMARY tagged `src/headless.rs` as Tier-C plain, but `src/main.rs:4` declares `mod headless;` — it's bin-private, not library. Confirmed: `src/lib.rs` does not declare `mod headless`. So `headless.rs` joins `windowed/` and `bin/combat_cli.rs` as a binary-private skip, on the same R003-out-of-scope rationale.

## Reference pattern (from Tier-A/B)

All 4 relocatable candidates are clean cases of the **pure relocate** path:
1. Every symbol the test touches is already `pub`.
2. `git mv` semantics: delete the inline block, create `tests/<name>_internals.rs` with the same body and crate-prefixed imports.

No visibility promotion or integration rewrite needed for this tier.

## Out of scope

- **Tier-D** (newly surfaced): 3 sibling `tests.rs` files totalling 1,021 LOC reach into private items via `use super::*` — `src/combat/runtime/builtins/tests.rs`, `src/combat/runtime/runner/tests.rs`, `src/combat/turn_system/tests.rs`. Separate workflow.
- **Architectural slice for `src/headless.rs`, `src/windowed/`, `src/bin/combat_cli.rs`** — only path to relocate the binary-private blocks. Pair with any future binary/library split, not standalone.

## Acceptance

- All 4 library targets no longer contain an inline `#[cfg(test)] mod tests` block.
- Corresponding `tests/<name>_internals.rs` exists for each relocated block.
- `cargo test --tests` and `cargo test --features windowed --tests` green.
- `cargo check --tests` and `cargo check --features windowed --tests` clean.
- `scripts/check_loc_cap.sh` still passes (0 offenders).
- `src/headless.rs` and `src/bin/combat_cli.rs` blocks remain in place, documented as binary-private skips.
