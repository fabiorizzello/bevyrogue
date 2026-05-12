---
id: T01
parent: S01
milestone: M017
key_files:
  - src/combat/status_effect.rs
  - src/combat/turn_system/mod.rs
  - src/data/skills_ron.rs
key_decisions:
  - turn_system status match replaced with no-op skeleton; per-status semantics deferred to S03-S05 per D004+D009
  - SpeedModifier insert/remove on Freeze removed since Freeze variant no longer exists; SpeedModifier component kept as query param (still used by turn order)
duration: 
verification_result: passed
completed_at: 2026-05-12T16:04:03.265Z
blocker_discovered: false
---

# T01: StatusEffectKind enum rewritten to Heated/Chilled/Paralyzed/Slowed/Blessed + reserved Burn/Shock; turn_system and skills_ron call sites migrated; cargo check headless+windowed green.

**StatusEffectKind enum rewritten to Heated/Chilled/Paralyzed/Slowed/Blessed + reserved Burn/Shock; turn_system and skills_ron call sites migrated; cargo check headless+windowed green.**

## What Happened

status_effect.rs already had the correct new enum (from a prior partial migration). The compile errors were in two call sites: (1) src/combat/turn_system/mod.rs matched on old Burn{damage_per_turn}/Freeze{speed_reduction}/Shock{cancel_chance_pct}/DeepFreeze variants — replaced with a no-op match over all 7 new variants (semantics deferred to S03-S05 per task contract); removed now-dead SpeedModifier insert/remove on Freeze/Freeze-expiry. (2) src/data/skills_ron.rs tests referenced old struct variants — migrated effect_roundtrip_apply_status_burn/freeze/shock to heated/chilled/paralyzed, and fixed apply_status_negative_duration_rejected_at_parse_time to use Heated. Both headless and windowed cargo check pass with zero errors.

## Verification

cargo check (headless): Finished dev, 0 errors. cargo check --features windowed: Finished dev, 0 errors. No references to Freeze or DeepFreeze in src/ (old Burn/Shock retained as reserved unit variants per §H.1).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 130ms |
| 2 | `cargo check --features windowed` | 0 | pass | 32780ms |

## Deviations

status_effect.rs was already partially migrated; no rewrite needed there. All effort was in migrating call sites (turn_system/mod.rs and skills_ron.rs tests).

## Known Issues

tests/ (status_effect_integration.rs, status_effect_turn_tick.rs, form_identity.rs) still reference old Freeze variant — those will fail cargo test. Covered by T05 per slice contract.

## Files Created/Modified

- `src/combat/status_effect.rs`
- `src/combat/turn_system/mod.rs`
- `src/data/skills_ron.rs`
