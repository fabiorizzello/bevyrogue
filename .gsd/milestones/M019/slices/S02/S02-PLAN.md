# S02: Effect::Heal { amount_pct_max_hp } primitive

**Goal:** Add Effect::Heal { amount_pct_max_hp } as a kernel primitive supporting Single, SelfOnly, and AllAllies targets. Heal is capped at hp_max, no-ops silently on KO targets (no event, no SP consumed for Single-KO), and emits CombatEventKind::OnHealed { amount, hp_after }. Kernel-only: no skill-specific logic, no franchise names.
**Demo:** Test integration tests/heal_effect.rs: skill RON con Effect::Heal applicata su Single e AllAllies, cap a maxHP, no-op su KO, CombatEvent::Healed nel JSONL stream.

## Must-Haves

- tests/heal_effect.rs covers: Heal Single on damaged ally, Heal Single at full HP (amount=0, event emitted), Heal Single on KO target (no state change, no event), Heal AllAllies with mixed KO+alive (KO skipped, slot-ordered events for alive), Heal cap (over-heal clamped to hp_max). Full cargo test green. JSONL stream contains OnHealed entries when fixture skill is exercised.

## Proof Level

- This slice proves: contract — integration tests verify the primitive contract end-to-end via apply_effects direct-call pattern.

## Integration Closure

Upstream: Effect enum (src/data/skills_ron.rs), TargetShape enum, resolve_targets (src/combat/resolution.rs), apply_effects pipeline, multi-target fan-out (src/combat/turn_system/pipeline.rs). New wiring: AllAllies branch in resolve_targets + pipeline; OnHealed event variant. Remaining for milestone: S03 Cleanse, S04 PerHop guard.

## Verification

- New CombatEventKind::OnHealed { amount: i32, hp_after: i32 } variant flows through the existing CombatEvent bus and JSONL logger (serde::Serialize derive on CombatEventKind serializes it automatically).

## Tasks

- [x] **T01: Add Effect::Heal variant, TargetShape::AllAllies, OnHealed event, and validator** `est:1 unit — pure additions to enums, one validator branch, exhaustiveness fallout.`
  Introduce the data-model surface area for Heal. Add `Effect::Heal { amount_pct_max_hp: u32, target: TargetShape }` to the skill DSL, extend `TargetShape` with an `AllAllies` arm, add `CombatEventKind::OnHealed { amount: i32, hp_after: i32 }`, and extend `validate_skill_def` so Heal rejects enemy-side target shapes (Bounce/AllEnemies/Blast). Also add `AllAllies` to the executable-now whitelist used by legality checks. No behaviour wiring in this task — only the type/enum surface and exhaustiveness fallout (match arms in resolution.rs / follow_up.rs that today inspect `Effect` must compile without behavioural change).
  - Files: `src/data/skills_ron.rs`, `src/combat/events.rs`, `src/combat/resolution.rs`, `src/combat/follow_up.rs`
  - Verify: cargo check passes (proves enum + match exhaustiveness across all sites). cargo test runs and remains green — no behavioural change yet, only new variants.

- [x] **T02: Implement apply_heal_only helper and wire Heal into apply_effects (Single/SelfOnly)** `est:2 units — helper + resolve_action wiring + apply_effects branch + resolve_targets AllAllies arm.`
  Add `skill_heal_pct(&[Effect])` extractor next to `skill_revive_pct` (around resolution.rs:334). Add `heal_pct: u32` field to `ResolvedAction` (src/combat/state.rs) and populate it in `resolve_action` (resolution.rs:233). Add `apply_heal_only` helper mirroring `apply_damage_only` (resolution.rs:420): KO check first → if KO, return without state change and without emitting an event; otherwise compute `healed = min((hp_max * pct + 99)/100, hp_max - hp_current)` using i64 widening, increment hp_current, emit `OnHealed { amount: healed, hp_after: hp_current }`. Use floor division `(hp_max * pct) / 100` (not ceil) for predictable arithmetic; document the rounding choice in a one-line comment only if non-obvious. Extend `resolve_targets` (resolution.rs:84) with the `AllAllies` arm: filter `team == primary.team && alive`, sort by `slot_index` ascending. Add `AllAllies` to `target_shape_is_executable_now` (resolution.rs:402). In `apply_effects` (resolution.rs:533), insert a heal branch BEFORE the existing damage-path KO guard: if `resolved.heal_pct > 0` and target is KO, return with sp_ok=true (no events, no SP consumed — matches the no-op policy from S02 ROADMAP). For alive Single/SelfOnly, call `apply_heal_only` and propagate the event. This task covers single-target heal only — multi-target fan-out lives in T03.
  - Files: `src/combat/resolution.rs`, `src/combat/state.rs`
  - Verify: cargo check passes. Write a temporary smoke test or run cargo test (existing suite must remain green; no Heal skills in baseline fixtures so trace identity is preserved). Integration test for heal behaviour is added in T03.

- [x] **T03: Wire AllAllies multi-target fan-out in pipeline.rs and add tests/heal_effect.rs** `est:2 units — pipeline fan-out extension + 5 integration test cases.`
  In `src/combat/turn_system/pipeline.rs` (around line 175), extend the multi-target block currently handling `Blast | AllEnemies` to also handle `AllAllies`. Hoist SP/Ult/streak resource consumption once (the existing pattern already does this); per-target dispatch chooses `apply_damage_only` for offensive shapes and `apply_heal_only` for AllAllies. Keep the existing damage paths unchanged. Then add `tests/heal_effect.rs` using the apply_effects direct-call pattern established in tests/dr_pipeline.rs (no Bevy world): (1) Single heal on damaged ally — amount = floor(hp_max * pct / 100), capped to hp_max - hp_current, OnHealed { amount, hp_after } emitted; (2) Single heal at full HP — amount = 0, event still emitted with amount=0; (3) Single heal on KO target — no state change, no event, sp_ok still true (no SP consumed); (4) AllAllies heal with 1 KO + 2 alive damaged — KO untouched and no event; both alive receive heal with OnHealed events ordered by slot_index ascending; (5) Cap test — ally at hp_max-3 with 50% heal → healed exactly 3, hp_after == hp_max. Naming is functional per CLAUDE.md (no s##_ prefix). Tests must be deterministic: no RNG, no wall-clock. If a RON fixture is needed, prefer a test-only fixture under tests/fixtures/ over editing assets/data/skills.ron to keep baseline JSONL trace identity neutral.
  - Files: `src/combat/turn_system/pipeline.rs`, `tests/heal_effect.rs`
  - Verify: cargo test --test heal_effect — all 5 cases pass. cargo test (full suite) — green, no regression in dr_pipeline.rs or other integration tests. cargo check — green.

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/events.rs
- src/combat/resolution.rs
- src/combat/follow_up.rs
- src/combat/state.rs
- src/combat/turn_system/pipeline.rs
- tests/heal_effect.rs
