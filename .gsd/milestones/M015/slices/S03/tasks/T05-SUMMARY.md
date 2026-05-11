---
id: T05
parent: S03
milestone: M015
key_files:
  - tests/holy_support_affordance.rs
  - docs/combat_authority_map.md
  - docs/combat_mixed_pattern_drift_ledger.md
  - scripts/verify_combat_authority_audit.py
  - docs/m015_failure_ledger.md
  - scripts/verify_m015_failure_ledger.py
key_decisions:
  - Kept Holy Support proof on canonical state/events/snapshots instead of restoring the removed HolySupportAffordance API.
  - Strengthened authority verification around concrete S03 claims instead of broad keyword-only checks.
  - Classified whole-suite no-run failures by blocker class and owner rather than treating a single compiler front as exhaustive.
duration: 
verification_result: mixed
completed_at: 2026-05-08T16:29:29.899Z
blocker_discovered: false
---

# T05: Closed S03 by normalizing Holy Support tests to snapshots, refreshing authority docs/verifiers, and classifying remaining no-run blockers.

**Closed S03 by normalizing Holy Support tests to snapshots, refreshing authority docs/verifiers, and classifying remaining no-run blockers.**

## What Happened

Rewrote `tests/holy_support_affordance.rs` away from the removed HolySupportAffordance/action-query resource API and onto the current HolySupportState, rejected transition, and validation snapshot contract. Refreshed `docs/combat_authority_map.md` to describe the implemented flow `RON custom signals -> per-Digimon blueprint Rust module -> generic kernel hooks -> canonical state/events/snapshots`, while explicitly saying the Patamon seam is seeded rather than a full blueprint migration. Updated `docs/combat_mixed_pattern_drift_ledger.md` to mark D1/D2/D5/D6/D7 as S03-normalized, D3/D9 as partially normalized with S05 CLI proof remaining, D4 as seeded rewrite-scale follow-up, and D10 as broad fixture/doc follow-up. Strengthened `scripts/verify_combat_authority_audit.py` so it checks claim-scoped markers for the Patamon blueprint seam, live beat/kernel events, Holy Support snapshots, downstream boundaries, and tracked evidence paths. Replaced stale `docs/m015_failure_ledger.md` content with current targeted-pass and no-run-fail evidence, then refreshed `scripts/verify_m015_failure_ledger.py` to validate the new ledger shape. Final no-run still fails, but the failures are classified as broad fixture/schema and docs/consumer follow-up work rather than S03 targeted authority blockers.

## Verification

Ran formatter on the rewritten Holy Support test. Ran final targeted S03 verification: all listed Rust test targets passed, `scripts/verify_combat_authority_audit.py` passed, and `scripts/verify_m015_failure_ledger.py` passed. Ran `cargo test --no-run`; it still exits 101 due to classified broad-suite blockers including missing UI readiness doc include, stale SkillDef fixture constructors, duplicate UnitDef fields, and related S06/S05-owned follow-ups recorded in `docs/m015_failure_ledger.md`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo fmt -- tests/holy_support_affordance.rs` | 0 | ✅ pass | 218ms |
| 2 | `cargo test --test patamon_blueprint_seam --test holy_support_resolution --test holy_support_roster_contract --test holy_support_affordance --test event_stream --test twin_core_mechanics --test twin_core_integration --test validation_snapshot --test battery_loop_kernel --test predator_loop_kernel && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 233ms |
| 3 | `cargo test --no-run` | 101 | ❌ fail (classified in docs/m015_failure_ledger.md) | 1345ms |
| 4 | `python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 16ms |

## Deviations

Updated `scripts/verify_m015_failure_ledger.py` in addition to the task's expected files because the existing verifier encoded the stale pre-S03 ledger sections and would otherwise reject the refreshed failure ledger.

## Known Issues

`cargo test --no-run` still exits 101. Remaining blockers are documented in `docs/m015_failure_ledger.md`: S06 broad fixture/schema repairs (`SkillDef` defaults, `UnitDef` duplicate/missing fields), S06 docs artifact repair for `docs/combat_ui_readiness_gap_matrix.md`, and S05 CLI/consumer proof work.

## Files Created/Modified

- `tests/holy_support_affordance.rs`
- `docs/combat_authority_map.md`
- `docs/combat_mixed_pattern_drift_ledger.md`
- `scripts/verify_combat_authority_audit.py`
- `docs/m015_failure_ledger.md`
- `scripts/verify_m015_failure_ledger.py`
