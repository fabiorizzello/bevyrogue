---
id: T01
parent: S02
milestone: M019
key_files:
  - src/data/skills_ron.rs
  - src/combat/events.rs
  - src/combat/resolution.rs
key_decisions:
  - AllAllies resolves to all alive units on caster's team (same team as primary/caster, not the target team — mirrors AllEnemies which uses primary_entry.team as the target side)
  - Heal validator uses WrongSide reason code (consistent with revive side validator pattern) to reject Bounce/AllEnemies/Blast
  - SelfOnly added to shape_is_executable and target_shape_is_executable_now since it was previously omitted from the whitelist despite being a valid ally-side shape
duration: 
verification_result: passed
completed_at: 2026-05-14T08:38:25.275Z
blocker_discovered: false
---

# T01: Added TargetShape::AllAllies, Effect::Heal { amount_pct_max_hp, target }, CombatEventKind::OnHealed { amount, hp_after }, Heal validator rejecting enemy-side shapes, and AllAllies/SelfOnly to executable whitelists

**Added TargetShape::AllAllies, Effect::Heal { amount_pct_max_hp, target }, CombatEventKind::OnHealed { amount, hp_after }, Heal validator rejecting enemy-side shapes, and AllAllies/SelfOnly to executable whitelists**

## What Happened

Read all four input files to understand the existing type surface before making changes.

Changes applied:
1. `src/data/skills_ron.rs`: Added `AllAllies` arm to `TargetShape` enum (all alive allies, slot_index ascending). Added `Heal { amount_pct_max_hp: u32, target: TargetShape }` to `Effect` enum. Expanded `shape_is_executable` (validator internal) and `target_shape_is_executable_now` (runtime legality check) to include `SelfOnly` and `AllAllies`. Added a Heal-specific validator loop that rejects `Bounce`, `AllEnemies`, and `Blast` target shapes with `LegalityReasonCode::WrongSide`.

2. `src/combat/events.rs`: Added `OnHealed { amount: i32, hp_after: i32 }` variant to `CombatEventKind`. The existing `serde::Serialize` derive handles it automatically — no logger changes required.

3. `src/combat/resolution.rs`: Added `AllAllies` arm to `resolve_targets` match (mirrors `AllEnemies` but filters on the caster's own team). Added `SelfOnly` and `AllAllies` to `target_shape_is_executable_now`. All existing `Effect::` match arms in resolution.rs are `find_map` calls with `_ => None` fallbacks — not exhaustive — so no Heal arm needed in this task.

4. `src/combat/follow_up.rs`: No production match on `Effect` variants exists (only test helpers using `Effect::Damage`); no changes required.

No behavioural wiring in this task — only type/enum surface and exhaustiveness handling.

## Verification

cargo check: clean (exit 0, warnings only, no errors). cargo test: all test suites pass — 0 failures across all integration test binaries including dr_pipeline, validation_snapshot, and others.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 3120ms |
| 2 | `cargo test` | 0 | pass | 45000ms |

## Deviations

SelfOnly was also missing from shape_is_executable and target_shape_is_executable_now — added alongside AllAllies as a minor correctness fix (plan only mentioned AllAllies).

## Known Issues

none

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
