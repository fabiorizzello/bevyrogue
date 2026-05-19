---
estimated_steps: 8
estimated_files: 1
skills_used: []
---

# T02: Prove real Agumon validation through an external adapter catalog

Why: The validator only proves R005 if real project data can feed its catalogs from outside animation core. This task demonstrates the adapter seam without adding `src/data` or `src/combat` imports to `src/animation`. Expected executor skills: design-an-interface, tdd, bevy, rust-best-practices, verify-before-complete.

Do: Extend `tests/anim_validation.rs` with a test-local adapter that consumes `bevyrogue::data::aggregate_skill_book()` and translates project data into `AnimationValidationCatalogs`. Keep the adapter in the test or another non-animation boundary; do not place it under `src/animation`. For Agumon, deserialize `assets/digimon/agumon/anim_graph.ron` and `assets/digimon/agumon/clip.ron` through the typed schemas, build catalogs from real skill data, and assert the validator report has no blocking diagnostics. Ensure the catalog includes the values the real graph needs: `ParticleId("baby_flame")` from the skill id/action catalog, `StatusId("Heated")` from status-bearing effects or the project status vocabulary, and any parameter names if graph fixtures later use non-literal `ParamRef`s. Add at least one test that deliberately removes a required adapter-provided catalog value and asserts the failure reason identifies the missing catalog entry. Add a boundary guard test that `src/animation/validation.rs` does not contain direct `crate::data`, `crate::combat`, or `digimon` coupling.

Done when: Real Agumon graph+clip validate successfully only through an adapter-built generic catalog, and a catalog omission fails with a typed diagnostic naming the missing value.

Q3 Threat Surface: uses committed local test assets and project data only; no external IO.
Q4 Requirement Impact: directly validates R005 and keeps R001's generic animation boundary intact.
Q5 Failure Modes: adapter missing or stale catalog entries must surface as validation diagnostics, not as panics or false success.
Q6 Load Profile: adapter construction is test/boot-scale; use set collection over the aggregated book and avoid per-command repeated whole-book scans inside core validation.
Q7 Negative Tests: missing `baby_flame` particle/action id and missing `Heated` status catalog entries fail loudly.

## Inputs

- `src/animation/validation.rs`
- `src/animation/mod.rs`
- `src/data/mod.rs`
- `src/data/skills_ron/types.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/agumon/clip.ron`
- `assets/data/digimon/agumon/skills.ron`
- `tests/anim_validation.rs`

## Expected Output

- `tests/anim_validation.rs`

## Verification

cargo test --test anim_validation

## Observability Impact

Proves diagnostics can localize adapter/catalog drift between real animation assets and real project data.
