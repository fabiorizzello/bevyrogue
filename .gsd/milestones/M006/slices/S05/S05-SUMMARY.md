---
id: S05
parent: M006
milestone: M006
provides:
  - Registry-driven multi-presentation windowed rendering (presentation_id-keyed atlas plus per-unit selectors); registry-backed windowed demo composition; Renamon presentation extension module; durable source-contract CI signal that engine files stay species-agnostic.
requires:
  - slice: S04
    provides: per-Digimon register(app) seam
  - slice: S03
    provides: generic DigimonSprite/cue dispatch
  - slice: S02
    provides: CueRegistry
  - slice: S01
    provides: enoki-only VFX path
affects:
  []
key_files: []
key_decisions:
  - Replaced the single-atlas single-entry seam with a presentation_id-keyed PresentationAtlasRegistry and unit_ids selectors owned by per-Digimon modules (D051 extension-first boundary held).
Moved windowed demo composition into a registry so Digimon modules contribute combatants without hardcoded EncounterPreset presets in engine code.
Kept the Renamon source contract semantic (ownership tokens, forbidden engine tokens, lookup-seam presence) rather than numeric/format-sensitive so it survives refactors.
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T18:53:24.822Z
blocker_discovered: false
---

# S05: Second digimon (Renamon) with zero engine edits

**Registered Renamon end-to-end in the windowed demo behind generalized multi-presentation and demo-composition seams, with source contracts locking engine files species-agnostic and all automated gates green.**

## What Happened

S05 closed the milestone extension-first presentation boundary by first generalizing the species-agnostic seams (T01), then adding Renamon as a pure extension module (T02), hardening the zero-engine-edit source contracts (T03), and running the full verification matrix (T04). T01 generalized the windowed presentation seam: DigimonSprite gained a presentation_id, SpritePresentationEntry gained registry-owned presentation_id and stable unit_ids selectors, and a PresentationAtlasRegistry keyed by presentation id replaced the single Agumon-owned atlas resource. Sprite spawning now resolves stance graph, skill graph, atlas image, and clip geometry per Unit with no species-specific engine match. Demo bootstrap moved from a hardcoded EncounterPreset AgumonTrainingDummy into a registry-backed composition seam where per-Digimon modules contribute combatants. Failure visibility was preserved: atlas and deferred-spawn warnings identify presentation id, asset path, and unit id. T02 added src/windowed/digimon/renamon/mod.rs plus its assets (stance.ron, clip.ron with all-range coverage, anim_graph.ron bridging the diamond_storm skill graph with a ReleaseKernel cue) and registered it through the aggregator, sprite presentation registry, and windowed demo registry. T03 hardened tests/windowed_only/renamon_extension_contract.rs to assert ownership semantically: it forbids Renamon identifiers in engine files, pins the generic SpritePresentationRegistry and presentation_entry_for_unit lookup seam, forbids single-entry lookups, and requires Renamon ownership of stance/presentation/skill/asset data without brittle frame or format pinning. T04 ran the full gate matrix with no code changes needed.</narrative>
<parameter name="keyFiles">src/windowed/render.rs
src/windowed/mod.rs
src/windowed/demo.rs
src/windowed/digimon/agumon/mod.rs
src/windowed/digimon/renamon/mod.rs
src/windowed/digimon/mod.rs
assets/digimon/renamon/stance.ron
assets/digimon/renamon/clip.ron
assets/digimon/renamon/anim_graph.ron
tests/windowed_only/renamon_extension_contract.rs

## Verification

Re-ran all four automated gates fresh on the current tree. Headless cargo test exited 0. cargo test --features windowed --test windowed_only produced 67 passed / 0 failed. cargo test --test dependency_gating produced 2 passed (bevy_enoki present in windowed graph, absent from headless graph). RUSTFLAGS=-D warnings cargo build --features windowed finished clean, exit 0. Source-contract tests confirm engine and core files remain species-agnostic and Renamon lives entirely under src/windowed/digimon/renamon/ plus assets.

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

K001 live-window visual sign-off (idle/skill/hurt/death presentation and cue-driven flash/shake quality in cargo winx) remains manual and was not executed from auto mode. A pre-existing unused-import warning in tests/timeline/timeline_loop_hop_cue_parity.rs (BeatEdge) is unrelated and did not affect any gate.

## Follow-ups

User to run cargo winx and confirm Renamon renders as a combatant with working idle/skill/hurt/death presentation and cue-driven flash/shake (K001 manual sign-off).

## Files Created/Modified

None.
