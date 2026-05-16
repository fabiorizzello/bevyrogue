# S06: Migrate 18 active skill canon + drop enum Effect

**Goal:** Migrate the remaining active canon skills onto CompiledTimeline, prove a real asset-backed loop path, and delete the legacy Effect-derived production dispatch so active runtime behavior comes only from timeline-backed skills.
**Demo:** 18 active via CompiledTimeline; suite verde + Loop tier-N.

## Must-Haves

- cargo test --test timeline_chain_bolt_port --test compiled_timeline_petit_thunder --test compiled_timeline_builtin_validation passes.
- cargo test --test compiled_timeline_boot_validation --test compiled_timeline_active_canon --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test roster_catalog passes.
- cargo test passes end to end.
- cargo check and cargo check --features windowed both pass.
- Structural grep confirms enum Effect, timeline_backed, apply_effects(, and effects: are removed from src/tests/assets/data/skills.ron.

## Proof Level

- This slice proves: Integration proof. This slice requires real runtime execution through boot-time SkillBook compilation, BeatRunner dispatch, canon roster skill execution, and structural removal checks against the codebase.

## Integration Closure

This slice consumes the existing S05 SkillBook -> CompiledTimeline bridge, assets/data/skills.ron, assets/data/units.ron, the generic timeline runtime in src/combat/api/, and the production turn pipeline in src/combat/turn_system/. It closes the active-skill migration loop by removing the production timeline-vs-legacy branch; later slices only need to migrate passives, blueprint-specific modifiers, and roster-specific composition on top of a single timeline-backed active path.

## Verification

- Preserve load-time skill_id/site validation errors and runtime combat-event inspection so hop order, effect ordering, and action lifecycle boundaries remain diagnosable after the legacy branch is removed.

## Tasks

- [x] **T01: Expand the generic timeline verb surface for looped active skills** `est:5h`
  Extend the timeline payload and builtin execution surface for the remaining active verbs needed by the child-roster canon set, including looped multi-hop damage sequencing, break, status, delay/advance tempo, revive, grant-free-skill, energy grant, and self-targeted tempo side effects. Reuse generic targeting and bounce helpers in src/combat/resolution.rs, keep runtime headless-safe, and add focused tests for loop iteration order, payload-to-intent translation, and malformed payload or missing registry wiring failure behavior.
  - Files: `src/combat/api/timeline.rs`, `src/combat/api/builtins.rs`, `src/combat/api/intent.rs`, `src/combat/api/applier.rs`, `src/combat/api/skill_ctx.rs`, `src/combat/resolution.rs`, `tests/timeline_chain_bolt_port.rs`, `tests/compiled_timeline_builtin_validation.rs`, `tests/compiled_timeline_petit_thunder.rs`
  - Verify: cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder

- [x] **T02: Rewrite canon active assets and load-time validation around timelines** `est:6h`
  Convert the remaining child-roster active canon skills in assets/data/skills.ron from effects semantics into timeline semantics, keeping migration anchored to the active skill ids referenced by the child roster in assets/data/units.ron plus canon follow-up actions. Update validation and round-trip assumptions in src/data/skills_ron.rs and src/data/skill_timeline.rs so boot-time validation speaks in terms of canonical timeline-backed skill data, and add canon-focused integration tests that load the real asset book and prove runtime execution through compiled timelines with truthful combat-event ordering.
  - Files: `assets/data/skills.ron`, `src/data/skill_timeline.rs`, `src/data/skills_ron.rs`, `tests/compiled_timeline_boot_validation.rs`, `tests/compiled_timeline_petit_thunder.rs`, `tests/compiled_timeline_active_canon.rs`, `tests/roster_catalog.rs`
  - Verify: cargo test --test compiled_timeline_boot_validation --test compiled_timeline_petit_thunder --test compiled_timeline_active_canon --test roster_catalog

- [x] **T03: Remove effect-derived production dispatch and action state** `est:6h`
  Replace ResolvedAction fields and turn-pipeline wiring that exist only to shuttle legacy effect-derived data, keeping only metadata still required for action declaration, target bookkeeping, observability, and timeline dispatch. Delete the production timeline_backed branch and legacy apply_effects execution path from the active action flow, remove the Effect enum from the data model, eliminate helper code in src/combat/resolution.rs that scans effect lists to synthesize runtime behavior, and update runtime tests to prove timeline-only dispatch, bounce truthfulness, and revive/support semantics through the new single-path production flow.
  - Files: `src/combat/state.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/tests.rs`, `tests/compiled_timeline_runtime_dispatch.rs`, `tests/target_shape_bounce_chain.rs`, `tests/patamon_revive.rs`
  - Verify: cargo test --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test patamon_revive

- [x] **T04: Migrate inline fixtures and close the slice with full verification** `est:5h`
  Rewrite the remaining inline test fixtures that depend on SkillDef.effects so they construct timeline-era skill definitions or load canonical assets, prioritizing action affordance, boundary contracts, encounter/coherence flows, SP/ultimate behavior, blueprint seams, and presentation-boundary tests. Remove assertions that still describe timeline-vs-legacy coexistence, replace them with timeline-only expectations and structural checks that the old schema is gone, then run the full verification ladder until structural removal checks, full tests, headless check, and windowed check all pass.
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
