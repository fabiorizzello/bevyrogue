# Summary — W0c Tier-B inline `mod tests` relocate

Closed: 2026-05-20
Parent: `260520-1-reduce-loc-tests` SUMMARY backlog #1.

## Outcome

5 of 7 candidate inline `mod tests` blocks relocated to `tests/<name>_internals.rs` via atomic per-file commits. 2 windowed-gated candidates skipped with rationale documented in PLAN.

## Commits

| Wave | Commit | Source | Target |
|---|---|---|---|
| W0c-5 | `fc36cbf` | `src/combat/runtime/registry.rs` (45) | `tests/registry_internals.rs` |
| W0c-6 | `bfef218` | `src/combat/state.rs` (43) | `tests/combat_state_internals.rs` |
| W0c-7 | — | `src/windowed/render.rs` (43) | **SKIPPED** — binary-private |
| W0c-8 | `04ec2ab` | `src/combat/mechanics/sp.rs` (40) | `tests/sp_mechanics_internals.rs` |
| W0c-9 | `707aec7` | `src/combat/runtime/event_filter.rs` (38) | `tests/event_filter_internals.rs` |
| W0c-10 | `7ef8c15` | `src/combat/mechanics/modifiers.rs` (34) | `tests/modifiers_internals.rs` |
| W0c-11 | — | `src/windowed/mod.rs` (31) | **SKIPPED** — binary-private |

**Relocated:** ~200 LOC out of `src/`. **Skipped (windowed binary-crate):** ~74 LOC, both <50 LOC and below R003 hard cap.

## Skipped waves rationale

`src/windowed/` lives in the binary crate `bevyrogue` (`src/main.rs:6 mod windowed;`), not the library. Integration tests in `tests/` only see the library's public API and cannot import `windowed::` symbols. Relocating these would require an architectural change (promote `windowed` to `pub mod` of the library, or hoist tested helpers into the library). Out of scope for an R003 LOC refactor. Both blocks remain at <50 LOC, well under the 100 LOC hard cap.

## Final verification

| Gate | Result |
|---|---|
| `cargo test --tests` (no features) | green |
| `cargo test --features windowed --tests` | green (user-confirmed) |
| `cargo check --tests` | exit 0 |
| `cargo check --features windowed --tests` | exit 0 |
| `scripts/check_loc_cap.sh` | 0 offenders |

## Post-state — residual inline `#[cfg(test)] mod tests` blocks ≥30 LOC

Matches PLAN's projection ("~159 LOC across 6 files + intentional outliers") **exactly**.

| File | LOC | Tier |
|---|---|---|
| `src/windowed/mod.rs` | 32 | Tier-B skipped (binary-private) |
| `src/windowed/render.rs` | 44 | Tier-B skipped (binary-private) |
| `src/headless.rs` | 30 | Tier-C |
| `src/combat/mechanics/buffs.rs` | 29 | Tier-C |
| `src/combat/mechanics/stun.rs` | 28 | Tier-C |
| `src/combat/encounter/bootstrap.rs` | 27 | Tier-C |
| `src/bin/combat_cli.rs` | 25 | Tier-C (binary-private) |
| `src/combat/observability/log.rs` | 20 | Tier-C |

Tier-C total: **159 LOC across 6 files** ✓

Also present (already migrated, orphan declarations only): `src/combat/runtime/runner.rs`, `src/combat/turn_system/mod.rs`, `src/combat/runtime/builtins.rs`.

## Backlog for follow-up

1. **Tier-C sweep** — 6 files × ~26 LOC avg. Defer until an organic touch on the host module brings it into a working set. Mechanical-only relocation has diminishing return below 30 LOC.
2. **Architectural slice for `src/windowed/`** — only path to relocate W0c-7/W0c-11. Pair with any future split of binary/library boundary; do not pursue standalone.
3. **Audit orphan `#[cfg(test)] mod tests;` declarations** — runner.rs, turn_system/mod.rs, builtins.rs all carry the attribute on a (presumably) module declaration line. Verify these still reference live external test modules and aren't dead.

## Risks confirmed retired

- The "remember `#[cfg(feature = "windowed")]` on relocated test file" risk from PLAN never materialized — the 2 windowed waves were skipped, so no windowed-gated `tests/*_internals.rs` exists.
- Tier-A pattern proven again across 5 more waves with zero regressions.
