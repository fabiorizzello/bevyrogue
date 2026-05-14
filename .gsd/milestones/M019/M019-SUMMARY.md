---
id: M019
title: "DR pipeline + Heal/Cleanse primitives + PerHop guard"
status: complete
completed_at: 2026-05-14T09:42:57.366Z
key_decisions:
  - DR sum unclamped at DrBag level; (1.0 - sum_dr).max(0.0) factor in calculate_damage provides the effective floor — values >1.0 are legal in the bag
  - DR applied as final multiplicative step after break amplification, not inside calculate_damage — keeps qualitative and quantitative axes testable in isolation with observability via pre_dr/final event fields
  - Heal uses floor division (hp_max * pct) / 100, consistent with revive formula; KO = silent no-op, sp_ok=true, no event
  - cleanse_n ordering: duration_remaining DESC, insertion-index ASC tiebreak — deterministic without extra structures
  - ResolvedAction.cleanse_count: Option<Option<u8>> — outer None = not a cleanse skill, inner None = cleanse all, inner Some(n) = cleanse N
  - D001: DamageCurve::PerHop runtime guard truncates loop and emits OnActionFailed diagnostic — never panics; load-time validator remains primary defence
key_files:
  - src/combat/buffs.rs
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/events.rs
  - src/combat/status_effect.rs
  - src/combat/follow_up.rs
  - src/data/skills_ron.rs
  - src/combat/state.rs
  - tests/dr_pipeline.rs
  - tests/heal_effect.rs
  - tests/cleanse_effect.rs
  - tests/perhop_guard.rs
lessons_learned:
  - follow_up.rs has its own local ResolveActorsQuery independent of resolution.rs — any new component added to the main resolution query must also be added to follow_up.rs or tuple-arity compile errors result (MEM001)
  - SelfOnly was silently missing from shape_is_executable and target_shape_is_executable_now; new ally-side target shapes must update both executability predicates together (MEM008)
  - Adding a field to ResolvedAction requires touching all test construction sites — search tests/ for ResolvedAction { .. } after any struct field addition (MEM009)
  - AllAllies fan-out must use actors.get_mut(entity) per iteration rather than get_many_mut to avoid Bevy panic when caster appears in both attacker and target lists (MEM006)
  - Mixed Heal+Cleanse on a single kernel skill is rejected at load time; single-effect-kind per skill is the kernel contract until M021 trait Skill (MEM010)
---

# M019: DR pipeline + Heal/Cleanse primitives + PerHop guard

**Added BuffKind::DR multiplicative damage reduction, Effect::Heal and Effect::Cleanse DSL primitives with AllAllies fan-out, and a DamageCurve::PerHop runtime length guard — all headless-first, integration-tested, kernel-franchise-agnostic.**

## What Happened

M019 extended the combat kernel with three generic defensive/support primitives and closed the last M018 follow-up debt item.

**S01 — BuffKind::DR (DrBag + calculate_damage integration):**
Added a `DrBag` component that accumulates DR entries with unclamped summation. `calculate_damage` in damage.rs applies a final multiplicative step `(1.0 - sum_dr).max(0.0)` after all qualitative modifiers (break amplification, tag resists, triangle). Values >1.0 in the bag clamp naturally to 0 damage via the floor. `CombatEvent::Damage` now carries both `pre_dr` and `final` amounts for observability. The `follow_up.rs` local `ResolveActorsQuery` was updated in lockstep with `resolution.rs` to avoid tuple-arity compile errors (MEM001). Verified by 6 tests in `tests/dr_pipeline.rs`: single DR, stacked DR, DR+resist, DR during Break, 100% clamp, >100% no-panic.

**S02 — Effect::Heal { amount_pct_max_hp }:**
Added `Effect::Heal` to the RON DSL with Single/SelfOnly/AllAllies targeting. `apply_heal_only` in resolution.rs implements the KO guard → floor-division compute → hp_max cap → OnHealed event pipeline. AllAllies fan-out was wired in pipeline.rs using the Blast resource-hoist-then-per-target-dispatch pattern, with `actors.get_mut(entity)` per iteration to safely handle caster-in-target-list. SelfOnly was found missing from `shape_is_executable` and `target_shape_is_executable_now` and added as a correctness fix. Verified by 5 tests in `tests/heal_effect.rs`.

**S03 — Effect::Cleanse { count: Option<u8> }:**
Added `Effect::Cleanse` to the RON DSL. `StatusBag::cleanse_n` removes debuffs in `duration_remaining DESC, insertion_index ASC` order — deterministic, no extra structures. Immunity is derived solely from `classify_buff_kind` (no hardcoded immune list). `ResolvedAction.cleanse_count` uses `Option<Option<u8>>` to distinguish not-a-cleanse / cleanse-all / cleanse-N. `apply_cleanse_only` (raised to `pub` to match `apply_heal_only` visibility) emits `OnCleansed` even on empty kinds lists for telemetry parity. Mixed Heal+Cleanse on a single skill is rejected at load time by `LegalityReasonCode::MixedEffectKinds` (deferred to M021). The AllAllies branch in pipeline.rs dispatches exclusively to heal XOR cleanse, validator-enforced. Verified by 8 tests in `tests/cleanse_effect.rs`.

**S04 — DamageCurve::PerHop runtime length guard (closes M018 follow-up #3):**
A pre-loop guard in pipeline.rs checks `v.len() < hops_planned`, emits a single `CombatEventKind::OnActionFailed` diagnostic, then clamps the loop bound to `v.len()`. Kernel never panics. Load-time validator in skills_ron.rs remains the primary defence; the runtime guard covers dynamically constructed `ResolvedAction`s from future blueprint emitters. Decision recorded as D001 in DECISIONS.md. Verified by 1 test in `tests/perhop_guard.rs`.

**Final state:** `cargo test` green across 668 tests (0 failures), including all 4 new test files and all prior regression tests. All M019 success criteria satisfied. PROJECT.md updated. LEARNINGS.md written with 6 decisions, 5 lessons, 5 patterns, 3 surprises. MEM005–MEM012 persisted to memory store.

## Success Criteria Results

- **BuffKind::DR as multiplicative step:** ✅ DrBag + calculate_damage integration verified by tests/dr_pipeline.rs (6 tests: single DR, stacked DR×N, DR+ARM combined, DR during Break, 100% clamp to 0, >100% no-panic). Sum unclamped at bag level; clamped to 0 via max(0) floor at calculate_damage.
- **Effect::Heal { amount_pct_max_hp }:** ✅ Applied in resolution.rs, capped at hp_max, KO skip (no-op + sp_ok=true, no event), CombatEvent::Healed emitted. Verified by tests/heal_effect.rs (5 tests) including Single, SelfOnly, AllAllies, KO no-op cases.
- **Effect::Cleanse { count: Option<u8> }:** ✅ Removes N debuffs from StatusBag respecting immune flag from classify_buff_kind (no hardcoded list). count=None removes all non-immune debuffs. Ordering: duration_remaining DESC, insertion-index ASC tiebreak. Verified by tests/cleanse_effect.rs (8 tests).
- **DamageCurve::PerHop guard:** ✅ Pre-loop guard in pipeline.rs emits OnActionFailed diagnostic and clamps loop to available coefficients without panicking. Verified by tests/perhop_guard.rs (1 test).
- **Kernel franchise-agnostic:** ✅ No Digimon names, no skill_id branches, no skill-specific rules introduced in src/combat/. All primitives are generic.

## Definition of Done Results

- **All slices [x]:** ✅ S01, S02, S03, S04 all marked complete in DB (4/4 slices, 11/11 tasks done).
- **Summaries exist:** ✅ S01-SUMMARY.md, S02-SUMMARY.md, S03-SUMMARY.md, S04-SUMMARY.md all present with verification=passed.
- **Integrations work:** ✅ 668 tests pass (0 failures) including all 4 new test files and all prior M017/M018 regression tests.
- **No `.gsd/` validation override:** Validation skipped via `skip_milestone_validation` preference (skip_validation: true recorded in M019-VALIDATION.md).
- **LEARNINGS.md written:** ✅ `.gsd/milestones/M019/M019-LEARNINGS.md` written with full cross-slice synthesis.
- **PROJECT.md updated:** ✅ Reflects M019 completion, new architecture patterns (DrBag, Heal/Cleanse, PerHop guard), updated recommended next milestone.
- **Memory store updated:** ✅ MEM005–MEM012 persisted (MEM001–MEM004 already existed from S01–S04 work).

## Requirement Outcomes

No active requirements were registered for M019. The REQUIREMENTS.md file does not exist and PROJECT.md records "Active requirements: none." No requirement status transitions apply.

## Deviations

["S01/T03: DR was applied as a post-calculate_damage subtraction in an intermediate commit before T02 landed its multiplicative damage.rs changes — superseded by final multiplicative approach; final code correct", "S02: SelfOnly added to shape_is_executable/target_shape_is_executable_now as unplanned correctness fix (plan only mentioned AllAllies)", "S03: 9 existing integration test fixtures required cleanse_count: None added — additive, no behavioural impact", "S03: apply_cleanse_only raised from pub(crate) to pub for test visibility — mirrors apply_heal_only, consistent with project conventions"]

## Follow-ups

["Buff expiry events when DrBag entry ticks to zero — deferred to general buff-expiry event system in a later milestone", "RON Effect::DR variant (DR currently lives at component/formula level only, no DSL surface) — deferred", "Selective cleanse by StatusEffectKind (cleanse only DoT, etc.) — deferred to M021 trait Skill + SkillCtx", "Mixed Heal+Cleanse on a single skill — deferred to M021", "Load-time PerHop coefficient count vs hops_planned validator check — deferred to M021", "M020: support blueprint kit implementation (Patamon holy_aegis, Gabumon fur_cloak) — first consumers of Heal/Cleanse/DR primitives"]
