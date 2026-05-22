//! Bevy-free atlas geometry descriptor.
//!
//! The headless lib excludes `bevy/2d`, so it cannot reference Bevy's
//! `TextureAtlasLayout`. This module instead exposes a pure descriptor copied
//! from [`ClipMeta`] plus a player-frame -> atlas-tile-index map.
//!
//! The map is the **identity** function: a clip's frame index equals the atlas
//! tile index. This holds because the authored clip ranges and the atlas grid
//! are generated from the same source sheet (proven by `clip_atlas_parity`), so
//! frame `n` always lives in atlas tile `n`. The real `TextureAtlasLayout` is
//! built windowed-side from these same geometry numbers; this descriptor is the
//! testable seam between the headless animation model and the windowed renderer.

use crate::animation::clip::{ClipMeta, FrameSize};

/// Pure, Bevy-free copy of the atlas grid geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasGeometry {
    pub frame_size: FrameSize,
    pub columns: u32,
    pub rows: u32,
    pub total_frames: u32,
}

impl AtlasGeometry {
    /// Copy the four geometry fields out of a [`ClipMeta`].
    pub fn from_clip_meta(meta: &ClipMeta) -> Self {
        Self {
            frame_size: meta.frame_size,
            columns: meta.columns,
            rows: meta.rows,
            total_frames: meta.total_frames,
        }
    }

    /// Map a player clip frame to its atlas tile index.
    ///
    /// Identity map: returns `Some(frame)` when `frame < total_frames`, else
    /// `None` for out-of-range frames.
    pub fn atlas_index(&self, frame: u32) -> Option<u32> {
        (frame < self.total_frames).then_some(frame)
    }
}
