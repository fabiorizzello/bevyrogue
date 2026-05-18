use std::{
    collections::BTreeSet,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use bevy::{asset::AssetPlugin, prelude::*};
use bevyrogue::animation::{
    AnimationAssetPlugin, AnimationClipPaths, AnimationGraphPaths, AnimationValidationCatalogs,
    AnimationValidationCheck, AnimationValidationReason, AnimationValidationState, ParticleId,
    StatusId,
};

fn manifest_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn build_asset_app(
    graph_path: &str,
    clip_path: &str,
    catalogs: AnimationValidationCatalogs,
) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: manifest_assets_dir().to_string_lossy().into_owned(),
        watch_for_changes_override: Some(false),
        ..default()
    });
    app.insert_resource(AnimationGraphPaths(vec![graph_path.to_string()]));
    app.insert_resource(AnimationClipPaths(vec![clip_path.to_string()]));
    app.insert_resource(catalogs);
    app.add_plugins(AnimationAssetPlugin);
    app
}

fn wait_for_validation(app: &mut App) -> AnimationValidationState {
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let poll_interval = Duration::from_millis(10);

    loop {
        app.update();

        let state = app.world().resource::<AnimationValidationState>().clone();
        if state.has_run() {
            return state;
        }

        assert!(
            start.elapsed() < timeout,
            "timed out waiting for animation validation state to settle"
        );
        thread::sleep(poll_interval);
    }
}

fn has_diag(
    diags: &[bevyrogue::animation::AnimationValidationDiagnostic],
    check: AnimationValidationCheck,
    reason: AnimationValidationReason,
) -> bool {
    diags
        .iter()
        .any(|diag| diag.check == check && diag.reason == reason)
}

#[test]
fn valid_assets_set_plugin_validation_ready() {
    let mut app = build_asset_app(
        "test/animation_validation/valid_anim_graph.ron",
        "test/animation_validation/valid_clip.ron",
        AnimationValidationCatalogs {
            params: BTreeSet::new(),
            statuses: BTreeSet::from([StatusId("Burn".into())]),
            particles: BTreeSet::from([ParticleId("impact_particle".into())]),
            skills: BTreeSet::new(),
        },
    );

    let state = wait_for_validation(&mut app);

    assert!(state.is_ready(), "expected ready state, got {state:?}");
    assert!(!state.is_failed());
    assert!(state.diagnostics().is_empty());
}

#[test]
fn broken_assets_set_failed_state_with_typed_diagnostics() {
    let mut app = build_asset_app(
        "test/animation_validation/broken_anim_graph.ron",
        "test/animation_validation/broken_clip.ron",
        AnimationValidationCatalogs::default(),
    );

    let state = wait_for_validation(&mut app);
    let diagnostics = state.blocking_diagnostics();

    assert!(!state.is_ready(), "broken assets must not report ready");
    assert!(state.is_failed(), "expected failed state, got {state:?}");
    assert!(has_diag(
        diagnostics,
        AnimationValidationCheck::GraphClipRange,
        AnimationValidationReason::MissingClipRange,
    ));
    assert!(has_diag(
        diagnostics,
        AnimationValidationCheck::TransitionTarget,
        AnimationValidationReason::UnknownNodeReference,
    ));
    assert!(has_diag(
        diagnostics,
        AnimationValidationCheck::CommandStatus,
        AnimationValidationReason::UnknownStatusReference,
    ));
    assert!(has_diag(
        diagnostics,
        AnimationValidationCheck::CommandParticle,
        AnimationValidationReason::UnknownParticleReference,
    ));
}
