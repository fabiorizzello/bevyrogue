---
estimated_steps: 1
estimated_files: 10
skills_used: []
---

# T05: Slice DoD tests all green: status_refresh_max_dur, status_multi_kind_coexist, status_cleanse_policy, status_accuracy (fresh), combat_coherence migrated — 0 FAILED across full suite

## Inputs

- None specified.

## Expected Output

- `tests/status_refresh_max_dur.rs`
- `tests/status_multi_kind_coexist.rs`
- `tests/status_cleanse_policy.rs`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/status_effect_turn_tick.rs`
- `tests/status_accuracy.rs`
- `tests/follow_up_chains.rs`
- `tests/combat_coherence.rs`
- `tests/form_identity.rs`

## Verification

`cargo test --test status_refresh_max_dur`, `cargo test --test status_multi_kind_coexist`, `cargo test --test status_cleanse_policy` all green individually. Full `cargo test` green with 0 ignored.
