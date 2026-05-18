---
id: T02
parent: S01
milestone: M021
key_files:
  - src/combat/api/skill_ctx.rs
  - src/combat/api/applier.rs
  - src/combat/api/mod.rs
  - tests/intent_applier_canary.rs
key_decisions:
  - intent_applier uses exclusive system (&mut World) to snapshot source/target unit data without Bevy query aliasing — avoids ParamSet complexity for a multi-entity read+write pattern
  - Used bevy::log (not standalone log crate) matching project logging convention
  - CombatEvent.cast_id field deferred to T03 as per slice plan — canary test does not assert cast_id propagation on the event
duration: 
verification_result: passed
completed_at: 2026-05-14T23:06:29.968Z
blocker_discovered: false
---

# T02: Added SkillCtx&lt;'a&gt; + SkillCtxMode, IntentQueue Resource, and intent_applier exclusive system wired DealDamage end-to-end to the existing damage formula as canary

**Added SkillCtx&lt;'a&gt; + SkillCtxMode, IntentQueue Resource, and intent_applier exclusive system wired DealDamage end-to-end to the existing damage formula as canary**

## What Happened

Created `src/combat/api/skill_ctx.rs` with `SkillCtxMode { Execute, DryRun, Preview }` (Default=Execute) and `SkillCtx<'a>` carrying caster, primary_target, cast_id, mode, and a `&'a mut VecDeque<Intent>` for deferred write via `enqueue()`. Created `src/combat/api/applier.rs` with `IntentQueue(VecDeque<Intent>)` Resource and `intent_applier` exclusive system (takes `&mut World`) that drains the queue each frame and routes: `DealDamage` is wired end-to-end — snapshots source/target unit data, calls `calculate_damage` from the existing damage module with full formula (tag mod, triangle mod), reduces target HP, and emits `CombatEvent::OnDamageDealt`; all other variants emit `log::warn!` via `bevy::log`. Updated `src/combat/api/mod.rs` to declare both new submodules and re-export `IntentQueue`, `SkillCtx`, `SkillCtxMode`. Wrote `tests/intent_applier_canary.rs` with two tests: `deal_damage_intent_reduces_hp_and_emits_event` (same-attribute matchup, verifies exactly 100 HP reduction and correct OnDamageDealt with source/target) and `intent_queue_is_empty_after_applier_runs` (confirms queue drain). Used `bevy::log` (not the standalone `log` crate) to match the project's logging convention. Note: `CombatEvent.cast_id` field is not yet added — that is an additive change scoped to T03 per the slice plan.

## Verification

cargo check (headless): clean. cargo check --features windowed: clean. rg 'fn intent_applier' src/combat/api/applier.rs → 1 match. rg 'use bevy::winit|use bevy::render|use bevy_egui' skill_ctx.rs applier.rs → 0 matches (exit 1). cargo test --test intent_applier_canary → 2 passed. cargo test (full suite) → 0 failures across all 78 test groups (includes the 74+ integration tests pre-existing).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 12000ms |
| 2 | `cargo check --features windowed` | 0 | pass | 8000ms |
| 3 | `rg 'fn intent_applier' src/combat/api/applier.rs` | 0 | pass — 1 match | 50ms |
| 4 | `rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/skill_ctx.rs src/combat/api/applier.rs` | 1 | pass — 0 forbidden imports | 50ms |
| 5 | `cargo test --test intent_applier_canary` | 0 | pass — 2/2 tests green | 8270ms |
| 6 | `cargo test` | 0 | pass — 0 failures across all 78 test groups | 45000ms |

## Deviations

none

## Known Issues

CombatEvent does not yet carry cast_id — T03 adds that field additively. All other Intent variants are no-ops with warn logs until S05+ migration.

## Files Created/Modified

- `src/combat/api/skill_ctx.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/mod.rs`
- `tests/intent_applier_canary.rs`
