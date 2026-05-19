use bevyrogue::animation::{AnimGraph, FrameCueCommand, NodeId, Predicate, ReleaseKernelCue};

#[test]
fn agumon_anim_graph_parses() {
    let ron_str = include_str!("../assets/digimon/agumon/anim_graph.ron");
    let graph: AnimGraph = ron::from_str(ron_str)
        .expect("assets/digimon/agumon/anim_graph.ron should parse");
    assert_eq!(graph.id.0, "agumon_skill");
    assert_eq!(graph.clip.0, "all");
    assert!(graph.nodes.contains_key(&NodeId("baby_flame_cast".into())));
    assert!(graph.nodes.contains_key(&NodeId("sharp_claws_windup".into())));
    assert!(graph.nodes.contains_key(&NodeId("sharp_claws_strike".into())));
    assert!(graph.nodes.contains_key(&NodeId("sharp_claws_recover".into())));
}

#[test]
fn agumon_sharp_claws_release_kernel_cue_parses() {
    let graph: AnimGraph = ron::from_str(include_str!("../assets/digimon/agumon/anim_graph.ron"))
        .expect("agumon anim_graph.ron should parse");
    let strike = &graph.nodes[&NodeId("sharp_claws_strike".into())];

    assert_eq!(strike.frames.start(), 3, "Sharp Claws strike should stay inside the attack atlas span");
    assert_eq!(strike.frames.end(), 5, "Sharp Claws strike should stay inside the attack atlas span");
    assert_eq!(strike.cues.len(), 1, "Sharp Claws strike should emit exactly one kernel release cue");
    assert_eq!(strike.cues[0].at, 1, "Sharp Claws should release on its authored impact frame");
    assert!(
        matches!(strike.cues[0].command, FrameCueCommand::ReleaseKernel(ReleaseKernelCue)),
        "Sharp Claws strike cue should be ReleaseKernel"
    );

    let edge = graph
        .transitions
        .iter()
        .find(|edge| edge.from == NodeId("sharp_claws_strike".into()))
        .expect("Sharp Claws strike transition should exist");
    assert!(
        matches!(edge.when, Predicate::KernelCue),
        "Sharp Claws strike should wait on KernelCue before recovery"
    );
}

#[test]
fn malformed_anim_graph_ron_fails_to_parse() {
    let malformed = r#"(
        id: "broken",
        clip: "all",
        entry: "cast",
        nodes: {
            "cast": (frames: (0, 1)),
        },
        transitions: [
            (from: "cast", to: Exit, when: Always)
    "#;

    let result = ron::from_str::<AnimGraph>(malformed);
    assert!(result.is_err(), "malformed asset-like RON must fail to parse");
}

#[test]
fn renamon_anim_graph_parses() {
    let ron_str = include_str!("../assets/digimon/renamon/anim_graph.ron");
    let graph: AnimGraph = ron::from_str(ron_str)
        .expect("assets/digimon/renamon/anim_graph.ron should parse");
    assert_eq!(graph.id.0, "renamon_skill");
    assert!(graph.nodes.contains_key(&NodeId("diamond_storm_cast".into())));
}
