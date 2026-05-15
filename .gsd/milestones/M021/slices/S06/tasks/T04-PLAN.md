---
estimated_steps: 11
estimated_files: 12
skills_used: []
---

# T04: Migrate inline fixtures and close the slice with full verification

Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.

Why: removing `Effect` will strand a broad integration-test surface that still builds inline `SkillDef { effects: ... }` fixtures. The slice is only complete once those fixtures are rewritten for the timeline-era schema and the full project passes under headless and windowed builds.

Do:
- Rewrite the remaining inline test fixtures that currently depend on `SkillDef.effects` so they construct timeline-era skill definitions or load canonical assets, whichever keeps the contract under test clearer.
- Prioritize high-signal integration surfaces: action affordance, boundary contracts, encounter and coherence flows, SP and ultimate behavior, blueprint seams, and presentation-boundary tests that currently assume legacy effect-backed construction.
- Remove any test assertions that still describe timeline-vs-legacy coexistence, replacing them with timeline-only expectations and structural checks that the old schema is gone.
- Run the full verification ladder and only stop when structural removal checks, full tests, headless check, and windowed check all pass.

Negative tests:
- Boundary fixtures must still cover malformed timeline references and unsupported target semantics where the old tests previously relied on invalid effect data.
- Roster or encounter tests must keep deterministic behavior when the same skill is executed in repeated runs.

Done when: no inline fixture or canonical asset still relies on `effects:`, the full automated verification ladder passes, and the codebase is structurally clean of the removed active-skill effect path.

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

cargo test
cargo check
cargo check --features windowed
bash -lc '! rg -n "enum Effect|timeline_backed|apply_effects\\(|effects:" src tests assets/data/skills.ron'

## Observability Impact

Leaves the repo with one active-skill execution story, so future failures are diagnosed through boot-time timeline compilation errors and runtime combat-event traces rather than split legacy and timeline semantics.
