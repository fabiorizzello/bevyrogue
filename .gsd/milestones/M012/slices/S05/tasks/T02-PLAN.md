---
estimated_steps: 12
estimated_files: 5
skills_used: []
---

# T02: Wire RoundEnergyTracker into the live GrantEnergy pipeline and reset it at turn start.

Expected `skills_used` frontmatter for executor: `test`, `verify-before-complete`.

Why: Existing R073 validated caps in isolation, but `src/combat/turn_system/pipeline.rs` currently bypasses `RoundEnergyTracker` and calls `Energy::gain(requested)` directly. This task proves the real runtime path enforces caps and reports actual mutation.

Do:
1. Add an `Energy` helper in `src/combat/energy.rs` that returns the actual applied amount after max clamping, e.g. `gain_capped(amount) -> i32`; keep existing `gain` behavior for compatibility or delegate it to the new helper.
2. Change the Energy query path in `resolve_action_system`, `resolve_follow_up_action_system`, and `pipeline::step_app` to fetch `Energy` together with optional `RoundEnergyTracker` for the attacker entity. For production entities with a tracker, route `GrantEnergy` through `RoundEnergyTracker::try_gain(EnergyGainSource::SecondaryAction, requested)` and then `Energy::gain_capped(actual_by_round_cap)`.
3. Emit `CombatEventKind::EnergyGained` only for the actual applied amount, or at minimum never emit a positive amount when round cap or Energy max applies zero. Do not overreport the requested amount.
4. Add `RoundEnergyTracker` to `advance_turn_system`'s query tuple and reset the active unit's tracker in the same block that resets `RoundFlags` at the start of the unit turn.
5. Add integration coverage in `tests/resource_caps.rs` using a real `App` with `resolve_action_system`, a simple implemented `GrantEnergy(15)` skill, `Energy`, and `RoundEnergyTracker`. Assert two same-round casts apply at most 10 total secondary Energy, Energy max clipping is truthful, tracker counters reflect applied cap budget, and event amounts do not exceed actual Energy gained.

Failure Modes (Q5): manual legacy fixtures may omit `RoundEnergyTracker`; decide explicitly whether to treat missing tracker as legacy uncapped compatibility or update fixtures under test. Production bootstrap already attaches the tracker, so cap-specific tests must spawn it.
Load Profile (Q6): per action this adds one component query and constant arithmetic; no shared resources beyond Bevy ECS message/event queues.
Negative Tests (Q7): repeated same-round grant past cap, actor near `Energy.max`, and tracker reset at next turn.

Done when: real Bevy action resolution enforces per-unit Energy caps and `cargo test-dev --test resource_caps` passes.

## Inputs

- `src/combat/energy.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/resource_caps.rs`

## Expected Output

- `src/combat/energy.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/resource_caps.rs`

## Verification

cargo test-dev --test resource_caps

## Observability Impact

`CombatEventKind::EnergyGained` becomes truthful cap-aware runtime evidence; tests inspect actual `Energy` and `RoundEnergyTracker` state to localize cap failures.
