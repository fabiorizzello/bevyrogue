# Summary — W0c Tier-C inline `mod tests` residue

Closed: 2026-05-20
Parent: `260520-2-w0c-tier-b-inline-tests` SUMMARY backlog #1.

## Outcome

4 of 6 candidate inline `mod tests` blocks relocated to `tests/<name>_internals.rs` via atomic per-file commits. 2 binary-private candidates skipped with rationale documented in PLAN.

## Commits

| Wave | Commit | Source | Target |
|---|---|---|---|
| W0c-12 | — | `src/headless.rs` (30) | **SKIPPED** — binary-private |
| W0c-13 | `c7bec9e` | `src/combat/mechanics/buffs.rs` (29) | `tests/buffs_internals.rs` |
| W0c-14 | `2958aee` | `src/combat/mechanics/stun.rs` (28) | `tests/stun_internals.rs` |
| W0c-15 | `e5d09ed` | `src/combat/encounter/bootstrap.rs` (27) | `tests/encounter_bootstrap_internals.rs` |
| W0c-16 | — | `src/bin/combat_cli.rs` (25) | **SKIPPED** — binary-private |
| W0c-17 | `676085b` | `src/combat/observability/log.rs` (20) | `tests/observability_log_internals.rs` |

**Relocated:** ~104 LOC out of `src/`. **Skipped (binary-private):** 55 LOC, both <50 LOC and well below R003 100 LOC hard cap.

## Course correction discovered during inventory

Tier-B SUMMARY listed `src/headless.rs` as a plain Tier-C target, but `src/main.rs:4 mod headless;` shows it lives in the binary crate, not the library. Same constraint as `src/windowed/` and `src/bin/combat_cli.rs` — integration tests in `tests/` only see the library's public surface. Relocation would require an architectural binary/library split; out of scope for an R003 LOC refactor. Documented and skipped.

## Skipped waves rationale

`src/headless.rs`, `src/bin/combat_cli.rs` — both live in binary crates (`mod headless;` from `src/main.rs`, and `src/bin/combat_cli.rs` is its own binary target). Integration tests cannot import symbols from binary crates. The inline blocks remain <50 LOC each, far below R003's 100 LOC hard cap. They stay in place; surface again only when an architectural slice promotes binary helpers into the library.

## Final verification

| Gate | Result |
|---|---|
| `cargo test --tests` (no features) | green |
| `cargo test --features windowed --tests` | green (all suites passing, 0 failures) |
| `cargo check --tests` | exit 0 |
| `cargo check --features windowed --tests` | exit 0 |
| `scripts/check_loc_cap.sh` | 0 offenders |

## Post-state — residual inline `#[cfg(test)] mod tests` blocks in `src/`

| File | LOC | Disposition |
|---|---|---|
| `src/windowed/render.rs` | 44 | Tier-B skipped (binary-private) |
| `src/windowed/mod.rs` | 32 | Tier-B skipped (binary-private) |
| `src/headless.rs` | 30 | Tier-C skipped (binary-private) |
| `src/bin/combat_cli.rs` | 25 | Tier-C skipped (binary-private) |

Total skipped inline LOC: **131**, all <50 LOC per file. Library-side inline tests: **0** (every `src/` module reachable via `pub mod` is now clean).

## Sibling `tests.rs` declarations (Tier-D scope)

Surfaced during Tier-B closeout, deferred to a separate workflow:

| Declaration | Target file | Tests | LOC | Access pattern |
|---|---|---|---|---|
| `src/combat/runtime/builtins.rs:330` | `src/combat/runtime/builtins/tests.rs` | 7 | 329 | `use super::*` (private items) |
| `src/combat/runtime/runner.rs:415` | `src/combat/runtime/runner/tests.rs` | 6 | 389 | `use super::*` (private items) |
| `src/combat/turn_system/mod.rs:30` | `src/combat/turn_system/tests.rs` | 4 | 303 | `use super::*` (private items) |

Total: **1,021 LOC of external sibling test files** — formally satisfy R003's "short `#[cfg(test)] mod tests`" wording but break its spirit. Each reaches into private items via `use super::*`, requiring per-file investigation of visibility seams (pubcrate-ify or rewrite as integration). Tier-D workflow to follow this closeout.

## Backlog for follow-up

1. **Tier-D sibling-tests sweep** — 3 files, 1,021 LOC, per-file `super::*` audit before relocation. High LOC payoff, non-mechanical.
2. **Architectural slice for `src/headless.rs`, `src/windowed/`, `src/bin/combat_cli.rs`** — only path to relocate the 131 LOC of remaining binary-private inline tests. Pair with any future binary/library boundary split.

## Risks confirmed retired

- The "headless.rs is plain Tier-C" assumption from the parent SUMMARY — caught during inventory before any code change.
- Tier-A/B pattern proven again across 4 more waves with zero regressions. Now applied to **9 source files** total across Tier-A+B+C.
