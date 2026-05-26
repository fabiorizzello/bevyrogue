---
id: S04
parent: M006
milestone: M006
provides:
  - A reusable per-Digimon presentation registration seam in `src/windowed/digimon/` for future species modules.
  - Generic registry-driven windowed engine wiring with no Agumon-specific identifiers in `src/windowed/mod.rs` or `src/windowed/render.rs`.
  - A structural CI contract that pins the extraction and protects S05 from accidental recoupling.
requires:
  - slice: S01
    provides: The single enoki spawn path and generic enoki lifecycle consumed by the registry-driven effect extraction.
  - slice: S02
    provides: CueRegistry and pure cue primitive math consumed by Agumon-owned cue registration.
  - slice: S03
    provides: The generic DigimonSprite/cue-dispatch windowed engine that S04 decoupled from Agumon-specific presentation data.
affects:
  - S05
key_files:
  - src/windowed/mod.rs
  - src/windowed/render.rs
  - src/windowed/digimon/mod.rs
  - src/windowed/digimon/agumon/mod.rs
  - tests/windowed_only/agumon_module_extraction.rs
  - tests/windowed_only.rs
key_decisions:
  - UiPlugin and RenderPlugin continue to own generic resource initialization, while per-Digimon modules only populate registry entries after engine setup.
  - Agumon-owned presentation data is registered through generic registries (EnokiVfxRegistry, OnEnterEffectRegistry, SkillStartNodeRegistry, SkillReleaseEffectRegistry, SpritePresentationRegistry, DetonateEffectRegistry) rather than hardcoded engine constants or helper matches.
  - The canonical CI proof for this extraction is an include_str!-based source-contract test because the windowed binary cannot be launched in auto-mode.
patterns_established:
  - Per-Digimon windowed presentation modules expose `register(app)` and are aggregated through `src/windowed/digimon::register_all(app)`.
  - Species-specific VFX, cue, skill-entry, and sprite-presentation data live in registries populated from the species module, while engine systems stay species-agnostic.
  - Binary-only windowed seams are guarded with token-based source-contract tests under `tests/windowed_only/` rather than brittle value-based assertions.
observability_surfaces:
  - Existing `windowed.agumon_playback` trace/info/warn targets remain available after the code move, preserving manual runtime debugging signals during K001 sign-off.
  - Build and test gates now include `agumon_module_extraction`, which acts as an early failure signal if Agumon-specific tokens leak back into engine files.
drill_down_paths:
  - .gsd/milestones/M006/slices/S04/tasks/T01-SUMMARY.md
  - .gsd/milestones/M006/slices/S04/tasks/T02-SUMMARY.md
  - .gsd/milestones/M006/slices/S04/tasks/T03-SUMMARY.md
  - .gsd/milestones/M006/slices/S04/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-26T14:02:46.867Z
blocker_discovered: false
---

# S04: Extract Agumon presentation into its own module

**Extracted all Agumon-specific windowed presentation data and registration into src/windowed/digimon/agumon/ so the engine now consumes only generic registries, with source-contract and build/test gates passing.**

## What Happened

S04 finished the Agumon extraction seam that S05 will extend. T01 introduced src/windowed/digimon/mod.rs with register_all(app), added the agumon submodule, moved Agumon cue registration into src/windowed/digimon/agumon/mod.rs, and kept UiPlugin responsible for shared CueRegistry initialization before per-Digimon registration runs. T02 moved Agumon-owned enoki effect ids, asset paths, projectile arrival wiring, on-enter effects, skill-release effects, and detonate effects into Agumon-owned registry population code; src/windowed/render.rs now drives effect spawning from generic EnokiVfxRegistry, OnEnterEffectRegistry, SkillReleaseEffectRegistry, and DetonateEffectRegistry entries instead of Agumon constants or closed matches. T03 completed the decoupling by moving skill start-node vocabulary plus sprite/atlas presentation data into SkillStartNodeRegistry and SpritePresentationRegistry entries populated by the Agumon module, removing the remaining AGUMON_* constants, skill_start_node helper, and hardcoded agumon atlas path from the engine files. T04 added and wired tests/windowed_only/agumon_module_extraction.rs as the structural contract: it include_str! checks that src/windowed/render.rs and src/windowed/mod.rs contain none of the forbidden Agumon-specific tokens while src/windowed/digimon/agumon/mod.rs owns the required registry, cue, and atlas-path tokens and src/windowed/digimon/mod.rs exposes mod agumon plus fn register_all. Closeout verification then reran the full slice gate and all required checks passed, leaving S04 downstream-ready for the zero-engine-edit Renamon registration work in S05.

## Verification

Fresh closeout verification passed on the current tree using the required slice-level commands: `RUSTFLAGS='-D warnings' cargo build --features windowed` completed successfully with warnings denied; `cargo test --features windowed --test windowed_only agumon_module_extraction -- --nocapture` passed 3/3 source-contract assertions; `cargo test --features windowed --test windowed_only` passed 62/62 tests; and `cargo test --test dependency_gating` passed 2/2 tests, confirming bevy_enoki remains present only in the windowed graph and absent from the headless graph.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Behavioral equivalence in the live `cargo run --features windowed` window remains a manual K001 sign-off item; auto-mode proved the structural contract and build/test gates only.

## Follow-ups

S05 should add Renamon by appending a second per-Digimon registration module against the registries established here, with no engine/core edits. Separately, src/windowed/render.rs remains a large generic presentation file and may benefit from the follow-on structural refactor noted in T03 before broader Digimon expansion.

## Files Created/Modified

- `src/windowed/mod.rs` — Removed inline Agumon cue/startup wiring, declared the digimon module, and routed registration through the per-Digimon seam.
- `src/windowed/render.rs` — Converted windowed presentation lookups to generic registries and removed the remaining Agumon-specific constants/helpers/paths.
- `src/windowed/digimon/mod.rs` — Added the per-Digimon aggregation seam with `register_all(app)` and the agumon submodule.
- `src/windowed/digimon/agumon/mod.rs` — Centralized Agumon cue, enoki VFX, skill-start-node, release, detonate, and sprite-presentation registration plus related source-owned tests.
- `tests/windowed_only/agumon_module_extraction.rs` — Added the include_str!-based source-contract test that enforces the S04 extraction seam.
- `tests/windowed_only.rs` — Registered the new agumon extraction contract in the windowed-only test harness.
