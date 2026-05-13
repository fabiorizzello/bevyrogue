---
id: T02
parent: S04
milestone: M017
key_files:
  - src/combat/turn_system/pipeline.rs
key_decisions:
  - is_first_apply_slowed computed before bag.apply() to capture pre-mutation state
  - TurnAdvance emitted after OnStatusApplied to preserve JSONL log order: applied → delayed
  - StatusEffectKind imported alongside StatusBag in pipeline.rs
duration: 
verification_result: passed
completed_at: 2026-05-13T09:25:08.944Z
blocker_discovered: false
---

# T02: Slowed first-apply emits TurnAdvance { amount_pct: -30 } via event bus; re-apply and resist branch do not re-emit.

**Slowed first-apply emits TurnAdvance { amount_pct: -30 } via event bus; re-apply and resist branch do not re-emit.**

## What Happened

In `src/combat/turn_system/pipeline.rs` inside the `if passes` branch (status application), added a `is_first_apply_slowed` guard computed before `bag.apply()` mutates the StatusBag. The guard checks `matches!(kind, StatusEffectKind::Slowed) && defender_bag.as_deref().map_or(true, |b| !b.has(&StatusEffectKind::Slowed))`. After the existing `OnStatusApplied` emission, if `is_first_apply_slowed`, a `CombatEventKind::TurnAdvance { target: target_id, amount_pct: -30 }` is emitted via `emit_combat_event`. This routes through `apply_turn_advance_system` → `resistance::apply_av_change` for TempoResistance handling at no extra cost. Re-apply does not re-emit because by the second call `has(&Slowed)` is true. The resist branch (`!passes`) is untouched. Added `StatusEffectKind` to the import line in pipeline.rs.

## Verification

cargo check (0 errors), cargo test --lib (147 passed, 0 failed)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 8000ms |
| 2 | `cargo test --lib` | 0 | 147 passed 0 failed | 20000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs`
