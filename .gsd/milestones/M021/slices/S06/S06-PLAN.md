# S06: Migrate 18 active skill canon + drop enum Effect

**Goal:** Migrate the remaining active canon skills onto CompiledTimeline, prove a real asset-backed loop path, and delete the legacy Effect-derived production dispatch so active runtime behavior comes only from timeline-backed skills.
**Demo:** 18 active via CompiledTimeline; suite verde + Loop tier-N.

## Must-Haves

- `cargo test --test timeline_chain_bolt_port --test compiled_timeline_petit_thunder --test compiled_timeline_builtin_validation` passes, proving the generic loop surface and a real canon loop-backed asset path.
- `cargo test --test compiled_timeline_boot_validation --test compiled_timeline_active_canon --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test roster_catalog` passes, proving canon assets compile at load, runtime dispatch no longer depends on legacy effect fallback, and bounce semantics stay truthful.
- `cargo test` passes end to end.
- `cargo check` and `cargo check --features windowed` both pass.
- `bash -lc '! rg -n "enum Effect|timeline_backed|apply_effects\\(|effects:" src tests assets/data/skills.ron'` succeeds, proving the legacy Effect schema and dual-path dispatch are removed from source, tests, and canonical skill assets.

## Proof Level

- This slice proves: Integration proof. This slice requires real runtime execution through boot-time SkillBook compilation, BeatRunner dispatch, canon roster skill execution, and structural removal checks against the codebase.

## Integration Closure

This slice consumes the existing S05 SkillBook -> CompiledTimeline bridge, `assets/data/skills.ron`, `assets/data/units.ron`, the generic timeline runtime in `src/combat/api/`, and the production turn pipeline in `src/combat/turn_system/`. It closes the active-skill migration loop by removing the production timeline-vs-legacy branch; later slices only need to migrate passives, blueprint-specific modifiers, and roster-specific composition on top of a single timeline-backed active path.

## Verification

- Preserve and extend the current failure surfaces: load-time timeline compilation errors must still report `skill_id` plus beat or edge site, and runtime verification continues to read the combat event stream for hop order, effect ordering, and action lifecycle boundaries after the legacy effect path is removed.

## Tasks

- [x] **T01: Expand the generic timeline verb surface for looped active skills** `est:5h`
  Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.
  - Files: `src/combat/api/timeline.rs`, `src/combat/api/builtins.rs`, `src/combat/api/intent.rs`, `src/combat/api/applier.rs`, `src/combat/api/skill_ctx.rs`, `src/combat/resolution.rs`, `tests/timeline_chain_bolt_port.rs`, `tests/compiled_timeline_builtin_validation.rs`, `tests/compiled_timeline_petit_thunder.rs`
  - Verify: cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder

- [ ] **T02: Rewrite canon active assets and load-time validation around timelines** `est:6h`
  Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.
  - Files: `assets/data/skills.ron`, `src/data/skill_timeline.rs`, `src/data/skills_ron.rs`, `tests/compiled_timeline_boot_validation.rs`, `tests/compiled_timeline_petit_thunder.rs`, `tests/compiled_timeline_active_canon.rs`, `tests/roster_catalog.rs`
  - Verify: cargo test --test compiled_timeline_boot_validation --test compiled_timeline_petit_thunder --test compiled_timeline_active_canon --test roster_catalog

- [ ] **T03: Remove effect-derived production dispatch and action state** `est:6h`
  Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.
  - Files: `src/combat/state.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/tests.rs`, `tests/compiled_timeline_runtime_dispatch.rs`, `tests/target_shape_bounce_chain.rs`, `tests/patamon_revive.rs`
  - Verify: cargo test --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test patamon_revive

- [ ] **T04: Migrate inline fixtures and close the slice with full verification** `est:5h`
  Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.
  - Files: `tests/action_affordance_query.rs`, `tests/action_affordance_consumers.rs`, `tests/boundary_contract.rs`, `tests/combat_coherence.rs`, `tests/encounter_e2e.rs`, `tests/patamon_blueprint_seam.rs`, `tests/presentation_metadata_boundary.rs`, `tests/resource_caps.rs`, `tests/revive_semantics.rs`, `tests/sp_economy.rs`, `tests/target_shape_bounce_chain.rs`, `tests/ultimate_event.rs`
  - Verify: cargo test
cargo check
cargo check --features windowed
bash -lc '! rg -n "enum Effect|timeline_backed|apply_effects\\(|effects:" src tests assets/data/skills.ron'

## Files Likely Touched

- src/combat/api/timeline.rs
- src/combat/api/builtins.rs
- src/combat/api/intent.rs
- src/combat/api/applier.rs
- src/combat/api/skill_ctx.rs
- src/combat/resolution.rs
- tests/timeline_chain_bolt_port.rs
- tests/compiled_timeline_builtin_validation.rs
- tests/compiled_timeline_petit_thunder.rs
- assets/data/skills.ron
- src/data/skill_timeline.rs
- src/data/skills_ron.rs
- tests/compiled_timeline_boot_validation.rs
- tests/compiled_timeline_active_canon.rs
- tests/roster_catalog.rs
- src/combat/state.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- src/combat/turn_system/tests.rs
- tests/compiled_timeline_runtime_dispatch.rs
- tests/target_shape_bounce_chain.rs
- tests/patamon_revive.rs
- tests/action_affordance_query.rs
- tests/action_affordance_consumers.rs
- tests/boundary_contract.rs
- tests/combat_coherence.rs
- tests/encounter_e2e.rs
- tests/patamon_blueprint_seam.rs
- tests/presentation_metadata_boundary.rs
- tests/resource_caps.rs
- tests/revive_semantics.rs
- tests/sp_economy.rs
- tests/ultimate_event.rs
