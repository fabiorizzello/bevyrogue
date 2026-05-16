---
estimated_steps: 12
estimated_files: 7
skills_used: []
---

# T02: Wire modifier aggregation and pre-damage Block Reaction into damage application

Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: the six canon passives need one shared modifier pipeline, and Tentomon's battery_loop specifically requires a pre-damage reaction that halves raw damage before the normal DR/status cascade. Today apply_deal_damage jumps straight from base damage to calculate_damage with no reactive pre-step.

Do:
- Introduce a shared modifier aggregation module that can combine intrinsic/status/buff/passive modifiers in a deterministic order, keeping the shared layer Digimon-free.
- Extend the intent and event surfaces with the pre-damage seam required by S07: IncomingDamage before HP mutation and BlockReactionTriggered after mitigation is committed.
- Refactor apply_deal_damage/calculate_damage so Block Reaction applies before standard DR, then preserve the existing downstream damage and KO event semantics.
- Keep the new events serializable and safe for JSONL/headless consumers, and add a focused test covering raw damage reduction ordering, no-op behavior when no modifier is armed, and deterministic replay for a fixed seed.

Failure modes / negative tests:
- A missing reaction must leave the old damage path unchanged.
- Multiple modifiers must aggregate in the documented order rather than double-apply or reorder DR.
- The pre-damage event must not mutate world state directly; all state changes still flow through intents/resources.

Done when: the dedicated test proves IncomingDamage -> Block Reaction -> normal DR ordering, the triggered event is emitted exactly once, and the unchanged path still matches the old baseline when no passive modifier is active.

## Inputs

- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/damage.rs`
- `src/combat/events.rs`
- `src/combat/buffs.rs`
- `src/combat/status_effect.rs`
- `docs/future_design_draft/02-08_effect_cascade.md`
- `docs/future_design_draft/digimon/tentomon/04_passive_battery_loop.md`

## Expected Output

- `src/combat/modifiers.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/damage.rs`
- `src/combat/events.rs`
- `src/combat/mod.rs`
- `tests/block_reaction_pipeline.rs`

## Verification

cargo test --test block_reaction_pipeline

## Observability Impact

Adds first-class pre-damage and post-mitigation events so future debugging can distinguish whether a failure came from reaction arming, modifier aggregation, or the existing damage/KO path.
