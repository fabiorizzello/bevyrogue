---
estimated_steps: 4
estimated_files: 6
skills_used: []
---

# T02: Compile SkillBook timelines on load and fail fast on bad refs

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: roadmap success requires RON typos to fail before runtime, but Bevy loads `skills.ron` after plugin finish. The compiler and validation bridge must therefore live on the SkillBook load path and populate `TimelineLibrary` for later runtime use.

Do: extend `SkillDef` with an optional timeline schema, add a dedicated compiler module that lowers the RON graph into `CompiledTimeline`, compile every timeline-backed skill when the SkillBook asset loads, mirror the validated results into `TimelineLibrary`, and panic immediately on compiler or dangling-reference errors with skill + beat/edge context. Keep legacy skills without a timeline field legal so S06 can migrate incrementally.

Done when: valid timeline-backed skills load into `TimelineLibrary`, invalid hook/selector/predicate ids in RON panic during startup/load, and asset-backed tests exercise both the happy path and the typo path without depending on `.gsd/` artifacts.

## Inputs

- `src/data/skills_ron.rs`
- `src/data/mod.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/registry.rs`
- `src/combat/plugin.rs`
- `assets/data/skills.ron`

## Expected Output

- `src/data/skills_ron.rs`
- `src/data/skill_timeline.rs`
- `src/data/mod.rs`
- `src/combat/api/timeline.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `assets/data/skills.ron`

## Verification

cargo test --test compiled_timeline_boot_validation

## Observability Impact

Adds a strict inspection surface at startup/load: future failures should name the exact skill id plus validation site instead of degrading into late runtime lookup errors.
