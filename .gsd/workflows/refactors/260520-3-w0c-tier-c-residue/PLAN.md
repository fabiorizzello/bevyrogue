# Plan — W0c Tier-C inline `mod tests` residue

Draft. Status: ready (Phase 2).
Parent: `260520-2-w0c-tier-b-inline-tests` SUMMARY backlog #1.

## Wave breakdown

One atomic commit per source file. Wave numbering continues the Tier-B sequence (last was W0c-11). Tier-C runs W0c-12 through W0c-17.

| Wave | Source | Target | Disposition | Commit format |
|---|---|---|---|---|
| W0c-12 | `src/headless.rs` (30 LOC) | — | **SKIPPED** — bin-private | n/a (see "Skipped waves") |
| W0c-13 | `src/combat/mechanics/buffs.rs` (29 LOC) | `tests/buffs_internals.rs` | relocate | `refactor(tests): W0c-13 — relocate buffs inline tests` |
| W0c-14 | `src/combat/mechanics/stun.rs` (28 LOC) | `tests/stun_internals.rs` | relocate | `refactor(tests): W0c-14 — relocate stun inline tests` |
| W0c-15 | `src/combat/encounter/bootstrap.rs` (27 LOC) | `tests/encounter_bootstrap_internals.rs` | relocate | `refactor(tests): W0c-15 — relocate encounter bootstrap inline tests` |
| W0c-16 | `src/bin/combat_cli.rs` (25 LOC) | — | **SKIPPED** — bin-private | n/a (see "Skipped waves") |
| W0c-17 | `src/combat/observability/log.rs` (20 LOC) | `tests/observability_log_internals.rs` | relocate | `refactor(tests): W0c-17 — relocate observability log inline tests` |

Relocations: 4 files, ~104 LOC moved out of `src/`. Skips: 2 files, 55 LOC, deferred to architectural slice.

## Skipped waves rationale

**W0c-12 (`src/headless.rs`)** — declared by `src/main.rs:4 mod headless;`; not in the library crate. Integration tests in `tests/` only see the library's public surface and cannot import `headless::` symbols (`CombatScript`, `ScriptStep`, etc.). Same constraint that excluded W0c-7/W0c-11.

**W0c-16 (`src/bin/combat_cli.rs`)** — files under `src/bin/` are each their own binary crate. The inline block reaches private items in that binary; tests in `tests/` cannot import them.

Both inline blocks remain <50 LOC (well below R003 100 LOC hard cap). They stay in place; surface again only when an architectural slice splits binary/library boundary, or when those modules grow large enough that promoting helpers to the library becomes worthwhile on independent grounds.

## Per-wave procedure

For each relocate wave (W0c-13, 14, 15, 17):

1. **Read** the inline `mod tests` block end-to-end (already done during inventory; symbols all `pub`).
2. **Create** `tests/<name>_internals.rs` with crate-prefixed imports.
   - `use bevyrogue::combat::mechanics::buffs::{DrBag, sum_dr};` (W0c-13)
   - `use bevyrogue::combat::mechanics::stun::Stunned;` (W0c-14)
   - `use bevyrogue::combat::encounter::bootstrap::{spawn_unit_from_def, taichi_def}; use bevyrogue::combat::unit::Commander;` (W0c-15)
   - `use bevyrogue::combat::observability::log::{ActionLog, LogEntry}; use bevyrogue::combat::types::UnitId;` (W0c-17)
3. **Delete** the inline `#[cfg(test)] mod tests { … }` block from the source file (trailing block, so straight truncation).
4. **Verify** before commit:
   - `cargo test --test <name>_internals` for the new file (sanity)
   - `cargo check --tests` clean
5. **Commit** atomically: one source edit + one test file create per commit.

For skip waves (W0c-12, W0c-16): no commits, rationale documented above and rolled into the closing SUMMARY.

## Stop conditions

Halt and downgrade to "skip with note" if any of:

- A test asserts on a private invariant — unexpected here since every touched symbol scanned as `pub` during inventory, but re-verify per wave.
- Adding the integration test file breaks `cargo test --features windowed --tests` (none of these sources are windowed-gated, so no `#[cfg(feature = "windowed")]` needed on the relocated files).
- The source file's trailing `}` from the deleted block leaves a syntax artifact — read once after edit, lsp diagnostics if any doubt.

## Final verify (Phase 3)

After W0c-13/14/15/17 committed:

```
cargo test --tests                       # all green
cargo test --features windowed --tests   # all green
cargo check --tests                      # exit 0
cargo check --features windowed --tests  # exit 0
scripts/check_loc_cap.sh                 # 0 offenders
```

Re-scan `src/` for inline `#[cfg(test)] mod tests` blocks ≥10 LOC. Expected residue: only the 3 binary-private skipped files (headless.rs, windowed/render.rs, windowed/mod.rs, bin/combat_cli.rs — 4 in total; 5 if counting the orphan declarations).

## Out of scope

- Tier-D sibling `tests.rs` files (1,021 LOC). Separate workflow opens after this one closes.
- Architectural slice promoting `headless` / `windowed` / `bin/combat_cli` helpers into the library.

## Risk

Low. All 4 relocations are pure-relocate against `pub` symbols, no feature gates, no private-item promotion. Tier-A/B applied this pattern across 9 commits with zero regressions. The only novel finding (`headless.rs` is bin-private) was caught during inventory before any code change.
