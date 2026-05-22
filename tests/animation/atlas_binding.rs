use bevyrogue::animation::{
    AnimGraph, AnimGraphPlayer, AnimNode, AtlasGeometry, Clip, FrameCueCommand, NodeId,
};

fn parse_clip(path: &str) -> Clip {
    let text = std::fs::read_to_string(format!(
        "{}/assets/digimon/{path}/clip.ron",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("clip.ron should be readable");
    ron::from_str(&text).expect("clip.ron should parse")
}

/// Agumon idle clip range, authored in `clip.ron` as `idle: (53, 58)`.
const IDLE_RANGE: std::ops::RangeInclusive<u32> = 53..=58;
/// Agumon attack clip range, authored in `clip.ron` as `attack: (0, 8)`.
const ATTACK_RANGE: std::ops::RangeInclusive<u32> = 0..=8;

const AGUMON_STANCE_RON: &str = include_str!("../../assets/digimon/agumon/stance.ron");
const AGUMON_SKILL_RON: &str = include_str!("../../assets/digimon/agumon/anim_graph.ron");

fn parse_graph(text: &str) -> AnimGraph {
    ron::from_str(text).expect("anim graph RON should parse")
}

/// Inverse of `local_frame_for` (src/windowed/render.rs): map a cue's local `at`
/// back to a clip frame within the node's `FrameRange`, honoring `reverse`.
fn clip_frame_at_cue(node: &AnimNode, at: u32) -> u32 {
    if node.reverse {
        node.frames.end().saturating_sub(at)
    } else {
        node.frames.start() + at
    }
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

/// (a) Idle parity: the stance entry node loops within [53,58] and each
/// player frame maps 1:1 onto the atlas tile index.
#[test]
fn idle_player_frames_map_identity_within_idle_range() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);
    let graph = parse_graph(AGUMON_STANCE_RON);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    // More ticks than the 6-frame idle loop to cross the wrap boundary.
    for tick in 0..24 {
        let frame = player.advance_result(&graph).frame;
        assert!(
            IDLE_RANGE.contains(&frame),
            "tick {tick}: frame {frame} outside idle range {IDLE_RANGE:?}"
        );
        assert_eq!(
            geometry.atlas_index(frame),
            Some(frame),
            "tick {tick}: atlas index must be identity for frame {frame}"
        );
    }
}

/// (b) Sharp Claws parity: driving windup -> strike -> recover keeps every
/// player frame inside the attack range [0,8] and identity-mapped.
#[test]
fn sharp_claws_player_frames_map_identity_within_attack_range() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);
    let graph = parse_graph(AGUMON_SKILL_RON);

    let mut player = AnimGraphPlayer::new(NodeId("sharp_claws_windup".into()));
    for tick in 0..16 {
        // The strike->recover edge is gated on the kernel cue; fire it once
        // partway so the player advances through the whole sequence.
        if tick == 6 {
            player.fire_kernel_cue();
        }
        let result = player.advance_result(&graph);
        assert!(
            ATTACK_RANGE.contains(&result.frame),
            "tick {tick}: frame {} outside attack range {ATTACK_RANGE:?}",
            result.frame
        );
        assert_eq!(
            geometry.atlas_index(result.frame),
            Some(result.frame),
            "tick {tick}: atlas index must be identity for frame {}",
            result.frame
        );
        if result.exited {
            break;
        }
    }
}

/// (c) Impact-frame-on-rendered-frame invariant: the atlas tile rendered at the
/// Sharp Claws `ReleaseKernel` cue is the impact frame — resolved from the
/// loaded graph, never hardcoded.
#[test]
fn sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile() {
    let clip = parse_clip("agumon");
    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);
    let graph = parse_graph(AGUMON_SKILL_RON);

    // Locate the Sharp Claws node carrying a ReleaseKernel cue (resolved from
    // the graph, not assumed by name).
    let (node_id, node, cue_at) = graph
        .nodes
        .iter()
        .filter(|(id, _)| id.0.starts_with("sharp_claws"))
        .find_map(|(id, node)| {
            node.cues
                .iter()
                .find(|cue| matches!(cue.command, FrameCueCommand::ReleaseKernel(_)))
                .map(|cue| (id, node, cue.at))
        })
        .expect("a sharp_claws node must carry a ReleaseKernel cue");

    let impact_frame = clip_frame_at_cue(node, cue_at);

    assert!(
        ATTACK_RANGE.contains(&impact_frame),
        "impact frame {impact_frame} for node {} outside attack range {ATTACK_RANGE:?}",
        node_id.0
    );
    assert_eq!(
        geometry.atlas_index(impact_frame),
        Some(impact_frame),
        "rendered atlas tile at the release cue must be the identity-mapped impact frame"
    );
}
