---
id: T02
parent: S05
milestone: M012
key_files:
  - src/combat/energy.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - tests/resource_caps.rs
key_decisions:
  - Use a tracker-backed runtime energy-cap path with a legacy compatibility fallback when RoundEnergyTracker is absent.
  - Report EnergyGained using the actual applied Energy delta from gain_capped() and suppress positive over-reporting when the round cap or Energy.max blocks the grant.
duration: 
verification_result: passed
completed_at: 2026-05-01T07:58:43.617Z
blocker_discovered: false
---

# T02: Wire RoundEnergyTracker into the live GrantEnergy pipeline and reset it at turn start.

**Wire RoundEnergyTracker into the live GrantEnergy pipeline and reset it at turn start.**

## What Happened

Added Energy::gain_capped() so runtime callers can clamp and report the actual applied Energy delta instead of assuming the requested amount was fully granted. Updated the real action pipeline in src/combat/turn_system/pipeline.rs and the follow-up resolver to route GrantEnergy through an optional RoundEnergyTracker when present, emit CombatEventKind::EnergyGained only for the applied amount, and keep a compatibility fallback for legacy entities that do not yet carry the tracker. Extended advance_turn_system so the active unit’s RoundEnergyTracker resets in the same turn-start block as RoundFlags, and added an ECS-level regression suite in tests/resource_caps.rs covering same-round secondary Energy caps, truthful Energy.max clipping, and tracker reset behavior on the next turn.

## Verification

Fresh verification after the last code change: `cargo test-dev --test resource_caps` passed 5/5 tests. The suite exercised the real Bevy systems, including `resolve_action_system` with a GrantEnergy(15) skill, same-round cap enforcement, Energy.max clipping, and `advance_turn_system` resetting RoundEnergyTracker and RoundFlags at turn start. Rust diagnostics on the edited source files were clean before the final test rerun.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test resource_caps` | 0 | ✅ pass | 19600ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/energy.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/resource_caps.rs`
