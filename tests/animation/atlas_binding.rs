use bevyrogue::animation::{AtlasGeometry, Clip};

fn parse_clip(path: &str) -> Clip {
    let text = std::fs::read_to_string(format!(
        "{}/assets/digimon/{path}/clip.ron",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("clip.ron should be readable");
    ron::from_str(&text).expect("clip.ron should parse")
}

#[test]
fn agumon_atlas_geometry_matches_clip_meta() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);

    assert_eq!(geometry.frame_size.w, 512, "frame width");
    assert_eq!(geometry.frame_size.h, 512, "frame height");
    assert_eq!(geometry.columns, 10, "columns");
    assert_eq!(geometry.rows, 10, "rows");
    assert_eq!(geometry.total_frames, 93, "total_frames");
}

#[test]
fn atlas_index_is_identity_within_range() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);

    assert_eq!(geometry.atlas_index(0), Some(0), "first frame");
    assert_eq!(geometry.atlas_index(92), Some(92), "last frame");
}

#[test]
fn atlas_index_rejects_out_of_range_frames() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);

    assert_eq!(geometry.atlas_index(93), None, "one past the last frame");
    assert_eq!(geometry.atlas_index(u32::MAX), None, "far out of range");
}
