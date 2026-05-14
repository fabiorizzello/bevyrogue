---
id: S03
parent: M019
milestone: M019
provides:
  - cleanse-primitive
  - oncleansed-event
  - statsbag-cleanse-n
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - src/combat/events.rs
  - src/combat/status_effect.rs
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
  - tests/cleanse_effect.rs
key_decisions:
  - Effect::Cleanse { count, target } — count=None removes all, count=Some(0) is no-op, same struct for both
  - cleanse_n ordering: duration_remaining DESC, insertion-index ASC tiebreak — deterministic without extra data structures
  - ResolvedAction.cleanse_count: Option<Option<u8>> — outer None distinguishes 'not a cleanse skill' from 'cleanse all' (inner None)
  - apply_cleanse_only emits OnCleansed even when kinds is empty — telemetry parity with OnHealed amount=0
  - KO target on cleanse: silent no-op, no event — mirrors apply_heal_only policy
  - Mixed Heal+Cleanse rejected by validator via LegalityReasonCode::MixedEffectKinds — deferred to M021 trait Skill
  - Immunity derived solely from classify_buff_kind — no hardcoded immune list in kernel
  - apply_cleanse_only raised to pub for integration test visibility — mirrors apply_heal_only
patterns_established:
  - apply_effects-pattern integration tests for new pipeline primitives (no Bevy world spin-up)
  - Either-or dispatch in AllAllies branch: heal XOR cleanse, validator-enforced at DSL level
  - OnCleansed { kinds: [] } for no-op cleanses (alive target) — empty event for telemetry completeness
  - Buff immunity via classify_buff_kind — the single authoritative classification point for cleanse eligibility
observability_surfaces:
  - CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> } emitted per target on every cleanse call, including no-ops (empty kinds vec) for alive targets — flows through existing CombatEvent bus and JSONL logger automatically
drill_down_paths:
  - .gsd/milestones/M019/slices/S03/tasks/T01-SUMMARY.md
  - .gsd/milestones/M019/slices/S03/tasks/T02-SUMMARY.md
  - .gsd/milestones/M019/slices/S03/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-14T09:26:27.473Z
blocker_discovered: false
---

# S03: Effect::Cleanse { count: Option<u8> } primitive

**Added Effect::Cleanse kernel primitive: StatusBag::cleanse_n with deterministic ordering, OnCleansed event, pipeline wiring for Single/SelfOnly/AllAllies, and 8-case integration test suite — all green.**

## What Happened

S03 delivered the Cleanse primitive across three tasks.

**T01** added the data surface: `Effect::Cleanse { count: Option<u8>, target: TargetShape }` variant in `skills_ron.rs`, `CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> }` in `events.rs`, and two new validators — ally-side guard (rejects Bounce/AllEnemies/Blast via `LegalityReasonCode::WrongSide`) and a mixed-effect guard (`LegalityReasonCode::MixedEffectKinds`) that rejects skills carrying both Heal and Cleanse. All match arms in `resolution.rs` and `follow_up.rs` used wildcard arms, so no exhaustiveness fallout. `cargo check --tests` clean; `validation_snapshot` 6/6 green.

**T02** implemented the cleanse primitive end-to-end (excluding pipeline dispatch). `StatusBag::cleanse_n(count: Option<u8>)` removes non-immune debuffs in deterministic order: duration_remaining DESC, insertion-index ASC tiebreak. Returns the removed `StatusEffectKind`s. `count=None` removes all non-immune debuffs; `count=Some(0)` is a no-op. Buff-classified entries (today: Blessed) are never removed — immunity derived solely from `classify_buff_kind`, no hardcoded list. `apply_cleanse_only` emits `OnCleansed { kinds }` even when kinds is empty (telemetry parity with `OnHealed amount=0`); KO target is a silent no-op with no event (mirrors `apply_heal_only` policy). `ResolvedAction.cleanse_count: Option<Option<u8>>` introduced (outer None = not a cleanse skill, Some(inner) = cleanse count). Nine existing integration tests needed `cleanse_count: None` added to `ResolvedAction` construction — additive fix. `cargo test --lib` 6/6 inline unit tests green; full suite green.

**T03** wired `apply_cleanse_only` into the pipeline and added the integration test suite. Two pipeline sites patched: the `status_to_apply` site in `pipeline.rs` handles Single/SelfOnly dispatch; the existing `AllAllies` branch (added by S02) extended with an either-or guard (`heal_pct > 0` XOR `cleanse_count.is_some()`). T01 validator enforces this at DSL level so no runtime collision is possible. `apply_cleanse_only` visibility raised from `pub(crate)` to `pub` to allow import from `tests/`. `tests/cleanse_effect.rs` added with 8 deterministic `apply_effects`-pattern tests. Full suite green with zero regressions across `heal_effect.rs`, `dr_pipeline.rs`, `follow_up_triggers.rs`, `status_blessed_offensive.rs`, `validation_snapshot.rs`.

## Verification

1. `cargo test --test cleanse_effect` — all 8 cases passed (exit 0): count=Some(2) ordering, tiebreak by insertion index, count=None keeps Blessed, count=Some(0) no-op with empty event, Blessed-only bag no-op, count exceeds debuff count, KO no-op no event, empty bag empty event.
2. `cargo test` (full suite) — all test binaries green, zero failures. heal_effect.rs, dr_pipeline.rs, follow_up_triggers.rs, status_blessed_offensive.rs, validation_snapshot.rs unaffected.
3. `cargo check --tests` — clean (only pre-existing warnings, no errors).

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

Nine existing integration tests constructing ResolvedAction directly required `cleanse_count: None` added — not mentioned in the plan but a necessary consequence of the new field. Additive fix with no behavioural impact. `apply_cleanse_only` raised from `pub(crate)` to `pub` to allow import from `tests/` — matches `apply_heal_only` visibility, consistent with project conventions.

## Known Limitations

Cleanse target shape limited to ally-side shapes (Single, SelfOnly, AllAllies). Bounce/AllEnemies/Blast rejected at validation time. Selective cleanse by StatusEffectKind (e.g. cleanse only DoT) deferred to M021 trait Skill + SkillCtx. Mixed Heal+Cleanse on the same skill deferred to M021.

## Follow-ups

S04 (DamageCurve::PerHop runtime length guard) can proceed — depends:S03 satisfied. M021 will add selective cleanse by kind and mixed Heal+Cleanse skills via the trait Skill abstraction.

## Files Created/Modified

- `src/data/skills_ron.rs` — Added Effect::Cleanse variant with count and target fields; ally-side and mixed-effect validators
- `src/combat/events.rs` — Added CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> }
- `src/combat/status_effect.rs` — Added StatusBag::cleanse_n(count) with deterministic ordering and inline unit tests
- `src/combat/state.rs` — Added apply_cleanse_only helper; KO policy, OnCleansed emit
- `src/combat/resolution.rs` — Added ResolvedAction.cleanse_count field; skill_cleanse_count extractor; apply_cleanse_only dispatch for Single/SelfOnly
- `src/combat/turn_system/pipeline.rs` — Extended AllAllies branch with cleanse fan-out (either-or with heal dispatch)
- `src/combat/follow_up.rs` — Added cleanse_count: None to ResolvedAction constructions (exhaustiveness)
- `tests/cleanse_effect.rs` — 8-case integration test suite for cleanse_effect primitive
