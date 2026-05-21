use bevy::prelude::Assets;
use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationGraphLoadState,
    AnimationGraphLookupDiagnostics, MISSING_GRAPH_FALLBACK_NODE_ID, ResolvedAnimGraphSource,
    SkillGraphRegistry,
};

fn parse_graph(ron_str: &str) -> AnimGraph {
    ron::from_str(ron_str).expect("inline graph should parse")
}

#[test]
fn missing_skill_graph_runtime_lookup_uses_structured_instant_fallback() {
    let registry = SkillGraphRegistry::default();
    let graphs = Assets::<AnimGraph>::default();
    let mut diagnostics = AnimationGraphLookupDiagnostics::default();
    let requested = AnimGraphId("missing_skill_graph".into());

    let resolved =
        registry.resolve_snapshot_or_instant_fallback(&requested, &graphs, &mut diagnostics);

    assert_eq!(resolved.source, ResolvedAnimGraphSource::InstantFallback);
    assert_eq!(resolved.requested_id, requested);
    assert_eq!(resolved.graph().id, requested);
    assert_eq!(resolved.graph().entry.0, MISSING_GRAPH_FALLBACK_NODE_ID);
    assert_eq!(resolved.graph().nodes.len(), 1);
    assert_eq!(resolved.graph().transitions.len(), 1);

    let diagnostic = diagnostics
        .last_missing_skill_graph
        .as_ref()
        .expect("missing skill lookup must persist a structured diagnostic");
    assert_eq!(diagnostic.registry, "skill");
    assert_eq!(diagnostic.requested_id, requested);
    assert_eq!(diagnostic.fallback_node.0, MISSING_GRAPH_FALLBACK_NODE_ID);
    assert!(
        diagnostic.message.contains("missing_or_unloaded"),
        "diagnostic should explain the fallback reason: {}",
        diagnostic.message
    );
    assert!(
        diagnostics
            .last_message
            .as_deref()
            .is_some_and(|msg| msg.contains("graph_id=missing_skill_graph")),
        "last_message should expose the missing graph id for later inspection"
    );

    let mut player = AnimGraphPlayer::new(resolved.graph().entry.clone());
    let result = player.advance_result(resolved.graph());
    assert_eq!(result.frame, 0, "instant fallback should stay deterministic");
    assert!(result.exited, "instant fallback should exit in one tick");
}

#[test]
fn graph_snapshot_hot_reload_only_affects_newly_resolved_players() {
    let mut registry = SkillGraphRegistry::default();
    let mut graphs = Assets::<AnimGraph>::default();
    let requested = AnimGraphId("agumon_skill".into());

    let v1 = parse_graph(
        r#"(
        id: "agumon_skill",
        clip: "all",
        entry: "cast",
        nodes: {
            "cast": (frames: (10, 10)),
        },
        transitions: []
    )"#,
    );
    let handle = graphs.add(v1.clone());
    registry.0.insert(requested.clone(), handle.clone());

    let resolved_v1 = registry
        .resolve_snapshot(&requested, &graphs)
        .expect("initial registry snapshot should resolve");
    let mut inflight_player = AnimGraphPlayer::new(resolved_v1.graph().entry.clone());
    assert_eq!(inflight_player.advance(resolved_v1.graph()), 10);

    let v2 = parse_graph(
        r#"(
        id: "agumon_skill",
        clip: "all",
        entry: "cast",
        nodes: {
            "cast": (frames: (42, 42)),
        },
        transitions: []
    )"#,
    );
    *graphs
        .get_mut(&handle)
        .expect("hot reload should mutate the existing asset handle") = v2;

    let resolved_v2 = registry
        .resolve_snapshot(&requested, &graphs)
        .expect("new players should resolve the reloaded graph snapshot");
    let mut new_player = AnimGraphPlayer::new(resolved_v2.graph().entry.clone());

    assert_eq!(
        inflight_player.advance(resolved_v1.graph()),
        10,
        "the in-flight player must keep its previously bound graph snapshot"
    );
    assert_eq!(
        new_player.advance(resolved_v2.graph()),
        42,
        "a newly resolved player must pick up the hot-reloaded graph"
    );
    assert_eq!(resolved_v1.source, ResolvedAnimGraphSource::Registry);
    assert_eq!(resolved_v2.source, ResolvedAnimGraphSource::Registry);
}

#[test]
fn graph_load_state_tracks_boot_failures_structurally() {
    let state = AnimationGraphLoadState {
        loaded: vec![true, true],
        failed_paths: vec!["digimon/agumon/anim_graph.ron".into()],
        ready: true,
    };

    assert!(state.has_boot_failures());
    assert_eq!(
        state.failed_paths(),
        ["digimon/agumon/anim_graph.ron"],
        "boot-time canonical graph failures should remain inspectable"
    );
}
