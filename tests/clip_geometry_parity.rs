use std::{collections::BTreeMap, fs, path::PathBuf};

use bevyrogue::animation::{Clip, ClipMeta, ClipRange, FrameSize};
use serde::Deserialize;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[derive(Debug, Deserialize)]
struct AtlasJson {
    meta: AtlasMeta,
    animations: BTreeMap<String, AtlasAnimation>,
}

#[derive(Debug, Deserialize)]
struct AtlasMeta {
    frame_size: AtlasFrameSize,
    columns: u32,
    rows: u32,
    total_frames: u32,
}

#[derive(Debug, Deserialize)]
struct AtlasFrameSize {
    w: u32,
    h: u32,
}

#[derive(Debug, Deserialize)]
struct AtlasAnimation {
    start_index: u32,
    end_index: u32,
    count: u32,
}

#[test]
fn agumon_clip_ron_matches_authoritative_atlas_geometry() {
    let clip_ron = fs::read_to_string(manifest_path("assets/digimon/agumon/clip.ron"))
        .expect("agumon clip.ron should exist");
    let clip: Clip = ron::from_str(&clip_ron).expect("agumon clip.ron should parse as Clip");

    let atlas_json = fs::read_to_string(manifest_path("assets/digimon/agumon_atlas.json"))
        .expect("agumon_atlas.json should exist");
    let atlas: AtlasJson =
        serde_json::from_str(&atlas_json).expect("agumon_atlas.json should parse");

    assert_eq!(
        clip.meta,
        ClipMeta {
            frame_size: FrameSize { w: 557, h: 561 },
            columns: 10,
            rows: 10,
            total_frames: 95,
        },
        "authored clip.ron geometry changed unexpectedly"
    );

    assert_eq!(clip.meta.frame_size.w, atlas.meta.frame_size.w);
    assert_eq!(clip.meta.frame_size.h, atlas.meta.frame_size.h);
    assert_eq!(clip.meta.columns, atlas.meta.columns);
    assert_eq!(clip.meta.rows, atlas.meta.rows);
    assert_eq!(clip.meta.total_frames, atlas.meta.total_frames);

    let expected_ranges = BTreeMap::from([
        ("attack".to_string(), ClipRange { start: 0, end: 8 }),
        ("block".to_string(), ClipRange { start: 9, end: 13 }),
        ("death".to_string(), ClipRange { start: 14, end: 22 }),
        (
            "heavy_attack".to_string(),
            ClipRange { start: 23, end: 46 },
        ),
        ("hurt".to_string(), ClipRange { start: 47, end: 53 }),
        ("idle".to_string(), ClipRange { start: 54, end: 59 }),
        ("skill".to_string(), ClipRange { start: 60, end: 77 }),
        ("victory".to_string(), ClipRange { start: 78, end: 94 }),
    ]);

    assert_eq!(
        clip.ranges, expected_ranges,
        "authored clip.ron ranges changed unexpectedly"
    );
    assert_eq!(
        clip.ranges.len(),
        atlas.animations.len(),
        "clip.ron and source atlas must expose the same clip count"
    );

    for (name, authored_range) in &clip.ranges {
        let atlas_range = atlas
            .animations
            .get(name)
            .unwrap_or_else(|| panic!("atlas missing animation range for {name}"));

        assert_eq!(
            authored_range.start, atlas_range.start_index,
            "{name} start frame drifted"
        );
        assert_eq!(
            authored_range.end, atlas_range.end_index,
            "{name} end frame drifted"
        );
        assert_eq!(
            atlas_range.count,
            atlas_range.end_index - atlas_range.start_index + 1,
            "{name} JSON count must remain inclusive and internally consistent"
        );
        assert_eq!(
            authored_range.len(), atlas_range.count,
            "{name} authored inclusive length must match source atlas count"
        );
    }
}
