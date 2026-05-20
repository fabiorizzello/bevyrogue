use bevyrogue::animation::{AnimGraph, FrameCueCommand, ReleaseKernelCue};
use rstest::rstest;

#[rstest]
#[case::missing_id_field(r#"(
    clip: "skill",
    entry: "idle",
    nodes: { "idle": (frames: (0, 3)) },
    transitions: []
)"#)]
#[case::unknown_frame_cue_variant(r#"(
    id: "test_graph",
    clip: "skill",
    entry: "idle",
    nodes: {
        "idle": (
            frames: (0, 3),
            cues: [(at: 1, command: UnknownVariant)]
        )
    },
    transitions: []
)"#)]
#[case::unknown_top_level_field(r#"(
    id: "test_graph",
    clip: "skill",
    entry: "idle",
    nodes: { "idle": (frames: (0, 3)) },
    transitions: [],
    unknown_field: "should_fail"
)"#)]
fn anim_graph_rejects_invalid_ron(#[case] input: &str) {
    assert!(
        ron::from_str::<AnimGraph>(input).is_err(),
        "AnimGraph deserialization should have failed"
    );
}

#[test]
fn cues_absent_graph_loads_with_default_empty() {
    let ron_str = r#"(
        id: "test_graph",
        clip: "skill",
        entry: "idle",
        nodes: { "idle": (frames: (0, 3)) },
        transitions: []
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("cues-absent graph should load");
    assert!(
        graph.nodes[&bevyrogue::animation::NodeId("idle".into())].cues.is_empty(),
        "cues should default to empty vec"
    );
}

#[test]
fn graph_with_release_kernel_cue_parses() {
    let ron_str = r#"(
        id: "test_graph",
        clip: "skill",
        entry: "idle",
        nodes: {
            "idle": (
                frames: (0, 3),
                cues: [
                    (at: 2, command: ReleaseKernel(())),
                ]
            )
        },
        transitions: []
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("graph with ReleaseKernelCue should parse");
    let cues = &graph.nodes[&bevyrogue::animation::NodeId("idle".into())].cues;
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].at, 2);
    assert!(
        matches!(cues[0].command, FrameCueCommand::ReleaseKernel(ReleaseKernelCue)),
        "command should be ReleaseKernel"
    );
}

#[test]
fn graph_with_presentation_cue_parses() {
    let ron_str = r#"(
        id: "test_graph",
        clip: "skill",
        entry: "idle",
        nodes: {
            "idle": (
                frames: (0, 3),
                cues: [
                    (at: 1, command: Presentation(SpawnParticle(
                        name: "fx",
                        origin: CasterCenter,
                        motion: Static,
                    ))),
                ]
            )
        },
        transitions: []
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("graph with Presentation cue should parse");
    let cues = &graph.nodes[&bevyrogue::animation::NodeId("idle".into())].cues;
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].at, 1);
    assert!(
        matches!(cues[0].command, FrameCueCommand::Presentation(_)),
        "command should be Presentation"
    );
}

#[test]
fn kernel_cue_predicate_parses_in_transition() {
    let ron_str = r#"(
        id: "test_graph",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (frames: (0, 5)),
            "recover": (frames: (6, 9)),
        },
        transitions: [
            (from: "cast", to: Node("recover"), when: KernelCue),
            (from: "recover", to: Exit, when: Always),
        ]
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("KernelCue predicate should parse");
    let cast_to_recover = graph.transitions.iter().find(|e| e.from.0 == "cast").unwrap();
    assert!(
        matches!(cast_to_recover.when, bevyrogue::animation::Predicate::KernelCue),
        "predicate should be KernelCue"
    );
}
