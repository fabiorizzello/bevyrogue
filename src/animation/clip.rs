use std::collections::BTreeMap;

use bevy::prelude::{Asset, TypePath};
use serde::{Deserialize, Serialize};

/// Typed sprite-sheet clip geometry shared across authored animation assets.
#[derive(Asset, TypePath, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub meta: ClipMeta,
    pub ranges: BTreeMap<String, ClipRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipMeta {
    pub frame_size: FrameSize,
    pub columns: u32,
    pub rows: u32,
    pub total_frames: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipRange {
    pub start: u32,
    pub end: u32,
}

impl ClipRange {
    /// Inclusive authored frame count.
    pub fn len(self) -> u32 {
        self.end - self.start + 1
    }

    pub fn contains(self, frame: u32) -> bool {
        (self.start..=self.end).contains(&frame)
    }
}
