use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use bevy::{asset::AssetPlugin, prelude::*};
use bevyrogue::animation::{
    AnimationAssetPlugin, AnimationClipHandles, AnimationClipLoadState, Clip,
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
fn agumon_clip_loads_as_typed_asset_before_ready_flips() {
    let mut app = build_asset_app();

    app.update();

    {
        let state = app.world().resource::<AnimationClipLoadState>();
        let handles = app.world().resource::<AnimationClipHandles>();
        assert_eq!(
            handles.0.len(),
            2,
            "expected two configured clips (Agumon + Renamon)"
        );
        assert_eq!(
            state.loaded,
            vec![false, false],
            "load events should not exist before updates settle"
        );
        assert!(!state.ready, "ready must start false before any asset event");
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let poll_interval = Duration::from_millis(10);

    loop {
        app.update();

        let (asset_present, ready, loaded_flags) = {
            let world = app.world();
            let state = world.resource::<AnimationClipLoadState>();
            let handles = world.resource::<AnimationClipHandles>();
            let clips = world.resource::<Assets<Clip>>();
            let asset_present = clips.get(&handles.0[0]).is_some();
            (asset_present, state.ready, state.loaded.clone())
        };

        if !asset_present {
            assert!(
                !ready,
                "loader reported ready before the clip became readable from Assets<Clip>"
            );
        }

        if ready {
            assert_eq!(
                loaded_flags,
                vec![true, true],
                "ready requires all clips to have a prior load/modify event"
            );
            break;
        }

        assert!(
            start.elapsed() < timeout,
            "timed out waiting for all clip.ron files to load; last loaded flags: {loaded_flags:?}"
        );
        thread::sleep(poll_interval);
    }

    let world = app.world();
    let state = world.resource::<AnimationClipLoadState>();
    let handles = world.resource::<AnimationClipHandles>();
    let clips = world.resource::<Assets<Clip>>();
    let clip = clips
        .get(&handles.0[0])
        .expect("ready state must correspond to an available Clip asset");

    assert!(state.ready);
    assert_eq!(state.loaded, vec![true, true]);
    assert_eq!(clip.meta.frame_size.w, 557);
    assert_eq!(clip.meta.frame_size.h, 561);
    assert_eq!(clip.meta.columns, 10);
    assert_eq!(clip.meta.rows, 10);
    assert_eq!(clip.meta.total_frames, 95);

    let attack = clip.ranges.get("attack").expect("attack range should exist");
    assert_eq!(attack.start, 0);
    assert_eq!(attack.end, 8);
    assert_eq!((*attack).len(), 9);
    assert!((*attack).contains(4));
    assert!(!(*attack).contains(9));

    let skill = clip.ranges.get("skill").expect("skill range should exist");
    assert_eq!(skill.start, 60);
    assert_eq!(skill.end, 77);
    assert_eq!((*skill).len(), 18);

    let victory = clip
        .ranges
        .get("victory")
        .expect("victory range should exist");
    assert_eq!(victory.start, 78);
    assert_eq!(victory.end, 94);
    assert_eq!(clip.ranges.len(), 8);
}
