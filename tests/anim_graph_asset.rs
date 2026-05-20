use bevyrogue::animation::{AnimGraph, NodeId};

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
