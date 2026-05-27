---
id: T01
parent: S09
milestone: M006
key_files:
  - src/windowed/render/registries.rs
  - src/windowed/render.rs
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Re-export moved types from render.rs via `pub(in crate::windowed) use registries::{...}` so the existing `crate::windowed::render::*` import path keeps working for T01's build while T02 repoints species to the new registries submodule path
  - Declare the submodule `pub(in crate::windowed) mod registries;` so the registries path is reachable for T02's direct imports
  - Raise matches_unit to pub(in crate::windowed) since presentation_entry_for_unit stays in render.rs and now calls it across the module boundary
  - Update the source-string contract test to split type-definition assertions (registries.rs) from lookup-seam assertions (render.rs), since the slice deliberately relocates the structs
duration: 
verification_result: passed
completed_at: 2026-05-27T11:31:14.239Z
blocker_discovered: false
---

# T01: Moved engine-generic presentation registries/types into render/registries.rs with a pub(in crate::windowed) re-export keeping all call sites compiling

**Moved engine-generic presentation registries/types into render/registries.rs with a pub(in crate::windowed) re-export keeping all call sites compiling**

## What Happened

Extracted the species-agnostic registry structs/resources and shared presentation types out of the 2498-line src/windowed/render.rs into a new src/windowed/render/registries.rs submodule (pure structural move, no logic change). Moved items: EnokiVfxRegistry, SoftParticleMaterial, EnokiEffect, EnokiLifecycle, OnEnterEffectRegistry, SkillReleaseEffectRegistry, DetonateEffectRegistry, SkillStartNodeRegistry, SpritePresentationRegistry, SpritePresentationEntry (+ its matches_unit impl). The new module imports PlacementAnchor from bevyrogue::animation and UnitId from bevyrogue::combat::types; all moved items retain pub(in crate::windowed) visibility. render.rs declares `pub(in crate::windowed) mod registries;` and re-exports the ten items via `pub(in crate::windowed) use registries::{...}`, so external call sites (the per-Digimon agumon/renamon modules importing through crate::windowed::render::*) keep resolving unchanged while the new registries submodule path is also reachable for T02 to repoint to. matches_unit was raised from private to pub(in crate::windowed) because presentation_entry_for_unit (which stays in render.rs) now calls it across the module boundary. The now-unused `Particle2dEffect` import was dropped from render.rs's bevy_enoki use line (it followed the types into registries.rs). The presentation lookup seam (presentation_entry_for_unit, the find/matches_unit call, and the per-entry accessors) and the S06/S08 warn-once cast-cue diagnostics remain in render.rs untouched. Updated the source-contract test tests/windowed_only/renamon_extension_contract.rs: render_keeps_the_multi_presentation_lookup_seam now asserts the SpritePresentationRegistry/entries type definitions against the new registries.rs include while keeping the lookup-seam token assertions against render.rs; engine_files_stay_species_agnostic and the warn-once contract are unaffected.

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed finished clean (no warnings, exit 0). cargo test --features windowed --test windowed_only: 75 passed, 0 failed (the relocated source-contract test passes against the split include). Did not execute the windowed binary (K001).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | pass | 14560ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1034ms |

## Deviations

Updated tests/windowed_only/renamon_extension_contract.rs (not listed in the task's Inputs) because its source-string contract pinned SpritePresentationRegistry to render.rs — the exact thing this slice moves; the test had to follow the relocation. Also removed the now-unused Particle2dEffect import from render.rs to keep the -D warnings build clean.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render/registries.rs`
- `src/windowed/render.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
