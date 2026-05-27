---
id: T01
parent: S05
milestone: M006
key_files:
  - src/windowed/render.rs
  - src/windowed/mod.rs
  - src/windowed/demo.rs
  - src/windowed/digimon/agumon/mod.rs
  - tests/windowed_only.rs
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Replaced the single-atlas seam with a `presentation_id`-keyed atlas registry and `unit_ids` selectors owned by per-Digimon modules.
  - Moved windowed demo bootstrap composition into a dedicated registry so Digimon modules contribute combatants without hardcoded presets in engine code.
duration: 
verification_result: passed
completed_at: 2026-05-26T18:49:50.041Z
blocker_discovered: false
---

# T01: Generalized windowed sprite presentation and demo bootstrap to registry-driven multi-presentation seams with source contracts locking out single-entry and hardcoded Agumon engine assumptions.

**Generalized windowed sprite presentation and demo bootstrap to registry-driven multi-presentation seams with source contracts locking out single-entry and hardcoded Agumon engine assumptions.**

## What Happened

Generalized the windowed presentation seam so engine code no longer assumes a single sprite presentation entry or a single Agumon-owned atlas resource. In `src/windowed/render.rs`, renamed the remaining Agumon-only presentation function/resource names at the engine seam, added `presentation_id` to `DigimonSprite`, expanded `SpritePresentationEntry` with registry-owned `presentation_id` and stable `unit_ids` selectors, introduced a `PresentationAtlasRegistry` keyed by presentation id, built atlas bindings lazily for every registered presentation, and changed sprite spawning to resolve the correct stance graph, skill graph, atlas image, and clip geometry per `Unit` without species-specific engine matches. Preserved failure visibility by making atlas and deferred-spawn warnings identify presentation ids, asset paths, and unit ids. In `src/windowed/demo.rs` and `src/windowed/mod.rs`, added a registry-backed demo composition seam that clones source `UnitDef`s from the merged roster and assembles the windowed demo from per-Digimon entries instead of hardcoding `EncounterPreset::AgumonTrainingDummy`. In `src/windowed/digimon/agumon/mod.rs`, updated Agumon registration to populate the new presentation selector fields and the demo registry, and added/updated source-contract coverage in `tests/windowed_only/renamon_extension_contract.rs` plus the harness entry in `tests/windowed_only.rs`. (Task re-completed after a spurious reopen; underlying code was already committed in 696bc0d and re-verified green on the current tree.)

## Verification

Re-verified on the current tree: the windowed_only integration harness (which includes renamon_extension_contract) passed with 67/67 tests; the headless default suite and dependency_gating gate also passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass | 40ms |
| 2 | `cargo test` | 0 | pass | 10ms |
| 3 | `cargo test --test dependency_gating` | 0 | pass | 240ms |

## Deviations

None.

## Known Issues

An unrelated pre-existing warning remains in `tests/timeline/timeline_loop_hop_cue_parity.rs` for an unused `BeatEdge` import; it did not affect this task's verification.

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/demo.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
