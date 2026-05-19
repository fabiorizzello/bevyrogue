---
id: T04
parent: S01
milestone: M002
key_files:
  - assets/digimon/agumon/clip.ron
  - assets/digimon/agumon/stance.ron
  - assets/digimon/agumon/anim_graph.ron
  - tests/anim_stance_asset.rs
  - tests/clip_atlas_parity.rs
  - Cargo.toml
key_decisions:
  - Treat `clip.ron` as a lossless projection of the tracked atlas JSON, with any extra `all` range derived from `total_frames`.
  - Keep the whole-sheet `all` stance seam, but align Agumon clip, stance, and authored frame ranges to the atlas as the source of truth.
duration: 
verification_result: passed
completed_at: 2026-05-19T19:32:33.094Z
blocker_discovered: false
---

# T04: Authored Agumon’s stance graph and whole-sheet clip seam, then restored clip↔atlas parity with a dedicated regression test.

**Authored Agumon’s stance graph and whole-sheet clip seam, then restored clip↔atlas parity with a dedicated regression test.**

## What Happened

Added the Agumon whole-sheet `all` range and authored `stance.ron` for idle, hurt, death, and victory. During formal closeout, a fresh direct comparison against `assets/digimon/agumon_atlas.json` exposed that `agumon/clip.ron` had drifted from the atlas metadata and named ranges, so the task was finished by realigning `clip.ron`, `stance.ron`, and the Agumon authored skill frames to the atlas source of truth. Added `tests/clip_atlas_parity.rs` so both Agumon and Renamon clip assets are now checked against their atlas JSON, with an explicit assertion that Agumon still exposes a full-sheet `all` range.

## Verification

Fresh `cargo nextest run --profile agent` passed after the parity remediation and test updates, including `anim_stance_asset` and the new `clip_atlas_parity` regression. The targeted asset suite also passed immediately after the asset changes landed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo nextest run --profile agent` | 0 | ✅ pass | 7700ms |
| 2 | `cargo test --test clip_atlas_parity --test anim_stance_asset --test anim_graph_asset --test anim_validation --test anim_gameplay_command_forbidden` | 0 | ✅ pass | 12900ms |

## Deviations

Closeout uncovered that the planned verification target `clip_geometry_parity` no longer existed as a standalone Cargo test. The task was completed by restoring Agumon clip↔atlas parity directly and adding an explicit `clip_atlas_parity` regression test to preserve that proof surface.

## Known Issues

None.

## Files Created/Modified

- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon/stance.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_stance_asset.rs`
- `tests/clip_atlas_parity.rs`
- `Cargo.toml`
