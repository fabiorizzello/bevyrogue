---
id: M020
title: "Reactive bus uniforme + shim removal"
status: complete
completed_at: 2026-05-14T10:57:48.792Z
key_decisions:
  - Emit UltimateUsed in all 4 pipeline hoist blocks symmetrically with UltGain — every UltEffect::Reset path emits the event once per cast, covering all fan-out widths
  - ko_payload() helper in resolution.rs centralizes StatusBag snapshot extraction — avoids duplication and makes the empty-payload stun-damage edge case explicit by contrast
  - stun-damage KO path (turn_system/mod.rs) emits UnitDied with empty payload (no StatusBag in scope) — documented with comment; known gap for future milestone
  - Blueprint mod.rs re-exports use pub use identity::{...} to keep canonical consumer surface at blueprints::<name>::<Type>, hiding the identity sub-module
  - Compiler-driven refactor for shim removal: remove alias first, let compiler enumerate all affected files, fix in one pass — more reliable than grep for Rust (found 11 files vs 9 predicted)
key_files:
  - src/combat/events.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/mod.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/patamon/mod.rs
  - src/combat/blueprints/dorumon/mod.rs
  - src/combat/blueprints/gabumon.rs
  - src/combat/observability.rs
  - tests/ultimate_event.rs
  - tests/unit_died_payload.rs
lessons_learned:
  - stun-damage KO path emits UnitDied with empty payload — no StatusBag in scope at turn_system/mod.rs stun site; downstream listeners must guard against empty status_remaining/heated_remaining for stun-triggered kills
  - Compiler-driven call-site enumeration is more reliable than grep for Rust alias removal — grep undercounted by 2 files (9 predicted vs 11 actual) due to multi-line use declarations and intermediate re-exports
  - New event variants at resource-consumption sites must be emitted on every branch performing that action, not just the outermost entry — emitting only at the top-level fan-out misses inner paths
---

# M020: Reactive bus uniforme + shim removal

**Completed the combat event bus with UltimateUsed and UnitDied payload variants, and eliminated all three legacy pub use shims from combat/mod.rs, leaving M021 a clean namespace and an informative event stream.**

## What Happened

M020 delivered two orthogonal but complementary changes to the combat kernel.

**S01 — Reactive bus enrichment:** Added `CombatEventKind::UltimateUsed { unit_id }` emitted symmetrically in all 4 resource hoist blocks of `pipeline.rs` (Single, Blast, AllEnemies, AllAllies fan-out paths) using the same source/target=attacker_id shape as the peer `UltGain` event. Renamed `OnKO` to `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` with a `ko_payload()` helper in `resolution.rs` that centralizes the StatusBag snapshot at emission time. The stun-damage KO path in `turn_system/mod.rs` emits an empty payload (no StatusBag in scope) — documented with a comment as a known limitation. All 4 `pipeline.rs` match arms updated to `UnitDied { .. }` wildcard; 10 files updated in total. Five new dedicated tests added: 3 in `tests/ultimate_event.rs` and 2 in `tests/unit_died_payload.rs`.

**S02 — Legacy shim removal:** Deleted the three `pub use` shim lines for `twin_core`, `holy_support`, and `predator_loop` from `src/combat/mod.rs`. Added canonical `pub use identity::{...}` re-exports to the three blueprint mod.rs files (agumon, patamon, dorumon) so the consumer surface remains at `blueprints::<name>::<Type>`. Used a compiler-driven refactor approach — removing aliases first then fixing all compiler errors — which found 11 affected test files vs the 9 predicted by grep. All 13 call-sites (2 src/, 11 tests/) updated atomically in one commit.

Both slices modified disjoint file sets with zero runtime conflicts. Combined test suite: 673 tests, 0 failures. Both `cargo check` and `cargo check --features windowed` exit 0 with no new warnings.

## Success Criteria Results

| Criterion | Status | Evidence |
|-----------|--------|----------|
| CombatEventKind::UltimateUsed emitted every time a unit uses the ultimate; existing listeners not broken | PASS | Added variant in events.rs; emitted in all 4 pipeline hoist blocks gated by UltEffect::Reset; 3 tests in ultimate_event.rs verify correct emission; 673 regression tests pass |
| OnKO renamed UnitDied with fields status_remaining + heated_remaining; all emit/match sites updated | PASS | Renamed in events.rs; ko_payload() helper in resolution.rs; all 4 pipeline.rs match arms updated to UnitDied { .. }; stun path emits empty payload with comment; rg CombatEventKind::OnKO → exit 1 (zero matches) |
| Three pub use shims removed from combat/mod.rs; all import sites updated to canonical blueprint paths | PASS | Shim lines deleted from mod.rs; canonical re-exports added to 3 blueprint mod.rs files; 13 call-sites updated; rg combat::twin_core\|combat::holy_support\|combat::predator_loop → exit 1 (zero matches) |
| cargo test green (72+ tests) | PASS | 673 tests pass across all suites (9x threshold); 0 failures |
| cargo check headless and windowed without new warnings | PASS | cargo check exit 0; cargo check --features windowed exit 0; warnings are pre-existing dead_code/unused_mut/deprecated StatusEffect |

## Definition of Done Results

| Item | Status |
|------|--------|
| All slices complete [x] | PASS — S01: complete (2/2 tasks), S02: complete (1/1 task) |
| SUMMARY.md present for each slice | PASS — S01-SUMMARY.md, S02-SUMMARY.md both present |
| Cross-slice integration works | PASS — S01 and S02 modified disjoint file sets; S02 correctly consumed the UnitDied rename from S01; combined 673-test suite confirms end-to-end integrity |
| Zero residual OnKO references | PASS — rg exit 1 |
| Zero shim alias references | PASS — rg exit 1 |
| No new warnings introduced | PASS — pre-existing warnings only |

## Requirement Outcomes

No REQUIREMENTS.md entries were linked to M020. All milestone success criteria served as the requirement contract and are fully covered (see Success Criteria Results). No requirement status transitions needed.

## Deviations

S02 affected 11 test files vs the 9 estimated in the plan. The extra files (validation_snapshot.rs, status_observability_canon.rs, presentation_metadata_boundary.rs) used legacy shim paths via multi-line use declarations not caught by the initial grep scan. All were fixed atomically in the same commit; no functional impact.

## Follow-ups

M021 can now consume blueprint types via combat::blueprints::<name>::<Type> without shim intermediaries — the canonical surface is stable. If a future milestone needs full StatusBag payload on stun-triggered kills, thread StatusBag into the stun-damage path in turn_system/mod.rs or emit a second enrichment event after stun resolution. Blueprint mod.rs files carry benign unused-import warnings for prophylactic re-export entries — can be trimmed in a future housekeeping pass.
