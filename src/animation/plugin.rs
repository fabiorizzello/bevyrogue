use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use super::AnimGraph;

/// Default animation graph assets to load at boot (relative to `assets/`).
pub const DEFAULT_ANIM_GRAPH_PATHS: &[&str] = &["digimon/agumon/anim_graph.ron"];

/// Data-driven list of animation graph asset paths.
#[derive(Resource, Debug, Clone)]
pub struct AnimationGraphPaths(pub Vec<String>);

impl Default for AnimationGraphPaths {
    fn default() -> Self {
        Self(
            DEFAULT_ANIM_GRAPH_PATHS
                .iter()
                .map(|path| (*path).to_string())
                .collect(),
        )
    }
}

/// Handles for every configured animation graph asset.
#[derive(Resource, Debug, Clone)]
pub struct AnimationGraphHandles(pub Vec<Handle<AnimGraph>>);

/// Load-state surface for animation graph assets.
#[derive(Resource, Debug, Default)]
pub struct AnimationGraphLoadState {
    /// Per-handle event observation state; index-aligned with [`AnimationGraphHandles`].
    pub loaded: Vec<bool>,
    /// True only after every configured asset has emitted a load/modify event and is readable
    /// through `Assets<AnimGraph>`.
    pub ready: bool,
}

pub struct AnimationAssetPlugin;

impl Plugin for AnimationAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<AnimGraph>::new(&["ron"]))
            .init_resource::<AnimationGraphPaths>()
            .add_systems(Startup, load_animation_graphs)
            .add_systems(Update, track_animation_graph_loads);
    }
}

fn load_animation_graphs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<AnimationGraphPaths>,
) {
    let handles: Vec<Handle<AnimGraph>> = paths
        .0
        .iter()
        .cloned()
        .map(|path| asset_server.load(path))
        .collect();

    commands.insert_resource(AnimationGraphLoadState {
        loaded: vec![false; handles.len()],
        ready: false,
    });
    commands.insert_resource(AnimationGraphHandles(handles));

    info!(
        "animation graph load requested: count={}, paths={:?}",
        paths.0.len(),
        paths.0
    );
}

fn track_animation_graph_loads(
    mut events: MessageReader<AssetEvent<AnimGraph>>,
    handles: Option<Res<AnimationGraphHandles>>,
    paths: Res<AnimationGraphPaths>,
    graphs: Res<Assets<AnimGraph>>,
    mut state: ResMut<AnimationGraphLoadState>,
) {
    let Some(handles) = handles else {
        return;
    };

    for event in events.read() {
        let (id, label) = match event {
            AssetEvent::LoadedWithDependencies { id } => (*id, "loaded"),
            AssetEvent::Modified { id } => (*id, "modified"),
            _ => continue,
        };

        for (index, handle) in handles.0.iter().enumerate() {
            if handle.id() == id {
                state.loaded[index] = true;
                let path = paths
                    .0
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("<unknown>");
                info!("animation graph {label}: {path}");
            }
        }
    }

    if state.ready || !state.loaded.iter().all(|loaded| *loaded) {
        return;
    }

    if handles.0.iter().all(|handle| graphs.get(handle).is_some()) {
        state.ready = true;
        info!("animation graphs ready: count={}", handles.0.len());
    }
}
