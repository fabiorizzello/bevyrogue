---
id: T03
parent: S05
milestone: M011
key_files:
  - src/combat/unit.rs
  - src/combat/resolution.rs
  - src/combat/bootstrap.rs
  - src/combat/resolution_tests.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
key_decisions:
  - BasicStreak stored as a per-unit Component (not a Resource) since streak state is per-actor, consistent with the per-unit RoundEnergyTracker pattern from T02
  - Added BasicStreak as the 11th element of ResolveActorsQuery rather than a separate query — keeps all per-entity mutable data in one query and avoids a second entity lookup in step_app
  - Discount computed as effective_sp_cost before the SpPool spend check, so the discounted amount is what actually leaves the pool; streak reset occurs at the same moment (if discount fires)
  - Streak only increments on successful Basic (GainFromBasic branch after action executes) and only resets when discount fires — non-Basic non-discount actions leave streak count unchanged, matching the plan's minimal spec
duration: 
verification_result: passed
completed_at: 2026-04-27T20:19:12.222Z
blocker_discovered: false
---

# T03: Added evo_stage to Unit component, BasicStreak tracker, and Child skill discount (-1 SP after 2+ consecutive Basics) in resolution pipeline

**Added evo_stage to Unit component, BasicStreak tracker, and Child skill discount (-1 SP after 2+ consecutive Basics) in resolution pipeline**

## What Happened

Added `evo_stage: EvoStage` field to the Bevy `Unit` component (previously it existed on `UnitDef` in data layer but not on the runtime component). Created the `BasicStreak` component in `unit.rs` with `increment()`, `reset()`, and `qualifies_for_discount()` (count >= 2) methods.

Updated `bootstrap.rs` `spawn_unit_from_def` to populate `evo_stage: def.evo_stage` and insert `BasicStreak::default()` in the spawn bundle so all entities carry the tracker.

Added `basic_streak: &mut BasicStreak` parameter to `apply_effects` in `resolution.rs`. Before the SP spend check, the function now computes an `effective_sp_cost`: if the action is a Skill (UltEffect::None, sp_cost > 0), the attacker is Child stage, and the streak qualifies, it applies a -1 SP discount (min 0) and resets the streak. After a Basic action succeeds (UltEffect::GainFromBasic branch), it calls `basic_streak.increment()`.

Updated all call sites: `ResolveActorsQuery` in `turn_system/mod.rs` gained `Option<&'static mut BasicStreak>` as its 11th element; the local copy in `follow_up.rs` was updated to match; all destructuring patterns in `pipeline.rs` (`step_declaration`, `step_app` snapshot loop, `get_many_mut`) were updated; `step_app` extracts the attacker's streak via the 11th query element and passes it as `&mut *s` (or a local default if absent).

Added 5 new tests in `resolution_tests.rs`: Child after 2 Basics gets -1 SP on Skill; Adult after 2+ Basics gets no discount; Child with 1 Basic gets no discount; discount resets streak (and needs 2 more basics); Adult with 5 consecutive Basics gets no discount.

The `Unit` struct change required updating every test file that constructs `Unit` directly — 20 integration test files in `tests/` plus 3 inline test modules (`damage_tests.rs`, `follow_up_tests.rs`, `turn_system/tests.rs`) — by adding `evo_stage: EvoStage::Adult` and the corresponding `EvoStage` import to `types::{}` import lines.

## Verification

cargo test ran 341 tests, 0 failures. New Child discount tests verified both happy path (discount fires, streak resets, SP correctly reduced) and all negative cases (Adult, 1-Basic, post-discount-reset states).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | tail -5` | 0 | ✅ pass — 341 tests passed, 0 failed | 8200ms |

## Deviations

BasicStreak is the 11th element of ResolveActorsQuery rather than a separate query parameter as the plan suggested ('query BasicStreak from the attacker entity'). This reduces call-site plumbing while producing the same behavior. All 20 integration test files and 3 inline test modules needed evo_stage field and EvoStage import updates — the plan noted resolution_tests.rs but not the full scope of the Unit struct change.

## Known Issues

none

## Files Created/Modified

- `src/combat/unit.rs`
- `src/combat/resolution.rs`
- `src/combat/bootstrap.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
