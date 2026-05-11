# S05: Resource caps (R073) + Child mechanics

**Goal:** Enforce per-round resource caps on SP (+2 non-Basic/round) and Energy (10 secondary + 30 external/round), and implement Child-specific mechanics (skill discount after 2 consecutive Basics). All three subsystems are data-driven, tested, and wired into the resolution pipeline.
**Demo:** scenario CLI: 3 turni di Basic con un Child mostrano discount al 3° skill; cap SP enforced via test che prova a sforare

## Must-Haves

- SpPool max=5, non-Basic SP gain capped at +2 per acting-unit turn
- Energy component exists with max 100 and per-turn gain caps (10 secondary, 30 external)
- Child units get -1 SP cost on next Skill after 2 consecutive Basics; discount consumed on use
- All existing tests updated for max=5 and passing
- New integration test exercises the slice demo scenario

## Proof Level

- This slice proves: - This slice proves: contract + integration
- Real runtime required: no (headless tests sufficient)
- Human/UAT required: no

## Integration Closure

- Upstream surfaces consumed: `src/combat/sp.rs` (SpPool), `src/combat/resolution.rs` (apply_effects), `src/combat/unit.rs` (Unit component), `src/combat/bootstrap.rs` (spawn_unit_from_def), `src/combat/types.rs` (EvoStage)
- New wiring introduced: `src/combat/energy.rs` module registered in `src/combat/mod.rs`; `evo_stage` field on Unit component; `BasicStreak` component; `RoundSpTracker` resource
- What remains: S06 (Tempo Resistance), S07 (Toughness categories), S08 (Form Identity consumes Energy)

## Verification

- Not provided.

## Tasks

- [x] **T01: Fix SpPool max=5 and add RoundSpTracker with per-turn cap** `est:45m`
  R073 requires SP max=5 (not 10) and a cap of +2 non-Basic SP gain per round. This task changes SpPool::default() max to 5, adds a RoundSpTracker resource that enforces the +2 cap, updates apply_effects to route SP gain through the tracker, and sweeps all test files that construct SpPool with max:10.

## Steps

1. In `src/combat/sp.rs`: change `SpPool::default()` max from 10 to 5. Add `SpGainSource` enum with `Basic` and `NonBasic` variants. Add `RoundSpTracker` resource with `non_basic_gained: i32` field and `try_gain_non_basic(amount: i32) -> i32` method that clamps to remaining cap (2 - non_basic_gained). Add `reset()` method.
2. In `src/combat/resolution.rs`: update `apply_effects` signature to accept `&mut RoundSpTracker`. In the `UltEffect::GainFromBasic` arm, keep `sp.gain(1)` as-is (Basic SP is uncapped). For future non-Basic SP gains, route through tracker. No non-Basic SP gain paths exist yet in apply_effects — the tracker is wired but the non-Basic path is exercised only by tests.
3. Sweep all files that construct `SpPool { ... max: 10 }` and update to `max: 5`. Files: `src/combat/resolution_tests.rs`, `src/combat/damage_tests.rs`, `src/combat/turn_system/tests.rs`, `src/combat/enemy_ai.rs`, `tests/patamon_revive.rs`, `tests/sp_economy.rs`, `tests/validation_snapshot.rs`, `tests/combat_coherence.rs`, `tests/status_accuracy.rs`, `tests/damage_breakdown_log.rs`, `tests/status_effect_apply.rs`. For tests using `max: 100` as an effectively-unlimited pool, leave them or set to a high value — the point is functional, not cosmetic.
4. Add unit tests in `src/combat/sp.rs` `#[cfg(test)] mod tests`: (a) RoundSpTracker caps non-Basic gain at +2, (b) Basic gain is uncapped by tracker, (c) SpPool default max is 5.
5. Run `cargo test` — all tests pass.

## Must-Haves

- [ ] SpPool::default().max == 5
- [ ] RoundSpTracker enforces +2 non-Basic cap per reset cycle
- [ ] All existing tests compile and pass with new max

## Verification

- `cargo test` passes with zero failures
- `grep -r 'max: 10' src/combat/sp.rs` returns nothing

## Negative Tests

- Attempt to gain +3 non-Basic SP in one round → only +2 applied
- After tracker reset, full +2 budget is available again
  - Files: `src/combat/sp.rs`, `src/combat/resolution.rs`, `src/combat/resolution_tests.rs`, `src/combat/damage_tests.rs`, `src/combat/turn_system/tests.rs`, `src/combat/enemy_ai.rs`, `tests/patamon_revive.rs`, `tests/sp_economy.rs`, `tests/validation_snapshot.rs`, `tests/combat_coherence.rs`, `tests/status_accuracy.rs`, `tests/damage_breakdown_log.rs`, `tests/status_effect_apply.rs`
  - Verify: cargo test 2>&1 | tail -5

- [x] **T02: Create Energy component and RoundEnergyTracker with per-turn gain caps** `est:30m`
  R073 requires per-unit Energy (max 100) with per-turn gain caps: 10 from secondary actions, 30 from external sources. No Energy component exists in the codebase. This task creates the module and wires it into mod.rs. Energy is not consumed by anything in this slice — S08 (Form Identity) will be the first consumer.

## Steps

1. Create `src/combat/energy.rs` with:
   - `Energy` component: `current: i32`, `max: i32` (default 100). Methods: `gain(amount: i32)`, `spend(amount: i32) -> bool`, `is_full() -> bool`.
   - `EnergyGainSource` enum: `SecondaryAction`, `External`.
   - `RoundEnergyTracker` component (per-unit, not resource): `secondary_gained: i32` (cap 10), `external_gained: i32` (cap 30). Method `try_gain(source: EnergyGainSource, amount: i32) -> i32` returns actual gain after cap. Method `reset()`.
2. Register module in `src/combat/mod.rs`: add `pub mod energy;`.
3. In `src/combat/bootstrap.rs`: add `Energy::default()` and `RoundEnergyTracker::default()` to the spawn bundle in `spawn_unit_from_def`.
4. Add `#[cfg(test)] mod tests` in `energy.rs`: (a) secondary cap at 10, (b) external cap at 30, (c) caps are independent, (d) reset restores full budget, (e) Energy::gain clamps at max.
5. Run `cargo test` — all tests pass.

## Must-Haves

- [ ] Energy component with max 100
- [ ] RoundEnergyTracker enforces 10/30 caps
- [ ] Module registered in mod.rs
- [ ] Units spawned with Energy and tracker

## Verification

- `cargo test` passes
- `grep -q 'pub mod energy' src/combat/mod.rs`
  - Files: `src/combat/energy.rs`, `src/combat/mod.rs`, `src/combat/bootstrap.rs`
  - Verify: cargo test 2>&1 | tail -5

- [x] **T03: Add evo_stage to Unit component, BasicStreak tracker, and Child skill discount in resolution** `est:45m`
  R081 requires Child units to get -1 SP cost on their next Skill after 2 consecutive Basic attacks. This needs: (1) evo_stage on Unit component so resolution can check it, (2) a BasicStreak per-unit component, (3) discount logic in apply_effects.

## Steps

1. In `src/combat/unit.rs`: add `evo_stage: EvoStage` field to `Unit` struct. Import `EvoStage` from `super::types`.
2. In `src/combat/bootstrap.rs` `spawn_unit_from_def`: set `evo_stage: def.evo_stage` in the Unit construction.
3. Create `BasicStreak` component in `src/combat/unit.rs`: `pub struct BasicStreak { pub count: u32 }` with Default. Methods: `increment(&mut self)`, `reset(&mut self)`, `qualifies_for_discount(&self) -> bool` (count >= 2).
4. In `src/combat/bootstrap.rs`: add `BasicStreak::default()` to spawn bundle.
5. In `src/combat/resolution.rs` `apply_effects`: add `basic_streak: &mut BasicStreak` parameter. After successful action execution:
   - If action is Basic (ult_effect == GainFromBasic): call `basic_streak.increment()`.
   - If action is Skill (ult_effect == None, sp_cost > 0): before SP spend, if attacker `evo_stage == EvoStage::Child` and `basic_streak.qualifies_for_discount()`, reduce effective sp_cost by 1 (min 0), then `basic_streak.reset()`.
6. Update all call sites of `apply_effects` to pass a `&mut BasicStreak`. In resolution_tests.rs, construct a default BasicStreak for each test. In turn_system pipeline, query BasicStreak from the attacker entity.
7. Add tests in resolution_tests.rs: (a) Child after 2 Basics gets -1 SP on Skill, (b) Adult after 2 Basics gets no discount, (c) discount resets streak, (d) 1 Basic is not enough for discount.
8. Run `cargo test` — all tests pass.

## Must-Haves

- [ ] Unit.evo_stage field populated at spawn
- [ ] BasicStreak component tracks consecutive Basics
- [ ] Child gets -1 SP cost after 2+ consecutive Basics
- [ ] Discount consumed (streak resets) after use
- [ ] Adult units unaffected

## Verification

- `cargo test` passes
- New tests cover Child discount and Adult no-discount

## Negative Tests

- Adult with 5 consecutive Basics → no discount
- Child with 1 Basic → no discount
- Child uses discount → streak resets to 0, needs 2 more Basics for next discount
  - Files: `src/combat/unit.rs`, `src/combat/bootstrap.rs`, `src/combat/resolution.rs`, `src/combat/resolution_tests.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/tests.rs`
  - Verify: cargo test 2>&1 | tail -5

- [x] **T04: Add integration test exercising SP cap and Child discount scenario end-to-end** `est:30m`
  Integration test proving the slice demo: 3 turns of Basic with a Child unit show discount at 3rd skill; SP cap enforced when attempting to exceed +2 non-Basic.

## Steps

1. Create `tests/resource_caps.rs` with two test functions:
   - `child_discount_after_two_basics`: Build a minimal world with a Child unit. Execute 2 Basic actions via ActionIntent. Then execute a Skill — verify SP cost is reduced by 1. Execute another Skill — verify no discount (streak was reset).
   - `sp_non_basic_cap_enforced`: Build a world with SpPool and RoundSpTracker. Attempt 3 non-Basic SP gains of +1 each — verify only 2 are applied. Reset tracker, gain 2 more — verify success.
2. Run `cargo test --test resource_caps` — both tests pass.
3. Run full `cargo test` — all tests pass.

## Must-Haves

- [ ] Integration test for Child discount scenario
- [ ] Integration test for SP non-Basic cap
- [ ] All existing tests still pass

## Verification

- `cargo test --test resource_caps` passes
- `cargo test` passes with zero failures
  - Files: `tests/resource_caps.rs`
  - Verify: cargo test --test resource_caps 2>&1 | tail -5

## Files Likely Touched

- src/combat/sp.rs
- src/combat/resolution.rs
- src/combat/resolution_tests.rs
- src/combat/damage_tests.rs
- src/combat/turn_system/tests.rs
- src/combat/enemy_ai.rs
- tests/patamon_revive.rs
- tests/sp_economy.rs
- tests/validation_snapshot.rs
- tests/combat_coherence.rs
- tests/status_accuracy.rs
- tests/damage_breakdown_log.rs
- tests/status_effect_apply.rs
- src/combat/energy.rs
- src/combat/mod.rs
- src/combat/bootstrap.rs
- src/combat/unit.rs
- src/combat/turn_system/pipeline.rs
- tests/resource_caps.rs
