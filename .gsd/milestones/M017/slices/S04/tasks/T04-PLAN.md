---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Integration test: status_slowed_delay

Create `tests/status_slowed_delay.rs`. Spawn a defender unit with a known starting `ActionValue` (e.g. 5000). Apply Slowed via the skill-resolution path (so `pipeline.rs` runs and the first-apply branch executes) — use a deterministic seed so the status-accuracy roll passes. Assert: exactly one `CombatEventKind::TurnAdvance { target, amount_pct: -30 }` is emitted, sourced after `OnStatusApplied`. Run `apply_turn_advance_system` and assert defender AV decreased by 3000 (or matches the expected value once `resistance::apply_av_change` is applied for a unit with no TempoResistance, which equals 3000). Then apply Slowed a second time on the same target and assert NO additional `TurnAdvance` event is emitted (refresh_max_dur path only, gauge already pushed). Keep deterministic and headless.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/av.rs`
- `src/combat/events.rs`
- `src/combat/status_effect.rs`
- `tests/status_amp_pipeline.rs`

## Expected Output

- `tests/status_slowed_delay.rs`

## Verification

cargo test --test status_slowed_delay
