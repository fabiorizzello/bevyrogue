# S02: TargetShape resolver: Blast e AoE(All) con tie-break slot_index

**Goal:** Extend the TargetShape resolver from Single to Blast (primary + slot_index ±1) and AoE(All) (every alive enemy), with slot_index ascending as the canonical deterministic tie-break. Introduce a SlotIndex Bevy Component assigned at spawn, a pure resolve_targets() helper, widen the three validation gates, and fan out apply_effects in pipeline::step_app over the resolved target list while keeping resource consumption (SP, ult charge, basic streak) bound to the primary hit only.
**Demo:** CLI scenario scripted con Blast (target primario + spillover adiacenti slot_index ±1) e AoE(All), ordine di applicazione damage stabile su 10 run. JSONL log mostra target list per ogni hit.

## Must-Haves

- SlotIndex(u8) Component assigned per-team contiguous 0..N at spawn_unit_from_def; stable across encounter incl. revives.
- Pure resolve_targets(shape, primary, snapshot) -> Vec&lt;UnitId&gt; ordered by slot_index ascending, deterministic, table-tested.
- TargetShape::Blast variant added; validation gates (skills_ron::validate_skill_def, action_query::target_status_for_*, resolution::target_shape_is_executable_now) all consistently allow Blast and AllEnemies past the Single-only gate.
- pipeline::step_app fans out apply_effects over resolve_targets; SP / ult charge / basic streak / energy consumed exactly once per cast (primary hit only).
- KO'd adjacent slot on Blast: absorb (skip per-target apply, no extra event) — deterministic, no spillover redirection.
- One Blast fixture skill + one AllEnemies (AoE(All) alias) fixture skill in skills.ron marked Implemented; load cleanly.
- combat_cli --scenario aoe-blast: exit 0, JSONL output stable byte-for-byte across 10 invocations, shows resolved target list and per-target damage step-by-step.
- cargo test full suite: 554 baseline tests green + 3 new test binaries (target_shape_blast_spillover, target_shape_aoe_all_order, slot_index_tiebreak). M017 regression tests (status_slowed_delay, tempo_resistance, turn_advance_split) still green.
- cargo check headless + --features windowed: clean.

## Proof Level

- This slice proves: integration

## Integration Closure

Upstream surfaces consumed: src/data/skills_ron.rs (TargetShape, validate_skill_def), src/combat/resolution.rs (resolve_action, apply_effects, target_shape_is_executable_now), src/combat/action_query.rs (target_status_for_* gates), src/combat/turn_system/pipeline.rs (step_app applicator), src/combat/bootstrap.rs (spawn_unit_from_def, apply_composition), assets/data/skills.ron (fixture skills). New wiring: SlotIndex Component declared and inserted at spawn; resolve_targets helper exported from resolution module; combat_cli --scenario aoe-blast dispatcher branch. End-to-end usable: yes — Blast and AoE(All) skills executable from RON, observable via JSONL.

## Verification

- Runtime signals: one CombatEvent::OnDamageDealt emitted per resolved target (no schema change — existing jsonl_logger surface). Inspection surfaces: combat_cli --scenario aoe-blast prints resolved target list + per-target damage; JSONL log shows target_entity per line, ordered slot_index asc. Failure visibility: KO'd-adjacent absorb path skips silently (deterministic); validation gate rejections still emit UnimplementedTargetShape with shape name for any non-allowlisted shape. No new metrics/state.

## Tasks

- [x] **T01: Introduce SlotIndex Component and wire it into spawn_unit_from_def** `est:45m`
  Declare a new SlotIndex(u8) Bevy Component (per-team scoped, paired with the existing Team component for global uniqueness) and assign it at spawn time. apply_composition iterates per-team allies/enemies; convert those loops to enumerate() and pass SlotIndex(idx as u8) into spawn_unit_from_def. SlotIndex must be stable across the encounter — never mutated, survives Revive. Add an integration test asserting that for a 3-ally / 3-enemy encounter each team's slot range is exactly {0,1,2}.
  - Files: `src/combat/unit.rs`, `src/combat/bootstrap.rs`, `tests/slot_index_tiebreak.rs`
  - Verify: cargo test --test slot_index_tiebreak 2>&1 | grep -E '(test result|FAILED)' && cargo check 2>&1 | tail -5

- [x] **T02: Add Blast variant + pure resolve_targets() helper with table-driven tests** `est:1h`
  Extend TargetShape with a Blast variant in src/data/skills_ron.rs. Treat the roadmap label 'AoE(All)' as a DSL alias for the existing AllEnemies variant — do NOT add a new variant (avoid wide diff across 11 deferred skills). Document the alias in the slice SUMMARY when T05 lands.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`
  - Verify: cargo test resolve_targets 2>&1 | grep -E '(test result|FAILED)' && cargo check 2>&1 | tail -5

- [ ] **T03: Widen the three validation gates to accept Blast and AllEnemies** `est:30m`
  Three sites currently gate non-Single shapes behind UnimplementedTargetShape. Widen each consistently so Blast and AllEnemies pass; Row and SelfOnly remain deferred.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `src/combat/action_query.rs`
  - Verify: cargo test 2>&1 | tail -20 | grep -E '(test result|FAILED)' && cargo check --features windowed 2>&1 | tail -5

- [ ] **T04: Fan out apply_effects in pipeline::step_app over resolve_targets()** `est:1h30m`
  In `src/combat/turn_system/pipeline.rs:160-292` (step_app), today: finds a single `target_entity` (~line 168) and calls `apply_effects` once (~line 278). Change: build the target list via `resolve_targets(&action.target_shape, action.target, &snapshot)`, then loop apply_effects over each defender.
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/resolution.rs`, `tests/target_shape_blast_spillover.rs`, `tests/target_shape_aoe_all_order.rs`
  - Verify: cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order 2>&1 | grep -E '(test result|FAILED)' && cargo test 2>&1 | tail -5

- [ ] **T05: Add Blast + AoE fixture skills and combat_cli --scenario aoe-blast with JSONL determinism gate** `est:1h`
  1. **Fixture skills (`assets/data/skills.ron`):** Add ONE new Blast skill and ONE new AllEnemies skill (or flip an existing Implemented Single skill if doing so is lower-churn). Both marked `implementation: Implemented`. Keep all 11 existing non-Single deferred skills untouched (no scope creep). Ensure Effect::Damage{target} matches targeting.shape so the consistency check at skills_ron.rs:291-330 passes.
  - Files: `assets/data/skills.ron`, `src/bin/combat_cli.rs`
  - Verify: cargo run --bin combat_cli -- --scenario aoe-blast > /tmp/aoe1.jsonl 2>&1 && cargo run --bin combat_cli -- --scenario aoe-blast > /tmp/aoe2.jsonl 2>&1 && diff /tmp/aoe1.jsonl /tmp/aoe2.jsonl && cargo test 2>&1 | tail -5

## Files Likely Touched

- src/combat/unit.rs
- src/combat/bootstrap.rs
- tests/slot_index_tiebreak.rs
- src/data/skills_ron.rs
- src/combat/resolution.rs
- src/combat/action_query.rs
- src/combat/turn_system/pipeline.rs
- tests/target_shape_blast_spillover.rs
- tests/target_shape_aoe_all_order.rs
- assets/data/skills.ron
- src/bin/combat_cli.rs
