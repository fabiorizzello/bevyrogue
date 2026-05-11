# S05 — Research: Resource caps (R073) + Child mechanics (R081)

**Date:** 2026-04-27

## Summary

This slice adds two independent subsystems: (1) per-round resource caps on SP and Energy as specified in combat_design.md sez. 4, and (2) Child-specific mechanics (skill discount after 2 consecutive Basics). Both are data-driven, require no new dependencies, and build on existing infrastructure.

**SP cap (R073):** The current `SpPool` (sp.rs) has max=10 but MEM017 and the design doc specify max=5 with +2 effective SP/round from non-Basic sources. The `max` field needs to change to 5, and a new per-round tracker must enforce the +2 non-Basic SP gain cap.

**Energy (R073):** No Energy component exists anywhere in the codebase. It needs to be created as a per-unit Component with max 100, plus per-round gain caps (10 from secondary actions, 30 from external sources). Energy is referenced by Form Identity (S08) but this slice only needs the component + cap enforcement.

**Child mechanics (R081):** The `Unit` component has no `evo_stage` field — it lives on `UnitDef` only. The resolution pipeline (`apply_effects`) doesn't have access to the attacker's evo_stage. A new per-unit `BasicStreak` component can track consecutive Basic uses, and the SP discount logic (-1 SP cost after 2 Basics) applies at resolution time when the attacker is a Child.

## Recommendation

Build in this order: (1) SP max fix (5) + round cap tracker, (2) Energy component + cap tracker, (3) Child BasicStreak + discount. Each is independently testable. All three are pure additions with minimal touch on existing code — resolution.rs is the main integration point.

## Implementation Landscape

### Key Files

- `src/combat/sp.rs` — `SpPool`: change `max` default to 5. Add `RoundSpTracker` resource to cap non-Basic SP gain at +2/round. The `gain()` method should accept a source discriminant (Basic vs NonBasic) so the tracker can enforce the cap.
- `src/combat/resolution.rs` — `apply_effects()`: currently calls `sp.gain(1)` on Basic. Needs to route through round tracker. Also needs to apply Child skill discount (-1 SP cost) when attacker has BasicStreak >= 2 and is Child stage.
- `src/combat/unit.rs` — `Unit` component: add `evo_stage: EvoStage` field so resolution has access at runtime (currently only on `UnitDef`).
- `src/combat/types.rs` — `EvoStage` enum already exists with `Child` variant.
- `src/combat/bootstrap.rs` — `spawn_unit()` or equivalent: must copy `evo_stage` from `UnitDef` to `Unit` component at spawn time.
- **New file:** `src/combat/energy.rs` — `Energy` component (per-unit, max 100) + `RoundEnergyTracker` component (per-unit) with two buckets: secondary_action (cap 10) and external (cap 30). Wire into `mod.rs`.
- `src/combat/state.rs` — `ResolvedAction`: no changes needed; SP discount is applied before constructing the ResolvedAction.
- `src/combat/mod.rs` — register new `energy` module.
- `src/main.rs` — register reset systems for round trackers (reset at round start).
- `src/combat/turn_system.rs` — hook round-start reset for `RoundSpTracker` and `RoundEnergyTracker`.

### Build Order

1. **SP max=5 + RoundSpTracker** — Simplest change, immediately testable. Fix `SpPool::default()` max to 5. Add `RoundSpTracker` resource with `non_basic_gained: i32` field and `try_gain_non_basic(amount) -> i32` that clamps to remaining cap. Update `apply_effects` to use tracker. Update all tests that assume max=10.
2. **Energy component + RoundEnergyTracker** — New module, no existing code touched. Component `Energy { current: i32, max: i32 }` + per-unit `RoundEnergyTracker { secondary: i32, external: i32 }`. Gain methods enforce caps. Tests are self-contained.
3. **Child BasicStreak + discount** — Add `evo_stage` to `Unit` component. Add `BasicStreak` component (per-unit, `count: u32`). Increment on Basic, reset on non-Basic. In resolution: if attacker is Child and streak >= 2, apply -1 SP cost (min 0). Reset streak after discount is consumed. Requires updating `apply_effects` signature or passing evo_stage info.

### Verification Approach

- `cargo test` — all existing tests must pass (some will need `max: 5` fixes in SpPool construction).
- New test: SP gain from non-Basic source is capped at +2/round (attempt to gain +3, verify only +2 applied).
- New test: Energy gain capped at 10/round from secondary, 30/round from external.
- New test: Child unit after 2 consecutive Basics gets -1 SP on next Skill. After using the discount, streak resets.
- New test: Adult unit does NOT get the discount regardless of Basic streak.
- `cargo check` — clean compile, no warnings.

## Constraints

- `apply_effects` currently takes `&Unit` for attacker — it doesn't have `evo_stage`. Either add `evo_stage` to `Unit` component or pass it separately. Adding to `Unit` is cleaner since S08 (Form Identity) will also need it.
- `SpPool::default()` max is 10 in current code but should be 5 per MEM017/design doc. This is a breaking change for test fixtures that construct `SpPool { current: X, max: 10 }` — all need updating.
- Round tracker reset must happen at round boundaries. The turn system in `turn_system.rs` manages turn advancement — need to identify where "round start" is signaled. If no explicit round concept exists, the tracker resets per-turn (each unit's action is its turn).

## Common Pitfalls

- **SP max mismatch in tests** — Many tests in `resolution_tests.rs` construct `SpPool { current: N, max: 10 }`. Must sweep all of them to `max: 5`. Missing even one will cause a test failure that looks like a logic bug.
- **Round vs Turn confusion** — The design doc says "+2 SP per round" but the engine may not have a formal "round" concept (it uses turn order). Clarify: "per round" likely means "between one unit's consecutive turns" — i.e., the cap resets each time a unit acts. For SP (team-shared), it resets each time any ally acts.
- **Child discount consuming the streak** — The design says "after 2 Basic Attack... the next Skill costs -1 SP". This implies the discount is consumed once, then streak resets. Not an ongoing discount for all subsequent skills.
