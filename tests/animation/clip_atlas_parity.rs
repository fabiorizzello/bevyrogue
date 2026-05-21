use bevyrogue::animation::Clip;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct Atlas {
    meta: AtlasMeta,
    animations: BTreeMap<String, AtlasRange>,
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
struct AtlasRange {
    start_index: u32,
    end_index: u32,
}

fn parse_clip(path: &str) -> Clip {
    let text = std::fs::read_to_string(format!(
        "{}/assets/digimon/{path}/clip.ron",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("clip.ron should be readable");
    ron::from_str(&text).expect("clip.ron should parse")
}

fn parse_atlas(path: &str) -> Atlas {
    let text = std::fs::read_to_string(format!(
        "{}/assets/digimon/{path}_atlas.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("atlas json should be readable");
    serde_json::from_str(&text).expect("atlas json should parse")
}

fn assert_clip_matches_atlas(name: &str, require_all_range: bool) {
    let clip = parse_clip(name);
    let atlas = parse_atlas(name);

    assert_eq!(
        clip.meta.frame_size.w, atlas.meta.frame_size.w,
        "{name}: frame width mismatch"
    );
    assert_eq!(
        clip.meta.frame_size.h, atlas.meta.frame_size.h,
        "{name}: frame height mismatch"
    );
    assert_eq!(
        clip.meta.columns, atlas.meta.columns,
        "{name}: columns mismatch"
    );
    assert_eq!(clip.meta.rows, atlas.meta.rows, "{name}: rows mismatch");
    assert_eq!(
        clip.meta.total_frames, atlas.meta.total_frames,
        "{name}: total_frames mismatch"
    );

    for (range_name, atlas_range) in &atlas.animations {
        let clip_range = clip
            .ranges
            .get(range_name)
            .unwrap_or_else(|| panic!("{name}: missing clip range '{range_name}'"));
        assert_eq!(
            clip_range.start, atlas_range.start_index,
            "{name}:{range_name} start mismatch"
        );
        assert_eq!(
            clip_range.end, atlas_range.end_index,
            "{name}:{range_name} end mismatch"
        );
    }

    if require_all_range {
        let all = clip
            .ranges
            .get("all")
            .expect("agumon clip should expose a whole-sheet 'all' range");
        assert_eq!(all.start, 0, "{name}: 'all' range must start at frame 0");
        assert_eq!(
            all.end,
            clip.meta.total_frames - 1,
            "{name}: 'all' range must span the full atlas"
        );
    }
}

#[test]
fn agumon_clip_matches_atlas_and_exposes_all_range() {
    assert_clip_matches_atlas("agumon", true);
}

#[test]
fn renamon_clip_matches_atlas() {
    assert_clip_matches_atlas("renamon", false);
}
