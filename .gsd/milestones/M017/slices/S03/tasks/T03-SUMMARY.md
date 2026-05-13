---
id: T03
parent: S03
milestone: M017
key_files:
  - src/combat/turn_system/mod.rs
key_decisions:
  - Used DamageKind::Normal (not ::Neutral which does not exist in the enum) for the DoT hit kind
  - DoT block placed before stunned_opt check so it runs unconditionally for both stunned and active units
duration: 
verification_result: passed
completed_at: 2026-05-13T09:09:26.737Z
blocker_discovered: false
---

# T03: Heated DoT emits 4 HP Fire OnDamageDealt at turn-end unconditionally, bypassing stun-skip (canon §H.1)

**Heated DoT emits 4 HP Fire OnDamageDealt at turn-end unconditionally, bypassing stun-skip (canon §H.1)**

## What Happened

Inserted Heated DoT block in `src/combat/turn_system/mod.rs` immediately before the stun early-return. The block checks `bag.has(&StatusEffectKind::Heated)` and if the unit is alive: decrements `hp_current` by 4 (clamped to 0), emits `OnDamageDealt { amount:4, kind:Normal, damage_tag:Fire, tag_mod_pct:100, triangle_mod_pct:100 }` via `emit_combat_event`, and follows with `OnKO` if hp_current ≤ 0. Placement is unconditional — above the `stunned_opt` check — so Heated+Stunned units still burn. Added `DamageKind` and `DamageTag` imports to the module. Task plan specified `DamageKind::Neutral` which does not exist; used `DamageKind::Normal` (the correct variant in the enum).

## Verification

`cargo check` clean (0 errors). `cargo test --test combat_coherence --test follow_up_chains` all pass (3+2 tests). Full `cargo test` suite: 0 failures across all test binaries.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1490ms |
| 2 | `cargo test --test combat_coherence --test follow_up_chains` | 0 | pass | 2790ms |
| 3 | `cargo test` | 0 | pass | 5000ms |

## Deviations

Task plan specified DamageKind::Neutral; actual enum variant is DamageKind::Normal. Used Normal.

## Known Issues

None.

## Files Created/Modified

- `src/combat/turn_system/mod.rs`
