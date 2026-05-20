# Summary — W0d sibling `tests.rs` relocate

Closed: 2026-05-20
Parent: `260520-3-w0c-tier-c-residue` SUMMARY backlog #1.

## Outcome

All 3 sibling `tests.rs` files relocated to `tests/` as integration tests. 17 tests preserved (4 + 7 + 6), zero behavior changes, zero visibility promotions. The `src/` tree now contains zero library-side test code.

## Audit finding (called out)

Pre-work assumption — and the framing carried into Tier-C closeout — was that Tier-D would be **non-mechanical**: "per-file investigation of `super::*` usage before each move… probably a multi-session effort." The inventory audit reversed this. Per-file symbol scan revealed:

- `builtins/tests.rs` references the 16 private builtin fns (`deal_damage`, `apply_status`, etc.) **only as string keys** (`"core/deal_damage"`) into the hook registry. Direct function call count: zero.
- `runner/tests.rs` consumes only `BeatRunner`, `StepOutcome` — both `pub`. `runner.rs` has zero `pub(crate)` items.
- `turn_system/tests.rs` consumes only `resolve_action_system`, `check_victory_system`, `ActionIntent` — all already `pub` re-exports. The `pub(crate)` re-exports (`step_app`, `step_declaration`, etc.) are never touched.

The `use super::*` glob in each file was sloppy import style, not a real coupling to private items. Result: Tier-D collapsed from "1,021 LOC multi-session investigative slog" to a **30-minute mechanical sweep** matching the Tier-A/B/C pattern.

## Commits

| Wave | Commit | Source | Target | Tests |
|---|---|---|---|---|
| W0d-1 | `1cadf20` | `src/combat/turn_system/tests.rs` (303) | `tests/turn_system_internals.rs` | 4 |
| W0d-2 | `9cc9d93` | `src/combat/runtime/builtins/tests.rs` (329) | `tests/runtime_builtins_internals.rs` | 7 |
| W0d-3 | `b123dd2` | `src/combat/runtime/runner/tests.rs` (389) | `tests/runtime_runner_internals.rs` | 6 |

**Relocated:** 1,021 LOC out of `src/`. Each commit is a single-file `git mv` + import rewrite + parent module `#[cfg(test)] mod tests;` deletion. Rename detection on all three commits ≥89% (1cadf20: 95%, 9cc9d93: 89%, b123dd2: 98%) — confirms changes are purely import-path mechanical, not body rewrites.

## Per-file mechanics

For each file:

1. Rewrote `use super::*;` as explicit `use bevyrogue::combat::<path>::{<items>};` for the symbols actually consumed.
2. Replaced every `use crate::` with `use bevyrogue::`.
3. Replaced inline `crate::` paths inside test bodies with `bevyrogue::` (e.g., `crate::data::skill_timeline::SkillTimeline` → `bevyrogue::...`).
4. `git mv src/combat/<path>/tests.rs tests/<name>_internals.rs`.
5. Deleted `#[cfg(test)] mod tests;` from the parent module.
6. `cargo test --test <name>_internals` before commit (all 3 passed first try).

## Final verification

| Gate | Result |
|---|---|
| `cargo test --tests` (no features) | green |
| `cargo test --features windowed --tests` | green |
| `cargo check --tests` | exit 0 |
| `cargo check --features windowed --tests` | exit 0 |
| `scripts/check_loc_cap.sh` | 0 offenders |

## Post-state — residual inline `#[cfg(test)] mod tests` blocks in `src/`

| File | LOC | Disposition |
|---|---|---|
| `src/windowed/render.rs` | 44 | binary-private (Tier-B skipped) |
| `src/windowed/mod.rs` | 32 | binary-private (Tier-B skipped) |
| `src/headless.rs` | 30 | binary-private (Tier-C skipped) |
| `src/bin/combat_cli.rs` | 25 | binary-private (Tier-C skipped) |

Total skipped inline LOC: **131**, all <50 LOC per file. Sibling `tests.rs` files: **0** (all relocated). Library-side inline test code: **0**.

## Cumulative W0c+W0d achievement

Across Tier-A (4 commits), Tier-B (5 commits), Tier-C (4 commits), Tier-D (3 commits):

- **16 source files** had test code moved out of `src/`.
- **~1,500 LOC** of test code relocated to `tests/`.
- **Zero visibility promotions** required — every relocation was pure-relocate against existing `pub` symbols.
- **Zero regressions** — all verify gates remained green across all 16 commits.

R003 is now satisfied in spirit, not just letter, for every library module. Only the binary-crate inline blocks remain, gated by an architectural choice (split bin/library boundary) rather than mechanical refactoring.

## Backlog for follow-up

1. **Architectural slice for binary-crate test residue** — only path to relocate the 131 LOC across `src/headless.rs`, `src/windowed/{mod,render}.rs`, `src/bin/combat_cli.rs`. Pair with any future binary/library split; do not pursue standalone.

## Risks confirmed retired

- The "Tier-D requires per-file visibility audit" framing — replaced by an actual audit that proved pure-relocate feasibility before any code change.
- The "use super::* means private coupling" heuristic — disproven for this codebase; in all 3 cases the glob was idiomatic noise, not a load-bearing dependency.
