---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Integration test tests/status_amp_pipeline.rs (slice DoD)

New integration test file `tests/status_amp_pipeline.rs` covering all S03 DoD scenarios in a single deterministic headless harness. Build minimal apps (no UI, no RNG) and assert: (A) Fire base=100, defender non-Heated, neutral attrs, no weakness → final damage = 100; (B) same with Heated applied → 115; (C) Ice base=100, defender Chilled, neutral attrs → 115; (D) active unit with Heated takes its turn → event stream contains an OnDamageDealt {amount:4, damage_tag: Fire, ..} attributed to that unit. Optional 5th case: Chilled unit AV-gain delta vs control. Use `combat::bootstrap` or direct spawn pattern as in existing `tests/status_*.rs`. Headless first; no `windowed` features. Skills: tdd, verify-before-complete.

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/bootstrap.rs`
- `src/combat/events.rs`
- `tests/status_refresh_max_dur.rs`
- `tests/status_multi_kind_coexist.rs`

## Expected Output

- `tests/status_amp_pipeline.rs`

## Verification

cargo test --test status_amp_pipeline && cargo test
