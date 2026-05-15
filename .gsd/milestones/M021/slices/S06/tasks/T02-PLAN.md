---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T02: Rewrite canon active assets and load-time validation around timelines

Convert the remaining child-roster active canon skills in assets/data/skills.ron from effects semantics into timeline semantics, keeping migration anchored to the active skill ids referenced by the child roster in assets/data/units.ron plus canon follow-up actions. Update validation and round-trip assumptions in src/data/skills_ron.rs and src/data/skill_timeline.rs so boot-time validation speaks in terms of canonical timeline-backed skill data, and add canon-focused integration tests that load the real asset book and prove runtime execution through compiled timelines with truthful combat-event ordering.

## Inputs

- `assets/data/skills.ron`
- `assets/data/units.ron`
- `src/data/skill_timeline.rs`
- `src/data/skills_ron.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `tests/compiled_timeline_petit_thunder.rs`
- `tests/roster_catalog.rs`

## Expected Output

- `assets/data/skills.ron`
- `src/data/skill_timeline.rs`
- `src/data/skills_ron.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `tests/compiled_timeline_petit_thunder.rs`
- `tests/compiled_timeline_active_canon.rs`
- `tests/roster_catalog.rs`

## Verification

cargo test --test compiled_timeline_boot_validation --test compiled_timeline_petit_thunder --test compiled_timeline_active_canon --test roster_catalog

## Observability Impact

Preserves the existing load-time failure surface by keeping timeline compilation errors keyed by skill_id and exact beat or edge site even after canonical assets stop carrying effects data.
