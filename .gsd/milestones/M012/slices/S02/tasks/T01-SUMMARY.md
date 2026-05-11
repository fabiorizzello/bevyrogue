---
id: T01
parent: S02
milestone: M012
key_files:
  - src/combat/toughness.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution_tests.rs
  - tests/toughness_enemy_only.rs
key_decisions:
  - Keep Toughness attached to allies for internal weakness data, but gate visible/applicable break semantics with team-aware helpers.
  - Use an optional toughness path in apply_effects and clone weakness tags before the mutable borrow so ally HP damage can flow without borrow-checker conflicts.
duration: 
verification_result: passed
completed_at: 2026-04-30T19:47:50.398Z
blocker_discovered: false
---

# T01: Made toughness enemy-only at runtime without blocking ally HP damage

**Made toughness enemy-only at runtime without blocking ally HP damage**

## What Happened

I added team-aware toughness helpers and rewired the damage pipeline so ally targets keep taking HP damage and status/revive resolution, but never consume toughness, emit OnBreak, or get Stunned from break logic. `apply_effects` now accepts the defender team plus optional toughness, keeps toughness weaknesses available for damage classification, and only applies toughness damage when the helper says the target is an enemy with a real bar. `step_app` now passes through missing/hidden toughness instead of aborting the action, and I updated the internal resolution tests plus a new integration test file to pin the ally-no-break and enemy-break behaviors end to end. While implementing, I also fixed a small borrow-edge by cloning the weakness list before the optional mutable toughness borrow.


## Verification

Fresh verification after the final code change passed: `cargo test-dev --test toughness_enemy_only` passed, `cargo test-dev --test toughness_enemy_only --test follow_up_triggers --test combat_coherence` passed, and `cargo test-dev` passed (122 lib tests + 123 bin tests + all integration/doc tests green).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test toughness_enemy_only` | 0 | ✅ pass | 17300ms |
| 2 | `cargo test-dev --test toughness_enemy_only --test follow_up_triggers --test combat_coherence` | 0 | ✅ pass | 9000ms |
| 3 | `cargo test-dev` | 0 | ✅ pass | 8900ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/toughness.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution_tests.rs`
- `tests/toughness_enemy_only.rs`
