use bevyrogue::animation::AnimGraph;

#[test]
fn agumon_anim_graph_parses() {
    let ron_str = include_str!("../assets/digimon/agumon/anim_graph.ron");
    let graph: AnimGraph = ron::from_str(ron_str)
        .expect("assets/digimon/agumon/anim_graph.ron should parse");
    assert_eq!(graph.id.0, "agumon_skill");
    assert!(graph.nodes.contains_key(&bevyrogue::animation::NodeId("baby_flame_cast".into())));
}

#[test]
fn renamon_anim_graph_parses() {
    let ron_str = include_str!("../assets/digimon/renamon/anim_graph.ron");
    let graph: AnimGraph = ron::from_str(ron_str)
        .expect("assets/digimon/renamon/anim_graph.ron should parse");
    assert_eq!(graph.id.0, "renamon_skill");
    assert!(graph.nodes.contains_key(&bevyrogue::animation::NodeId("diamond_storm_cast".into())));
}
