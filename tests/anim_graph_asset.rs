use bevyrogue::animation::{AnimGraph, NodeId};
use rstest::rstest;

#[rstest]
#[case::agumon(
    include_str!("../assets/digimon/agumon/anim_graph.ron"),
    "agumon_skill",
    Some("all"),
    &["baby_flame_cast", "sharp_claws_windup", "sharp_claws_strike", "sharp_claws_recover"],
)]
#[case::renamon(
    include_str!("../assets/digimon/renamon/anim_graph.ron"),
    "renamon_skill",
    None,
    &["diamond_storm_cast"],
)]
fn shipped_anim_graph_asset_parses(
    #[case] ron_str: &str,
    #[case] expected_id: &str,
    #[case] expected_clip: Option<&str>,
    #[case] expected_node_keys: &[&str],
) {
    let graph: AnimGraph =
        ron::from_str(ron_str).unwrap_or_else(|err| panic!("asset must parse: {err}"));
    assert_eq!(graph.id.0, expected_id);
    if let Some(clip) = expected_clip {
        assert_eq!(graph.clip.0, clip);
    }
    for key in expected_node_keys {
        assert!(
            graph.nodes.contains_key(&NodeId((*key).into())),
            "missing node `{key}` in {expected_id}"
        );
    }
}
