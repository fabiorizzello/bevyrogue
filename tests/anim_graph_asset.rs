use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use bevy::{asset::AssetPlugin, prelude::*};
use bevyrogue::animation::{
    AnimGraph, AnimationAssetPlugin, AnimationGraphHandles, AnimationGraphLoadState, ClipId,
    Command, NodeId, ParamRef, PlaybackModifier, StatusId, TargetShape, TransitionTarget, VfxLocus,
    VfxMotion,
};

fn manifest_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn build_asset_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: manifest_assets_dir().to_string_lossy().into_owned(),
        watch_for_changes_override: Some(false),
        ..default()
    });
    app.add_plugins(AnimationAssetPlugin);
    app
}

#[test]
fn agumon_anim_graph_loads_as_typed_asset_before_ready_flips() {
    let mut app = build_asset_app();

    app.update();

    {
        let state = app.world().resource::<AnimationGraphLoadState>();
        let handles = app.world().resource::<AnimationGraphHandles>();
        assert_eq!(
            handles.0.len(),
            2,
            "expected two configured graphs (Agumon + Renamon)"
        );
        assert_eq!(
            state.loaded,
            vec![false, false],
            "load events should not exist before updates settle"
        );
        assert!(
            !state.ready,
            "ready must start false before any asset event"
        );
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let poll_interval = Duration::from_millis(10);

    loop {
        app.update();

        let (asset_present, ready, loaded_flags) = {
            let world = app.world();
            let state = world.resource::<AnimationGraphLoadState>();
            let handles = world.resource::<AnimationGraphHandles>();
            let graphs = world.resource::<Assets<AnimGraph>>();
            let asset_present = graphs.get(&handles.0[0]).is_some();
            (asset_present, state.ready, state.loaded.clone())
        };

        if !asset_present {
            assert!(
                !ready,
                "loader reported ready before the graph became readable from Assets<AnimGraph>"
            );
        }

        if ready {
            assert_eq!(
                loaded_flags,
                vec![true, true],
                "ready requires all graphs to have a prior load/modify event"
            );
            break;
        }

        assert!(
            start.elapsed() < timeout,
            "timed out waiting for all anim_graphs to load; last loaded flags: {loaded_flags:?}"
        );
        thread::sleep(poll_interval);
    }

    let world = app.world();
    let state = world.resource::<AnimationGraphLoadState>();
    let handles = world.resource::<AnimationGraphHandles>();
    let graphs = world.resource::<Assets<AnimGraph>>();
    let graph = graphs
        .get(&handles.0[0])
        .expect("ready state must correspond to an available AnimGraph asset");

    assert!(state.ready);
    assert_eq!(state.loaded, vec![true, true]);
    assert_eq!(graph.clip, ClipId("skill".into()));
    assert_eq!(graph.entry, NodeId("baby_flame_cast".into()));
    assert_eq!(graph.transitions.len(), 3);
    assert_eq!(
        graph.transitions[0].to,
        TransitionTarget::Node(NodeId("baby_flame_impact".into()))
    );
    assert_eq!(graph.transitions[2].to, TransitionTarget::Exit);
    assert_eq!(
        graph.nodes[&NodeId("baby_flame_cast".into())].frames,
        bevyrogue::animation::FrameRange(60, 68)
    );
    assert_eq!(
        graph.nodes[&NodeId("baby_flame_impact".into())].modifier,
        Some(PlaybackModifier::Hold { extra_frames: 2 })
    );

    match &graph.nodes[&NodeId("baby_flame_cast".into())].on_enter[0] {
        Command::SpawnParticle {
            name,
            origin,
            motion,
        } => {
            assert_eq!(name.0, "baby_flame");
            assert_eq!(origin, &VfxLocus::CasterCenter);
            assert_eq!(motion, &VfxMotion::ArcToTarget);
        }
        other => panic!("expected SpawnParticle, got {other:?}"),
    }

    match &graph.nodes[&NodeId("baby_flame_impact".into())].on_enter[0] {
        Command::EmitDamage {
            hits,
            mul,
            status,
            chance_pct,
            duration,
            target,
        } => {
            assert_eq!(hits, &ParamRef::Literal(1));
            assert_eq!(mul, &ParamRef::Literal(18));
            assert_eq!(status, &Some(StatusId("Heated".into())));
            assert_eq!(chance_pct, &None);
            assert_eq!(duration, &Some(ParamRef::Literal(3)));
            assert_eq!(target, &TargetShape::Primary);
        }
        other => panic!("expected EmitDamage, got {other:?}"),
    }
}
