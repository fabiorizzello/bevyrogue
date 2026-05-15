---
estimated_steps: 11
estimated_files: 7
skills_used: []
---

# T02: Rewrite canon active assets and load-time validation around timelines

Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.

Why: the slice cannot delete `Effect` until the shipped canon asset set actually carries timeline-backed data for the active child-roster skills exercised by `units.ron` and the runtime boot path can validate that catalog directly.

Do:
- Convert the remaining child-roster active canon skills in `assets/data/skills.ron` from `effects:` semantics into `timeline:` semantics, using `petit_thunder` as the first real asset-backed loop migration and aligning its behavior with the documented bounce canon.
- Keep the migration anchored to the active skill ids referenced by the child roster in `assets/data/units.ron`, plus any canon follow-up active actions that must still execute through the same runtime path once the legacy branch disappears.
- Update `src/data/skills_ron.rs` and `src/data/skill_timeline.rs` validation and round-trip assumptions so boot-time validation speaks in terms of canonical timeline-backed skill data instead of effect-backed payload scans.
- Add a canon-focused integration test that loads the real asset book, compiles the active canon timelines, and proves those skill ids execute through the compiled timeline path with truthful combat-event ordering.

Negative tests:
- Canon assets with dangling hook, selector, or predicate ids must still fail at boot with `skill_id` and beat or edge site.
- The loop-backed canon skill must degrade deterministically when the bounce chain runs out of alive targets.

Done when: the canonical active child-roster skill set is timeline-backed in assets, boot validation compiles it eagerly, and the canon asset tests pass without using the legacy effect fallback.

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

Preserves the existing load-time failure surface by keeping timeline compilation errors keyed by `skill_id` and exact beat or edge site even after canonical assets stop carrying `effects:` data.
