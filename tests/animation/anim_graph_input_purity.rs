use bevyrogue::animation::{AnimGraph, AnimGraphInput, AnimGraphPlayer, AnimGraphRole};

fn parse_graph(ron_str: &str) -> AnimGraph {
    ron::from_str(ron_str).expect("inline graph should parse")
}

#[test]
fn graph_input_parses_closed_typed_roles() {
    let input: AnimGraphInput = ron::from_str(
        r#"(
        roles: [Caster, PrimaryTarget, AdjacentLeftTarget, AdjacentRightTarget],
    )"#,
    )
    .expect("typed graph input should parse");

    assert!(input.contains(AnimGraphRole::Caster));
    assert!(input.contains(AnimGraphRole::PrimaryTarget));
    assert!(input.contains(AnimGraphRole::AdjacentLeftTarget));
    assert!(input.contains(AnimGraphRole::AdjacentRightTarget));
}

#[test]
fn graph_input_rejects_unknown_or_stringly_roles() {
    for invalid in [
        r#"(roles: ["caster"])"#,
        r#"(roles: [BossTarget])"#,
        r#"(roles: [Custom("boss")])"#,
    ] {
        assert!(
            ron::from_str::<AnimGraphInput>(invalid).is_err(),
            "invalid typed graph input should fail: {invalid}"
        );
    }
}

#[test]
fn player_accepts_explicit_read_only_typed_input_without_world_context() {
    let graph = parse_graph(
        r#"(
        id: "test",
        clip: "skill",
        entry: "windup",
        nodes: {
            "windup": (frames: (0, 1)),
            "recover": (frames: (2, 3)),
        },
        transitions: [
            (from: "windup", to: Node("recover"), when: KernelCue),
            (from: "recover", to: Exit, when: TimeInNode),
        ]
    )"#,
    );
    let input = AnimGraphInput::new([
        AnimGraphRole::Caster,
        AnimGraphRole::PrimaryTarget,
        AnimGraphRole::AdjacentLeftTarget,
    ]);
    let snapshot = input.clone();
    let advance_with_input: fn(&mut AnimGraphPlayer, &AnimGraph, &AnimGraphInput) -> u32 =
        AnimGraphPlayer::advance_with_input;
    let advance_result_with_input: fn(
        &mut AnimGraphPlayer,
        &AnimGraph,
        &AnimGraphInput,
    ) -> bevyrogue::animation::AnimAdvanceResult = AnimGraphPlayer::advance_result_with_input;

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    assert_eq!(advance_with_input(&mut player, &graph, &input), 0);
    player.fire_kernel_cue();
    let result = advance_result_with_input(&mut player, &graph, &input);

    assert_eq!(result.frame, 2, "kernel cue should transition into recover");
    assert!(!result.exited);
    assert_eq!(input, snapshot, "typed graph input must stay read-only");
}

#[test]
fn legacy_player_entrypoint_remains_behaviorally_equivalent_to_default_input() {
    let graph = parse_graph(
        r#"(
        id: "test",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (frames: (0, 2)),
            "recover": (frames: (3, 4)),
        },
        transitions: [
            (from: "cast", to: Node("recover"), when: TimeInNode),
            (from: "recover", to: Exit, when: TimeInNode),
        ]
    )"#,
    );
    let input = AnimGraphInput::new([AnimGraphRole::Caster, AnimGraphRole::PrimaryTarget]);
    let mut legacy = AnimGraphPlayer::new(graph.entry.clone());
    let mut explicit = AnimGraphPlayer::new(graph.entry.clone());

    for _ in 0..6 {
        assert_eq!(
            legacy.advance(&graph),
            explicit.advance_with_input(&graph, &input)
        );
    }
}
