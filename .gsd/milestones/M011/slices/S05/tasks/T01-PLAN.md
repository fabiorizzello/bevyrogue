---
estimated_steps: 17
estimated_files: 13
skills_used: []
---

# T01: Fix SpPool max=5 and add RoundSpTracker with per-turn cap

R073 requires SP max=5 (not 10) and a cap of +2 non-Basic SP gain per round. This task changes SpPool::default() max to 5, adds a RoundSpTracker resource that enforces the +2 cap, updates apply_effects to route SP gain through the tracker, and sweeps all test files that construct SpPool with max:10.

## Steps

1. In `src/combat/sp.rs`: change `SpPool::default()` max from 10 to 5. Add `SpGainSource` enum with `Basic` and `NonBasic` variants. Add `RoundSpTracker` resource with `non_basic_gained: i32` field and `try_gain_non_basic(amount: i32) -> i32` method that clamps to remaining cap (2 - non_basic_gained). Add `reset()` method.
2. In `src/combat/resolution.rs`: update `apply_effects` signature to accept `&mut RoundSpTracker`. In the `UltEffect::GainFromBasic` arm, keep `sp.gain(1)` as-is (Basic SP is uncapped). For future non-Basic SP gains, route through tracker. No non-Basic SP gain paths exist yet in apply_effects — the tracker is wired but the non-Basic path is exercised only by tests.
3. Sweep all files that construct `SpPool { ... max: 10 }` and update to `max: 5`. Files: `src/combat/resolution_tests.rs`, `src/combat/damage_tests.rs`, `src/combat/turn_system/tests.rs`, `src/combat/enemy_ai.rs`, `tests/patamon_revive.rs`, `tests/sp_economy.rs`, `tests/validation_snapshot.rs`, `tests/combat_coherence.rs`, `tests/status_accuracy.rs`, `tests/damage_breakdown_log.rs`, `tests/status_effect_apply.rs`. For tests using `max: 100` as an effectively-unlimited pool, leave them or set to a high value — the point is functional, not cosmetic.
4. Add unit tests in `src/combat/sp.rs` `#[cfg(test)] mod tests`: (a) RoundSpTracker caps non-Basic gain at +2, (b) Basic gain is uncapped by tracker, (c) SpPool default max is 5.
5. Run `cargo test` — all tests pass.

## Must-Haves

- [ ] SpPool::default().max == 5
- [ ] RoundSpTracker enforces +2 non-Basic cap per reset cycle
- [ ] All existing tests compile and pass with new max

## Verification

- `cargo test` passes with zero failures
- `grep -r 'max: 10' src/combat/sp.rs` returns nothing

## Negative Tests

- Attempt to gain +3 non-Basic SP in one round → only +2 applied
- After tracker reset, full +2 budget is available again

## Inputs

- ``src/combat/sp.rs``
- ``src/combat/resolution.rs``
- ``src/combat/resolution_tests.rs``
- ``src/combat/damage_tests.rs``
- ``src/combat/turn_system/tests.rs``
- ``src/combat/enemy_ai.rs``
- ``tests/patamon_revive.rs``
- ``tests/sp_economy.rs``
- ``tests/validation_snapshot.rs``
- ``tests/combat_coherence.rs``
- ``tests/status_accuracy.rs``
- ``tests/damage_breakdown_log.rs``
- ``tests/status_effect_apply.rs``

## Expected Output

- ``src/combat/sp.rs` — SpPool max=5, RoundSpTracker resource, SpGainSource enum`
- ``src/combat/resolution.rs` — apply_effects accepts &mut RoundSpTracker`

## Verification

cargo test 2>&1 | tail -5
