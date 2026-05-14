---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Slowed delay-on-apply via TurnAdvance event in pipeline

In `src/combat/turn_system/pipeline.rs` around L723–760 (the `if outcome.succeeded { if let Some((kind, duration)) = ... status_to_apply` branch), add the first-apply gauge push for Slowed. Before calling `bag.apply(kind.clone(), duration)`, compute `let is_first_apply_slowed = matches!(kind, StatusEffectKind::Slowed) && defender_bag.as_ref().map_or(true, |b| !b.has(&StatusEffectKind::Slowed));`. After the existing `OnStatusApplied` emission (so JSONL log order reads applied → delayed), if `is_first_apply_slowed` emit `CombatEventKind::TurnAdvance { target: target_id, amount_pct: -30 }` via `emit_combat_event` so `apply_turn_advance_system` consumes it next tick (this routes through `resistance::apply_av_change` for free TempoResistance handling). Do NOT mutate `ActionValue` directly — keep the event-bus path. Re-apply must NOT re-emit: the `has(&Slowed)` check guarantees first-apply-only. Resist branch (`!passes`) must NOT emit either.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/events.rs`
- `src/combat/av.rs`
- `src/combat/status_effect.rs`
- `.gsd/milestones/M017/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/turn_system/pipeline.rs`

## Verification

cargo check && cargo test --lib
