---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Integration test: status_paralyzed_skip

Create `tests/status_paralyzed_skip.rs`. Spawn 1 ally + 1 enemy via the standard test bootstrap; apply `StatusEffectKind::Paralyzed` with duration=100 to the enemy via direct `StatusBag::apply` insertion (or via a skill resolution if simpler). Drive 100 `TurnAdvanced` cycles for the enemy by writing the event and running the schedule. Assert: zero `ActionIntent` written for the enemy across all 100 cycles; the matching skip-signal events (e.g. `OnActionFailed { reason: "paralyzed" }` or whatever T01 settled on) appear at the expected count (100 minus expirations once duration ticks to 0 — pick the count consistent with T01's tick ordering). Use a fixed seed for `CombatRng`. Keep the test deterministic and headless. Naming follows the functional convention from `CLAUDE.md` (no `s##_` prefix).

## Inputs

- `src/combat/turn_system/mod.rs`
- `src/combat/status_effect.rs`
- `src/combat/events.rs`
- `tests/status_refresh_max_dur.rs`
- `tests/status_amp_pipeline.rs`

## Expected Output

- `tests/status_paralyzed_skip.rs`

## Verification

cargo test --test status_paralyzed_skip
