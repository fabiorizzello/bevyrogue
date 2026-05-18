use std::collections::BTreeMap;

use bevyrogue::animation::{Clip, ClipMeta, ClipRange, FrameSize};

#[test]
fn valid_clip_parses_into_typed_geometry() {
    let clip: Clip = ron::from_str(
        r#"(
            meta: (
                frame_size: (w: 557, h: 561),
                columns: 10,
                rows: 10,
                total_frames: 95,
            ),
            ranges: {
                "attack": (start: 0, end: 8),
                "idle": (start: 54, end: 59),
                "victory": (start: 78, end: 94),
            },
        )"#,
    )
    .expect("valid clip should parse");

    assert_eq!(
        clip.meta,
        ClipMeta {
            frame_size: FrameSize { w: 557, h: 561 },
            columns: 10,
            rows: 10,
            total_frames: 95,
        }
    );
    assert_eq!(
        clip.ranges,
        BTreeMap::from([
            ("attack".into(), ClipRange { start: 0, end: 8 }),
            ("idle".into(), ClipRange { start: 54, end: 59 }),
            ("victory".into(), ClipRange { start: 78, end: 94 }),
        ])
    );
    assert_eq!(clip.ranges["attack"].len(), 9);
    assert!(clip.ranges["idle"].contains(57));
    assert!(!clip.ranges["idle"].contains(60));
}

#[test]
fn unknown_clip_field_is_rejected() {
    let err = ron::from_str::<Clip>(
        r#"(
            meta: (
                frame_size: (w: 557, h: 561),
                columns: 10,
                rows: 10,
                total_frames: 95,
                fps: 12,
            ),
            ranges: {
                "idle": (start: 54, end: 59),
            },
        )"#,
    )
    .expect_err("unknown field should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("fps") || msg.contains("Unexpected") || msg.contains("unknown field"),
        "unexpected parse error: {msg}"
    );
}

#[test]
fn malformed_range_shape_is_rejected() {
    let err = ron::from_str::<Clip>(
        r#"(
            meta: (
                frame_size: (w: 557, h: 561),
                columns: 10,
                rows: 10,
                total_frames: 95,
            ),
            ranges: {
                "idle": (start_index: 54, end: 59),
            },
        )"#,
    )
    .expect_err("malformed range should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("start") || msg.contains("start_index") || msg.contains("unknown field"),
        "unexpected parse error: {msg}"
    );
}
