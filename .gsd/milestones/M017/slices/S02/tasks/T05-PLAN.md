---
estimated_steps: 1
estimated_files: 10
skills_used: []
---

# T05: Slice DoD tests + integration test migration

Add three new integration tests under `tests/`, each spinning up a minimal Bevy `App` mirroring the fixture pattern already used in `src/combat/turn_system/tests.rs`: (a) `tests/status_refresh_max_dur.rs` — apply Heated(dur=2), apply Heated(dur=1), assert exactly one Heated instance with `dur=2`; then apply Heated(dur=5), assert `dur=5`. Use the same triangle/threshold setup so accuracy is 100% (otherwise the second apply could be resisted and skew the test). (b) `tests/status_multi_kind_coexist.rs` — apply Heated + Chilled + Blessed to the same target, assert all three present with their durations via `bag.has(kind)` and `bag.get_dur(kind)`. (c) `tests/status_cleanse_policy.rs` — stage a bag with Heated+Chilled+Paralyzed+Slowed+Blessed; call `cleanse_debuffs`; assert returned `Vec` is the four debuff kinds (order-insensitive) and only Blessed remains. Also migrate any remaining `tests/*.rs` files that still reference the old single-component `StatusEffect` shape — research lists `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_accuracy.rs`, `tests/follow_up_chains.rs`, `tests/combat_coherence.rs`, `tests/form_identity.rs` as candidates — update them to the `StatusBag` API. Lifecycle assertions remain; per-status semantic assertions stay deleted (S03-S05).

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/tests.rs`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/status_effect_turn_tick.rs`
- `tests/status_accuracy.rs`
- `tests/follow_up_chains.rs`
- `tests/combat_coherence.rs`
- `tests/form_identity.rs`

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

## Observability Impact

Tests assert per-instance lifecycle semantics; they do not depend on JSONL log shape.
