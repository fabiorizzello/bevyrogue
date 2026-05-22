---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T01: Add Bevy-free atlas-geometry seam in the lib with headless geometry + identity-index tests

Why: the on-screen Sprite needs atlas geometry (frame size, grid, frame count) and a player-frame->atlas-tile-index map, but the headless lib excludes bevy/2d so it cannot use Bevy's TextureAtlasLayout; the testable contract must be a pure descriptor (see D-atlas-geometry-seam). skills_used: tdd, design-an-interface, verify-before-complete. Do: create src/animation/atlas.rs defining a Bevy-free `AtlasGeometry { frame_size: FrameSize, columns: u32, rows: u32, total_frames: u32 }` with `pub fn from_clip_meta(meta: &ClipMeta) -> Self` (copy the four ClipMeta fields) and `pub fn atlas_index(&self, frame: u32) -> Option<u32>` returning Some(frame) when frame < total_frames else None (identity map — clip frame index equals atlas tile index, proven by clip_atlas_parity), plus a doc comment stating the identity rationale and that the real TextureAtlasLayout is built windowed-side. Re-export it from src/animation/mod.rs (`pub use atlas::*;`). Write tests/animation/atlas_binding.rs (TDD: write the asserts first) that parses assets/digimon/agumon/clip.ron into a Clip (mirror clip_atlas_parity's parse helper using CARGO_MANIFEST_DIR + ron::from_str), builds AtlasGeometry::from_clip_meta(&clip.meta), and asserts geometry == 512x512 / columns 10 / rows 10 / total_frames 93; that atlas_index(0)==Some(0), atlas_index(92)==Some(92), atlas_index(93)==None, atlas_index(u32::MAX)==None (negative/out-of-range test). Register the new file in tests/animation.rs with a `#[path = "animation/atlas_binding.rs"] mod atlas_binding;` line. Done when: cargo test --test animation passes including the new atlas geometry + identity-index assertions, and atlas.rs has no bevy/2d / windowed-only imports.

## Inputs

- `src/animation/clip.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/clip.ron`
- `tests/animation.rs`
- `tests/animation/clip_atlas_parity.rs`

## Expected Output

- `src/animation/atlas.rs`
- `src/animation/mod.rs`
- `tests/animation/atlas_binding.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation
