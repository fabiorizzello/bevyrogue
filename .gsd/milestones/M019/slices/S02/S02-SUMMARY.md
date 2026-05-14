---
id: S02
parent: M019
milestone: M019
provides:
  - Effect::Heal primitive with Single/SelfOnly/AllAllies targeting
  - CombatEventKind::OnHealed event
  - AllAllies resolve_targets branch
  - AllAllies pipeline fan-out
  - tests/heal_effect.rs integration coverage
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - src/combat/events.rs
  - src/combat/resolution.rs
  - src/combat/follow_up.rs
  - src/combat/state.rs
  - src/combat/turn_system/pipeline.rs
  - tests/heal_effect.rs
key_decisions:
  - Floor division (hp_max * pct) / 100 chosen for Heal — consistent with revive formula, no ceiling arithmetic
  - Heal KO no-op returns sp_ok=true with empty events, no OnActionFailed emitted — matches S02 no-op policy
  - AllAllies branch uses actors.get_mut(def_entity) not get_many_mut — avoids entity-collision when caster is in target list
  - AllAllies branch inserted before the offensive self-skip guard in pipeline.rs so caster is included in the fan-out
  - SelfOnly added to shape_is_executable and target_shape_is_executable_now alongside AllAllies — it had been previously omitted
patterns_established:
  - apply_heal_only mirrors apply_damage_only structure: KO guard → compute → mutate → emit event
  - Heal integration tests follow the apply_effects direct-call pattern from dr_pipeline.rs — no Bevy world spin-up
  - AllAllies fan-out in pipeline.rs reuses the existing Blast/AllEnemies resource-hoist-then-per-target-dispatch pattern
observability_surfaces:
  - CombatEventKind::OnHealed { amount, hp_after } flows through the existing CombatEvent bus and JSONL logger via serde::Serialize derive — no additional wiring needed
drill_down_paths:
  - .gsd/milestones/M019/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M019/slices/S02/tasks/T02-SUMMARY.md
  - .gsd/milestones/M019/slices/S02/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-14T08:51:19.235Z
blocker_discovered: false
---

# S02: Effect::Heal { amount_pct_max_hp } primitive

**Added Effect::Heal as a kernel primitive with Single/SelfOnly/AllAllies targeting, hp_max cap, KO no-op, and OnHealed event emission — verified by 5 integration tests in tests/heal_effect.rs**

## What Happened

S02 extended the combat kernel with a Heal primitive across three tasks.

**T01** introduced the full data-model surface: `Effect::Heal { amount_pct_max_hp, target }` in the skill DSL, `TargetShape::AllAllies`, `CombatEventKind::OnHealed { amount: i32, hp_after: i32 }`, and a validator that rejects enemy-side shapes (Bounce/AllEnemies/Blast) for Heal using the existing WrongSide reason code. `SelfOnly` and `AllAllies` were also added to `target_shape_is_executable_now` — SelfOnly had been previously omitted from the whitelist. No behaviour was wired in T01; only enum surface and exhaustiveness fallout.

**T02** implemented the actual heal logic. `apply_heal_only` was added to `resolution.rs`, mirroring `apply_damage_only`: KO check first — if KO, return with `sp_ok=true` and no events; otherwise compute `healed = min((hp_max * pct) / 100, hp_max - hp_current)` using floor division (consistent with the revive formula), increment `hp_current`, emit `OnHealed`. `resolve_action` was extended with a `heal_pct` field in `ResolvedAction`, populated from a new `skill_heal_pct` extractor. `apply_effects` gained a heal branch before the existing damage/revive guard so Heal skills bypass the KO early-return path.

**T03** wired the AllAllies fan-out in `pipeline.rs` and added `tests/heal_effect.rs` with 5 deterministic cases: (1) Single heal on damaged ally — correct floor-division amount and OnHealed event; (2) Single heal at full HP — amount=0, event still emitted; (3) Single heal on KO target — no state change, no event, sp_ok=true; (4) AllAllies with 1 KO + 2 alive — KO skipped, both alive receive OnHealed events ordered by slot_index ascending; (5) Cap test — ally at hp_max-3 with 50% heal clamps exactly to hp_max. The AllAllies pipeline branch uses `actors.get_mut(def_entity)` rather than `get_many_mut` — no attacker borrow is needed for heal, avoiding entity-collision when the caster is in the target list. KO filtering is double-guarded (resolve_targets already excludes KO, apply_heal_only re-checks), which is safe and harmless.

## Verification

cargo test --test heal_effect: 5/5 pass (single_heal_on_damaged_ally, single_heal_at_full_hp_emits_zero_amount, single_heal_on_ko_is_no_op, all_allies_fan_out_ko_skipped_alive_healed_slot_order, heal_cap_at_hp_max). cargo test (full suite): all test binaries green, 0 failures, 0 regressions — dr_pipeline, validation_snapshot, ultimate_meter, and all other integration tests unaffected. cargo check: clean (0 errors, pre-existing warnings only).

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

SelfOnly was missing from shape_is_executable and target_shape_is_executable_now — added alongside AllAllies in T01 as a minor correctness fix (plan only mentioned AllAllies).

## Known Limitations

Heal scaling from ATK stat, selective cleanse per StatusKind, and immunity overrides are deferred to M021 (trait Skill + SkillCtx). AllAllies heal target ordering is slot_index ascending — alternative orderings (e.g. lowest HP first) are not yet supported.

## Follow-ups

S03: Effect::Cleanse { count: Option<u8> } primitive (depends on S02 infrastructure). S04: DamageCurve::PerHop runtime length guard. M021: per-skill Heal scaling (ATK-based), selective cleanse, custom immunity hooks via trait Skill.

## Files Created/Modified

- `src/data/skills_ron.rs` — Added Effect::Heal { amount_pct_max_hp, target } variant and Heal validator (rejects enemy-side shapes)
- `src/combat/events.rs` — Added CombatEventKind::OnHealed { amount: i32, hp_after: i32 } variant
- `src/combat/resolution.rs` — Added TargetShape::AllAllies arm in resolve_targets, skill_heal_pct extractor, apply_heal_only helper, heal branch in apply_effects, AllAllies in target_shape_is_executable_now, SelfOnly added to both whitelists
- `src/combat/follow_up.rs` — Exhaustiveness fallout: new Effect::Heal and TargetShape::AllAllies match arms (no behavioural change)
- `src/combat/state.rs` — Added heal_pct: u32 field to ResolvedAction
- `src/combat/turn_system/pipeline.rs` — Added AllAllies branch in multi-target fan-out using apply_heal_only dispatch
- `tests/heal_effect.rs` — New integration test file: 5 deterministic test cases covering Single, full-HP, KO no-op, AllAllies fan-out, and cap scenarios
