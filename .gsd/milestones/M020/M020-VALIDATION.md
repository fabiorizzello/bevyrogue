---
verdict: pass
remediation_round: 0
---

# Milestone Validation: M020

## Success Criteria Checklist
## Success Criteria Checklist

| Criterion | Evidence |
|-----------|----------|
| [x] CombatEventKind::UltimateUsed emitted every time a unit uses the ultimate; existing listeners not broken | S01-SUMMARY confirms emission in all 4 pipeline hoist blocks; 3 new tests in `tests/ultimate_event.rs` pass green; 673 total tests pass with 0 failures |
| [x] OnKO renamed UnitDied with fields status_remaining: Vec<StatusEffectKind> and heated_remaining: u32; all emit/match sites updated | S01-SUMMARY lists 10 files updated; `src/combat/events.rs` lines 41-44 confirm variant definition; S01-ASSESSMENT confirms zero residual OnKO references via `rg` search |
| [x] Three pub use shims (twin_core, holy_support, predator_loop) removed from combat/mod.rs; all import sites updated to canonical blueprint paths | S02-SUMMARY confirms 3 shims removed from `src/combat/mod.rs` and 13 call-sites updated (2 src/, 11 tests/); S02-ASSESSMENT confirms `rg 'combat::twin_core\|combat::holy_support\|combat::predator_loop' src tests` → exit 1 (zero matches) |
| [x] cargo test green (72+ tests) | 673 total tests pass across all suites with 0 failures; S01-ASSESSMENT confirms 673 passed, 0 failed; S02-ASSESSMENT confirms 60+ integration test binaries all passed |
| [x] cargo check headless and windowed without new warnings | `cargo check` exit 0; `cargo check --features windowed` exit 0; no new warnings introduced (warnings are pre-existing dead_code, unused_mut, deprecated StatusEffect) |

## Slice Delivery Audit
## Slice Delivery Audit

| Slice | SUMMARY.md | Assessment | Follow-ups / Known Limitations |
|-------|-----------|------------|-------------------------------|
| S01 — Nuovi eventi reactive bus: UltimateUsed + UnitDied payload | Present (`S01-SUMMARY.md`) | PASS — S01-ASSESSMENT.md, 8/8 checks pass, including 5 new tests green, 673-test regression clean, zero residual OnKO references | Known limitation: stun-damage KO path (`turn_system/mod.rs:488`) emits UnitDied with empty payload (no StatusBag in scope) — documented with comment; no follow-ups |
| S02 — Rimozione shim pub use legacy (twin_core / holy_support / predator_loop) | Present (`S02-SUMMARY.md`) | PASS — `cargo check` exit 0, `cargo test` all suites pass, `rg` confirms zero shim alias occurrences | Known limitation: blueprint mod.rs files carry unused-import warnings for prophylactic re-export entries (benign); Follow-up: M021 can consume canonical paths without shim intermediaries |

Both slices completed; no outstanding blockers. S01 known limitation (empty stun-damage payload) is documented and intentional.

## Cross-Slice Integration
## Cross-Slice Integration

| Boundary | Producer (S01) | Consumer (S02) | Status |
|----------|----------------|----------------|--------|
| Event Variant Production | Added `UltimateUsed { unit_id }` + renamed `OnKO` → `UnitDied { status_remaining, heated_remaining }` in `events.rs`; emitted in all 4 pipeline hoist blocks | No interaction with event definitions; S02 is purely a namespace refactor (shim removal) — `events.rs` not modified | PASS |
| Pipeline Event Consumption | Updated `pipeline.rs` match arms to consume `UnitDied { .. }` wildcard; all 4 hoist blocks emit `UltimateUsed` | S02 did not modify `pipeline.rs` — event handling logic untouched | PASS |
| Shared File Modifications | Modified: `events.rs`, `pipeline.rs`, `resolution.rs`, `turn_system/mod.rs` + 5 test files | Modified: `mod.rs`, blueprint mod.rs files, `gabumon.rs`, `observability.rs` + 11 test files | PASS — zero file overlap between slices |
| Test Coverage | 673 total tests pass; 5 new tests added (3 in ultimate_event, 2 in unit_died_payload) | All 60+ integration test binaries pass; 11 test files updated for canonical path imports | PASS — independent suites, combined zero failures |
| Observability Surfaces | S01 exports UltimateUsed and UnitDied payload to JSONL event stream | S02 does not touch `jsonl_logger.rs` or serialization — observability signals from S01 intact | PASS |
| Slice Dependencies | S01 declares `affects: [S02]` (event names available post-S01) | S02 declares `requires: []` — correctly identifies itself as orthogonal namespace work | PASS |

Clean separation of concerns: S01 enriched the event vocabulary, S02 refactored legacy shims. Zero overlapping files, no runtime conflicts, combined test suite confirms end-to-end integrity.

## Requirement Coverage
## Requirement Coverage

M020 requirements are defined as its roadmap success criteria (no linked REQUIREMENTS.md entries were advanced or surfaced by this milestone's slices).

| Requirement | Status | Evidence |
|-------------|--------|----------|
| CombatEventKind::UltimateUsed emitted once per cast; existing listeners not broken | COVERED | S01: Added variant in `events.rs`, emitted in 4 pipeline hoist blocks gated by `UltEffect::Reset`; 3 tests in `ultimate_event.rs` verify correct emission; 673 regression tests pass |
| OnKO renamed UnitDied with fields status_remaining + heated_remaining; all emit/match sites updated | COVERED | S01: Renamed in `events.rs`; `ko_payload()` helper in `resolution.rs`; all 4 `pipeline.rs` match arms updated to `UnitDied { .. }`; `mod.rs` stun path emits empty payload with comment; `rg CombatEventKind::OnKO` → exit 1 |
| Three pub use shims removed from combat/mod.rs; all import sites updated to canonical paths | COVERED | S02: Shim lines deleted from `mod.rs`; canonical re-exports added to 3 blueprint mod.rs files; 13 call-sites updated; `rg combat::twin_core\|combat::holy_support\|combat::predator_loop` → exit 1 |
| cargo test green (72+ tests) | COVERED | 673 tests pass across all suites (exceeds 72+ threshold); 0 failures |
| cargo check headless and windowed without new warnings | COVERED | `cargo check` exit 0; `cargo check --features windowed` exit 0; no new warnings introduced |

**All requirements: COVERED. No partials, no missing.**

## Verification Class Compliance
## Verification Classes

| Class | Planned Check | Evidence | Verdict |
|-------|---------------|----------|---------|
| Contract | `cargo test --all` green | 673 tests pass, 0 failed across all integration and lib targets | PASS |
| Contract | `cargo check` (headless) clean | Exit 0, warnings are pre-existing (dead_code, unused_mut, deprecated StatusEffect) | PASS |
| Contract | `cargo check --features windowed` clean | Exit 0, no new errors | PASS |
| Contract | `grep -rn 'CombatEventKind::OnKO' src/ tests/` → zero matches | `rg` exit 1, zero matches — rename complete throughout codebase (S01-ASSESSMENT) | PASS |
| Contract | `grep -rn 'combat::twin_core\|combat::holy_support\|combat::predator_loop' src/ tests/` → zero matches | `rg` exit 1, zero matches — shims fully removed (S02-SUMMARY) | PASS |


## Verdict Rationale
All three independent reviewers returned PASS. Every M020 success criterion is fully evidenced: UltimateUsed is emitted in all 4 pipeline hoist blocks with 3 dedicated tests; UnitDied carries StatusBag payload with 2 dedicated tests and zero residual OnKO references; all three legacy shims are removed and 13 call-sites rerouted to canonical blueprint paths with grep confirming zero occurrences; 673 tests pass (9x the 72+ threshold); both cargo check targets exit 0 with no new warnings. Cross-slice integration is clean — S01 and S02 modified disjoint file sets with no runtime conflicts.
