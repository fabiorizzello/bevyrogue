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
completed_at: 2026-05-26T17:41:13.269Z
blocker_discovered: false
---

# T01: Generalized windowed sprite presentation and demo bootstrap to registry-driven multi-presentation seams with source contracts locking out single-entry and hardcoded Agumon engine assumptions.

**Generalized windowed sprite presentation and demo bootstrap to registry-driven multi-presentation seams with source contracts locking out single-entry and hardcoded Agumon engine assumptions.**

## What Happened

Generalized the windowed presentation seam so engine code no longer assumes a single sprite presentation entry or a single Agumon-owned atlas resource. In `src/windowed/render.rs`, I renamed the remaining Agumon-only presentation function/resource names at the engine seam, added `presentation_id` to `DigimonSprite`, expanded `SpritePresentationEntry` with registry-owned `presentation_id` and stable `unit_ids` selectors, introduced a `PresentationAtlasRegistry` keyed by presentation id, built atlas bindings lazily for every registered presentation, and changed sprite spawning to resolve the correct stance graph, skill graph, atlas image, and clip geometry per `Unit` without species-specific engine matches. I also preserved failure visibility by making atlas and deferred-spawn warnings identify presentation ids, asset paths, and unit ids. In `src/windowed/demo.rs` and `src/windowed/mod.rs`, I added a small registry-backed demo composition seam that clones source `UnitDef`s from the merged roster and assembles the windowed demo from per-Digimon entries instead of hardcoding `EncounterPreset::AgumonTrainingDummy`. Finally, in `src/windowed/digimon/agumon/mod.rs`, I updated Agumon registration to populate the new presentation selector fields and the demo registry, and added/updated source-contract coverage in `tests/windowed_only/renamon_extension_contract.rs` plus the harness entry in `tests/windowed_only.rs`.

## Verification

Verified the new seam at three levels: the task-required source-contract test passed (`cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture`), the full `windowed_only` integration harness passed after the render/bootstrap refactor (`cargo test --features windowed --test windowed_only -- --nocapture`), and a filtered `cargo test --features windowed register_populates_the_ -- --nocapture` run confirmed the new Agumon-side registry unit tests compile and pass. The filtered cargo run surfaced only an unrelated existing warning about an unused `BeatEdge` import in a timeline test file.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture` | 0 | ✅ pass | 4792ms |
| 2 | `cargo test --features windowed --test windowed_only -- --nocapture` | 0 | ✅ pass | 703ms |
| 3 | `cargo test --features windowed register_populates_the_ -- --nocapture` | 0 | ✅ pass | 3348ms |

## Deviations

None.

## Known Issues

An unrelated pre-existing warning remains in `tests/timeline/timeline_loop_hop_cue_parity.rs` for an unused `BeatEdge` import; it did not affect this task’s verification.

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/demo.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
