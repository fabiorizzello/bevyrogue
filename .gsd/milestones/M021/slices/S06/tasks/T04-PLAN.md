---
estimated_steps: 1
estimated_files: 18
skills_used: []
---

# T04: Migrate inline fixtures and close the slice with full verification

Rewrite the remaining inline test fixtures that depend on SkillDef.effects so they construct timeline-era skill definitions or load canonical assets, prioritizing action affordance, boundary contracts, encounter/coherence flows, SP/ultimate behavior, blueprint seams, and presentation-boundary tests. Remove assertions that still describe timeline-vs-legacy coexistence, replace them with timeline-only expectations and structural checks that the old schema is gone, then run the full verification ladder until structural removal checks, full tests, headless check, and windowed check all pass.

## Inputs

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `assets/data/skills.ron`
- `tests/action_affordance_query.rs`
- `tests/action_affordance_consumers.rs`
- `tests/boundary_contract.rs`
- `tests/combat_coherence.rs`
- `tests/encounter_e2e.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/resource_caps.rs`
- `tests/revive_semantics.rs`
- `tests/sp_economy.rs`
- `tests/target_shape_bounce_chain.rs`
- `tests/ultimate_event.rs`

## Expected Output

- `tests/action_affordance_query.rs`
- `tests/action_affordance_consumers.rs`
- `tests/boundary_contract.rs`
- `tests/combat_coherence.rs`
- `tests/encounter_e2e.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/resource_caps.rs`
- `tests/revive_semantics.rs`
- `tests/sp_economy.rs`
- `tests/target_shape_bounce_chain.rs`
- `tests/ultimate_event.rs`

## Verification

bash tools/verify_m021_s06_t04.sh

## Observability Impact

Leaves the repo with one active-skill execution story, so future failures are diagnosed through boot-time timeline compilation errors and runtime combat-event traces rather than split legacy and timeline semantics.
