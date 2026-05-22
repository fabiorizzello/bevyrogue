---
id: T01
parent: S01
milestone: M003
key_files:
  - src/animation/atlas.rs
  - src/animation/mod.rs
  - tests/animation/atlas_binding.rs
  - tests/animation.rs
key_decisions:
  - Identity frame->atlas-index map: atlas_index(frame) returns Some(frame) iff frame < total_frames
  - Kept AtlasGeometry Bevy-free (pure descriptor); real TextureAtlasLayout built windowed-side
  - Split the four atlas_index assertions across two focused test fns
duration: 
verification_result: passed
completed_at: 2026-05-22T10:21:05.728Z
blocker_discovered: false
---

# T01: Added Bevy-free AtlasGeometry seam (from_clip_meta + identity atlas_index map) with headless geometry/identity-index tests

**Added Bevy-free AtlasGeometry seam (from_clip_meta + identity atlas_index map) with headless geometry/identity-index tests**

## What Happened

Created src/animation/atlas.rs defining AtlasGeometry { frame_size, columns, rows, total_frames }. from_clip_meta(&ClipMeta) copies the four geometry fields; atlas_index(frame) is an identity map returning Some(frame) when frame < total_frames else None. Doc comment states the identity rationale and notes the real TextureAtlasLayout is built windowed-side. Imports only crate::animation::clip::{ClipMeta, FrameSize} — no bevy/2d imports. Re-exported via pub use atlas::*; in src/animation/mod.rs. Wrote tests/animation/atlas_binding.rs: geometry from agumon clip.ron asserted as 512x512 / cols 10 / rows 10 / total 93; atlas_index(0)==Some(0), atlas_index(92)==Some(92), atlas_index(93)==None, atlas_index(u32::MAX)==None. Registered in tests/animation.rs.

## Verification

Ran cargo test --test animation (exit 0): 54 passed, 0 failed, including the three new atlas_binding tests. atlas.rs confirmed free of bevy/2d / windowed-only imports.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 4525ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/animation/atlas.rs`
- `src/animation/mod.rs`
- `tests/animation/atlas_binding.rs`
- `tests/animation.rs`
