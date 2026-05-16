# S07 Research: Modifier pipeline + Migrate 6 passive canon

## Summary

Slice S07 implements the remaining foundational framework pieces required for passive mechanics and state modifications, then migrates the 6 core digimon passives onto this framework.

Currently, the `PassiveRunner` and `SignalBus` are scaffolded but limited (e.g. `PassiveRunner` only handles linear timelines), and the unified modifier pipeline does not exist yet. This slice will introduce `ModifierCondition`, `EventFilter`, and the aggregator for status/buff mechanics, followed by migrating the 6 canonical passives (Agumon, Gabumon, Dorumon, Tentomon, Patamon, Renamon). It also requires implementing `BlockReaction` logic for Tentomon's `battery_loop` override to ensure deterministic testing.

## Implementation Landscape

- **Modifier Pipeline**: Needs `ModifierCondition` (D017) and `EventFilter` (D021). An aggregator will collect modifiers from `StatusDef::intrinsic_modifiers` (e.g., Heated, Chilled), ability modifiers, and buffs.
- **SignalBus & EventFilter**: `SignalBus` handles `BlueprintSignal` but needs `EventFilter::{All, Any, Not, Custom}` to allow composite listener rules.
- **PassiveRunner Upgrades**: The S04 implementation of `PassiveRunner` states `// Note: Passive timelines are assumed to be linear for S04 (no Loop support)`. We may need to add `LoopFrame` tracking if any of the 6 passive FSMs use Loops.
- **Block Reaction**: The "Block Reaction verde deterministico" requires introducing a `BlockReaction` effect/intent with `damage_mult` and a `BlockReactionTriggered` event to the `intent_applier` pipeline. This is critical for Tentomon's RNG-gated edge in the `battery_loop` FSM (docs/future_design_draft/digimon/tentomon/04_passive_battery_loop.md).
- **The 6 Passive Canon**:
  1. Agumon: `twin_core_fire` (paired logic with Gabumon)
  2. Gabumon: `fur_cloak` (plus ice twin core)
  3. Dorumon: `predator_loop`
  4. Tentomon: `battery_loop`
  5. Patamon: `holy_support`
  6. Renamon: `kitsune_grace`

## Don't Hand-Roll

- `EventFilter` should cleanly compose rather than rely on hardcoded match conditions in `PassiveRunner`.
- Modifiers shouldn't individually mutate unit state; they should flow through the `modifier_aggregator`.
- Randomness for `BlockReaction` must strictly use `SkillCtx::rng_u32` (SplitMix64) seeded correctly to ensure determinism across `HeadlessAuto` and `Windowed` clocks.

## Forward Intelligence

- The `BlockReaction` (Tentomon) is separated from generic damage reduction buffs. It applies first (`damage * 0.50`), followed by standard DRs. Order of operations in the pipeline must be strictly maintained.
- Passive workflows will introduce `IncomingDamage` as a pre-damage event to allow reactions to trigger before health pools are updated.
- Some VFX / Presentation layers won't be fully realized headless, but the core states (`BlockReady`, `BlockProc`) must tick identically in `HeadlessAuto`.