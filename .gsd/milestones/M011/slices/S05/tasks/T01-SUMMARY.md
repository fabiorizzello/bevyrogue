---
id: T01
parent: S05
milestone: M011
key_files:
  - src/combat/sp.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution_tests.rs
  - tests/sp_economy.rs
  - tests/patamon_revive.rs
  - tests/validation_snapshot.rs
  - tests/combat_coherence.rs
key_decisions:
  - Used local RoundSpTracker::default() in pipeline.rs step_app rather than adding it to the Bevy system parameters — avoids modifying 17 test files since no non-Basic SP paths exist yet and the tracker has no cross-turn state to preserve
  - Changed revive skill costs from 6 to 5 in test fixtures where max=5 made cost-6 revives structurally impossible; this preserves the economic intent (revive requires full pool) while being achievable under the new cap
  - Redesigned sp_economy 20-turn sequence: removed the ally_skill_3 drain step, replaced with 5 basics rebuild, keeping total turns at 20 and final SP==2 assertion unchanged
duration: 
verification_result: passed
completed_at: 2026-04-27T19:59:57.922Z
blocker_discovered: false
---

# T01: Fixed SpPool max=5, wired RoundSpTracker into apply_effects, and adapted all affected tests to the new SP economy

**Fixed SpPool max=5, wired RoundSpTracker into apply_effects, and adapted all affected tests to the new SP economy**

## What Happened

sp.rs already had SpGainSource, RoundSpTracker, and SpPool::default().max=5 from a prior partial session. The gap was that pipeline.rs called apply_effects with 7 args but the signature now required 8 (sp_tracker). Fixed by importing RoundSpTracker in pipeline.rs and creating a local default per-action in step_app (no cross-turn persistence needed yet since no non-Basic SP paths exist).

resolution_tests.rs had 8 apply_effects call sites; all updated to pass &mut RoundSpTracker::default() as the new 7th argument. Removed unused SpGainSource import from resolution.rs and prefixed _sp_tracker to suppress the unused-variable warning.

Swept all max:10 SpPool usages across test files. validation_snapshot.rs: changed SpPool to {current:5,max:5} and updated two hardcoded snapshot assertion strings (sp=7/10→sp=5/5, sp=3/10→sp=3/5). patamon_revive.rs: changed pool to max:5 and revive cost from 6→5 (6 > max is impossible). combat_coherence.rs: changed pool to max:5, holy_revive cost from 6→5, the gate assertion from sp==3 to sp==2, and the window check from [10,7,3] to [5,2,2] to match the new drain sequence.

sp_economy.rs required the most restructuring: revive cost 6→5, starting pool {current:5,max:5}, replaced the two-skill drain (skill_4+skill_3) with a single skill_4 drain (5→1), 5 basics to rebuild (loop 0..5 instead of 0..6), updated two SP-spent assertion messages from "full 6 SP" to "full 5 SP". The 20-turn sequence and final SP==2 assertion remain valid under the new economy.

Added 4 unit tests in src/combat/sp.rs: SpPool default max is 5, RoundSpTracker caps non-Basic at +2, reset restores full budget, partial gain tracks correctly.

All 29 test suites pass: 126 unit tests + all integration tests, 0 failures.

## Verification

cargo test — all 29 test suites pass with 0 failures. Verified: SpPool::default().max==5 (unit test), RoundSpTracker clamps +3 gain to +2 (unit test), grep 'max: 10' src/combat/sp.rs returns nothing.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | grep -E 'test result'` | 0 | ✅ pass | 12000ms |
| 2 | `grep -r 'max: 10' src/combat/sp.rs || echo CLEAN` | 0 | ✅ pass | 50ms |

## Deviations

The task plan said to sweep max:10 → max:5 in all listed files. Several test files had 6-cost revive skills that exceed the new max=5, requiring skill costs to be updated alongside the pool change. sp_economy.rs and combat_coherence.rs also had hardcoded SP window assertions that referenced the old economy values and needed updating. All adaptations preserve the original test intents (blocked revive, rebuild, succeed).

## Known Issues

RoundSpTracker is created fresh per action in pipeline.rs (not persisted as a Bevy resource). This means cross-round SP cap enforcement via the tracker is not wired into the live game loop yet — it's exercised only in unit tests. This is by design per the task plan ("the non-Basic path is exercised only by tests"), and will be addressed when non-Basic SP gain paths are added.

## Files Created/Modified

- `src/combat/sp.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution_tests.rs`
- `tests/sp_economy.rs`
- `tests/patamon_revive.rs`
- `tests/validation_snapshot.rs`
- `tests/combat_coherence.rs`
