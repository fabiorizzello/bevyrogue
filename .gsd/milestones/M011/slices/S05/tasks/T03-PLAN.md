---
estimated_steps: 25
estimated_files: 6
skills_used: []
---

# T03: Add evo_stage to Unit component, BasicStreak tracker, and Child skill discount in resolution

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

## Inputs

- ``src/combat/unit.rs``
- ``src/combat/bootstrap.rs``
- ``src/combat/resolution.rs` — updated in T01 with RoundSpTracker param`
- ``src/combat/resolution_tests.rs` — updated in T01`
- ``src/combat/turn_system/pipeline.rs``
- ``src/combat/types.rs``

## Expected Output

- ``src/combat/unit.rs` — Unit.evo_stage + BasicStreak component`
- ``src/combat/bootstrap.rs` — evo_stage + BasicStreak in spawn`
- ``src/combat/resolution.rs` — Child discount logic in apply_effects`
- ``src/combat/resolution_tests.rs` — Child/Adult discount tests`

## Verification

cargo test 2>&1 | tail -5
