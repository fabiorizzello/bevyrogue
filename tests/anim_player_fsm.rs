use bevyrogue::animation::{AnimGraph, AnimGraphPlayer, NodeId};

fn parse_stance() -> AnimGraph {
    ron::from_str(include_str!("../assets/digimon/agumon/stance.ron"))
        .expect("stance.ron should parse")
}

fn parse_graph(ron_str: &str) -> AnimGraph {
    ron::from_str(ron_str).expect("inline graph should parse")
}

// --- basic frame derivation ---

#[test]
fn player_starts_at_entry_node_first_frame() {
    let graph = parse_stance();
    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    // idle range is 54-59; first advance returns frame 54
    assert_eq!(player.advance(&graph), 54);
}

#[test]
fn player_cycles_idle_through_loop_modifier() {
    let graph = parse_stance();
    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    let range_len = 6u32; // frames 54-59
    for _ in 0..range_len * 3 {
        let frame = player.advance(&graph);
        assert!((54..=59).contains(&frame), "idle frame must stay within 54-59, got {frame}");
    }
}

#[test]
fn player_idle_does_not_leave_node_with_infinite_loop() {
    let graph = parse_stance();
    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    for _ in 0..300 {
        player.advance(&graph);
    }
    assert_eq!(
        player.current_node.0, "idle",
        "infinite-loop idle must not transition away"
    );
}

#[test]
fn player_returns_zero_for_missing_current_node() {
    let graph = parse_stance();
    let mut player = AnimGraphPlayer::new(NodeId("missing".into()));

    player.fire_kernel_cue();

    assert_eq!(player.advance(&graph), 0, "missing nodes must keep the safe frame-0 fallback");
    assert_eq!(player.current_node.0, "missing");
    assert_eq!(player.elapsed_anim_frames, 0);
}

// --- TimeInNode transitions ---

#[test]
fn player_transitions_on_time_in_node() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (frames: (0, 3)),
            "recover": (frames: (4, 6)),
        },
        transitions: [
            (from: "cast", to: Node("recover"), when: TimeInNode),
            (from: "recover", to: Exit, when: TimeInNode),
        ]
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    // cast: 4 frames (0-3), elapsed reaches 4 after the 4th advance.
    // On the 5th advance, TimeInNode (elapsed >= 4) fires → transition.
    for _ in 0..5 {
        player.advance(&graph);
    }
    assert_eq!(
        player.current_node.0, "recover",
        "player must transition to recover after cast duration"
    );
}

// --- KernelCue transitions ---

#[test]
fn player_kernel_cue_transition_waits_for_signal() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "strike",
        nodes: {
            "strike": (frames: (0, 2)),
            "recover": (frames: (3, 5)),
        },
        transitions: [
            (from: "strike", to: Node("recover"), when: KernelCue),
            (from: "recover", to: Exit, when: TimeInNode),
        ]
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    let frames: Vec<u32> = (0..5).map(|_| player.advance(&graph)).collect();
    assert_eq!(frames, vec![0, 1, 2, 2, 2], "strike must clamp on its last frame before release");
    assert_eq!(player.current_node.0, "strike");
    assert_eq!(player.elapsed_anim_frames, 5, "elapsed ticks should keep climbing while the cue is blocked");

    player.fire_kernel_cue();

    assert_eq!(player.advance(&graph), 3, "release should jump to recover's first frame");
    assert_eq!(player.current_node.0, "recover");
    assert_eq!(player.elapsed_anim_frames, 0, "transitioning on KernelCue must reset elapsed ticks");
}

#[test]
fn player_kernel_cue_consumes_signal_once() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "windup",
        nodes: {
            "windup": (frames: (0, 1)),
            "strike": (frames: (2, 3)),
            "recover": (frames: (4, 5)),
        },
        transitions: [
            (from: "windup", to: Node("strike"), when: KernelCue),
            (from: "strike", to: Node("recover"), when: KernelCue),
            (from: "recover", to: Exit, when: TimeInNode),
        ]
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());

    player.fire_kernel_cue();
    assert_eq!(player.advance(&graph), 2, "first cue should enter strike");
    assert_eq!(player.current_node.0, "strike");
    assert_eq!(player.elapsed_anim_frames, 0);

    assert_eq!(player.advance(&graph), 2, "stale cue must not satisfy strike -> recover");
    assert_eq!(player.current_node.0, "strike");
    assert_eq!(player.elapsed_anim_frames, 1);

    player.fire_kernel_cue();
    assert_eq!(player.advance(&graph), 4, "second cue should enter recover");
    assert_eq!(player.current_node.0, "recover");
    assert_eq!(player.elapsed_anim_frames, 0);
}

// --- Always transitions ---

#[test]
fn player_always_transition_fires_immediately() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "a",
        nodes: {
            "a": (frames: (0, 3)),
            "b": (frames: (4, 6)),
        },
        transitions: [
            (from: "a", to: Node("b"), when: Always),
            (from: "b", to: Exit, when: Always),
        ]
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    // First advance: Always fires, transitions to "b"
    player.advance(&graph);
    assert_eq!(player.current_node.0, "b");
}

// --- PlaybackModifier::Hold ---

#[test]
fn hold_modifier_extends_last_frame() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "hit",
        nodes: {
            "hit": (frames: (0, 1), modifier: Some(Hold(extra_frames: 3))),
        },
        transitions: []
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    let frames: Vec<u32> = (0..6).map(|_| player.advance(&graph)).collect();
    // range_len=2 (frames 0-1); hold adds 3 more at frame 1
    // elapsed: 0→frame 0, 1→frame 1, 2→frame 1 (clamped), 3→1, 4→1, 5→1
    assert_eq!(frames, vec![0, 1, 1, 1, 1, 1]);
}

// --- reverse ---

#[test]
fn reverse_flag_plays_range_backwards() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "back",
        nodes: {
            "back": (frames: (0, 2), reverse: true),
        },
        transitions: []
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    assert_eq!(player.advance(&graph), 2); // frame 2 first
    assert_eq!(player.advance(&graph), 1);
    assert_eq!(player.advance(&graph), 0);
}

// --- SpeedMul ---

#[test]
fn speed_mul_200pct_advances_two_anim_frames_per_tick() {
    let graph = parse_graph(r#"(
        id: "test",
        clip: "skill",
        entry: "fast",
        nodes: {
            "fast": (frames: (0, 9), modifier: Some(SpeedMul(pct: 200))),
        },
        transitions: []
    )"#);

    let mut player = AnimGraphPlayer::new(graph.entry.clone());
    // At 200%, tick 0 → anim frame 0*200/100=0, tick 1 → 1*200/100=2
    assert_eq!(player.advance(&graph), 0);
    assert_eq!(player.advance(&graph), 2);
    assert_eq!(player.advance(&graph), 4);
}
