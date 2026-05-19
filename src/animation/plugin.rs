use std::collections::BTreeMap;

use bevy::{ecs::system::Local, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;

use super::{
    validate_anim_graph, AnimGraph, AnimationValidationCatalogs, AnimationValidationCheck,
    AnimationValidationContext, AnimationValidationDiagnostic, AnimationValidationReason,
    AnimationValidationReport, AnimationValidationSeverity, AnimationValidationState, Clip,
    ClipMeta, ClipRange, FrameSize, SkillIdRef, StatusId,
};
use super::registry::{
    SkillGraphPaths, SkillGraphRegistry, StanceGraphPaths, StanceGraphRegistry,
    has_matching_asset_event, populate_graph_registries,
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

/// Default animation graph assets to load at boot (relative to `assets/`).
pub const DEFAULT_ANIM_GRAPH_PATHS: &[&str] = &[
    "digimon/agumon/anim_graph.ron",
    "digimon/renamon/anim_graph.ron",
];
/// Default clip geometry assets to load at boot (relative to `assets/`).
pub const DEFAULT_ANIM_CLIP_PATHS: &[&str] = &[
    "digimon/agumon/clip.ron",
    "digimon/renamon/clip.ron",
];

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

/// Data-driven list of clip geometry asset paths.
#[derive(Resource, Debug, Clone)]
pub struct AnimationClipPaths(pub Vec<String>);

impl Default for AnimationClipPaths {
    fn default() -> Self {
        Self(
            DEFAULT_ANIM_CLIP_PATHS
                .iter()
                .map(|path| (*path).to_string())
                .collect(),
        )
    }
}

/// Handles for every configured animation graph asset.
#[derive(Resource, Debug, Clone)]
pub struct AnimationGraphHandles(pub Vec<Handle<AnimGraph>>);

/// Handles for every configured clip geometry asset.
#[derive(Resource, Debug, Clone)]
pub struct AnimationClipHandles(pub Vec<Handle<Clip>>);

/// Load-state surface for animation graph assets.
#[derive(Resource, Debug, Default)]
pub struct AnimationGraphLoadState {
    /// Per-handle event observation state; index-aligned with [`AnimationGraphHandles`].
    pub loaded: Vec<bool>,
    /// True only after every configured asset has emitted a load/modify event and is readable
    /// through `Assets<AnimGraph>`.
    pub ready: bool,
}

/// Load-state surface for clip geometry assets.
#[derive(Resource, Debug, Default)]
pub struct AnimationClipLoadState {
    /// Per-handle event observation state; index-aligned with [`AnimationClipHandles`].
    pub loaded: Vec<bool>,
    /// True only after every configured asset has emitted a load/modify event and is readable
    /// through `Assets<Clip>`.
    pub ready: bool,
}

pub struct AnimationAssetPlugin;

impl Plugin for AnimationAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<AnimGraph>::new(&["ron"]),
            RonAssetPlugin::<Clip>::new(&["ron"]),
        ))
        .init_resource::<AnimationGraphPaths>()
        .init_resource::<AnimationClipPaths>()
        .init_resource::<AnimationValidationCatalogs>()
        .init_resource::<AnimationValidationState>()
        .init_resource::<SkillGraphPaths>()
        .init_resource::<StanceGraphPaths>()
        .init_resource::<SkillGraphRegistry>()
        .init_resource::<StanceGraphRegistry>()
        .add_systems(Startup, (load_animation_graphs, load_animation_clips))
        .add_systems(
            Update,
            (
                track_animation_graph_loads,
                track_animation_clip_loads,
                sync_validation_catalogs,
                validate_animation_assets,
                populate_graph_registries,
            ),
        );
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

fn load_animation_clips(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<AnimationClipPaths>,
) {
    let handles: Vec<Handle<Clip>> = paths
        .0
        .iter()
        .cloned()
        .map(|path| asset_server.load(path))
        .collect();

    commands.insert_resource(AnimationClipLoadState {
        loaded: vec![false; handles.len()],
        ready: false,
    });
    commands.insert_resource(AnimationClipHandles(handles));

    info!(
        "animation clip load requested: count={}, paths={:?}",
        paths.0.len(),
        paths.0
    );
}

fn track_animation_graph_loads(
    mut events: MessageReader<AssetEvent<AnimGraph>>,
    handles: Option<Res<AnimationGraphHandles>>,
    paths: Res<AnimationGraphPaths>,
    graphs: Res<Assets<AnimGraph>>,
    asset_server: Res<AssetServer>,
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

    // Mark handles that failed to load so they don't block global readiness.
    for (index, handle) in handles.0.iter().enumerate() {
        if !state.loaded[index] {
            if matches!(
                asset_server.load_state(handle.id()),
                bevy::asset::LoadState::Failed(_)
            ) {
                state.loaded[index] = true;
                let path = paths
                    .0
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("<unknown>");
                warn!("animation graph missing or failed: {path} — skipping");
            }
        }
    }

    if state.ready || !state.loaded.iter().all(|loaded| *loaded) {
        return;
    }

    // Ready when all non-failed handles have their asset loaded.
    let loaded_count = handles
        .0
        .iter()
        .filter(|handle| graphs.get(*handle).is_some())
        .count();
    state.ready = true;
    info!(
        "animation graphs ready: loaded={}, total={}",
        loaded_count,
        handles.0.len()
    );
}

fn track_animation_clip_loads(
    mut events: MessageReader<AssetEvent<Clip>>,
    handles: Option<Res<AnimationClipHandles>>,
    paths: Res<AnimationClipPaths>,
    clips: Res<Assets<Clip>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AnimationClipLoadState>,
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
                info!("animation clip {label}: {path}");
            }
        }
    }

    // Mark handles that failed to load so they don't block global readiness.
    for (index, handle) in handles.0.iter().enumerate() {
        if !state.loaded[index] {
            if matches!(
                asset_server.load_state(handle.id()),
                bevy::asset::LoadState::Failed(_)
            ) {
                state.loaded[index] = true;
                let path = paths
                    .0
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("<unknown>");
                warn!("animation clip missing or failed: {path} — skipping");
            }
        }
    }

    if state.ready || !state.loaded.iter().all(|loaded| *loaded) {
        return;
    }

    let loaded_count = handles
        .0
        .iter()
        .filter(|handle| clips.get(*handle).is_some())
        .count();
    state.ready = true;
    info!(
        "animation clips ready: loaded={}, total={}",
        loaded_count,
        handles.0.len()
    );
}

fn validate_animation_assets(
    graph_handles: Option<Res<AnimationGraphHandles>>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    graph_state: Res<AnimationGraphLoadState>,
    clip_state: Res<AnimationClipLoadState>,
    graphs: Res<Assets<AnimGraph>>,
    clips: Res<Assets<Clip>>,
    catalogs: Res<AnimationValidationCatalogs>,
    graph_paths: Res<AnimationGraphPaths>,
    clip_paths: Res<AnimationClipPaths>,
    mut validation_state: ResMut<AnimationValidationState>,
    mut graph_events: MessageReader<AssetEvent<AnimGraph>>,
    mut clip_events: MessageReader<AssetEvent<Clip>>,
    mut dirty: Local<bool>,
) {
    let Some(graph_handles) = graph_handles else {
        return;
    };
    let Some(clip_handles) = clip_handles else {
        return;
    };

    *dirty |= has_matching_asset_event(&mut graph_events, &graph_handles.0);
    *dirty |= has_matching_asset_event(&mut clip_events, &clip_handles.0);
    *dirty |= catalogs.is_changed();

    if !graph_state.ready || !clip_state.ready {
        return;
    }

    if !*dirty && validation_state.has_run() {
        return;
    }

    let mut diagnostics = Vec::new();

    for (graph_index, graph_handle) in graph_handles.0.iter().enumerate() {
        let Some(graph) = graphs.get(graph_handle) else {
            // Graph failed to load; skip it without blocking other graphs.
            continue;
        };

        let matching_clips: Vec<_> = clip_handles
            .0
            .iter()
            .enumerate()
            .filter_map(|(clip_index, clip_handle)| {
                clips
                    .get(clip_handle)
                    .filter(|clip| clip.ranges.contains_key(&graph.clip.0))
                    .map(|clip| (clip_index, clip))
            })
            .collect();

        let Some((clip_index, clip)) = matching_clips.first().copied() else {
            diagnostics.push(AnimationValidationDiagnostic {
                severity: AnimationValidationSeverity::Error,
                check: AnimationValidationCheck::GraphClipRange,
                reason: AnimationValidationReason::MissingClipRange,
                context: AnimationValidationContext {
                    clip_id: Some(graph.clip.clone()),
                    ..Default::default()
                },
                detail: format!(
                    "graph clip '{}' did not match any loaded clip asset ranges",
                    graph.clip.0
                ),
            });
            let graph_path = graph_paths
                .0
                .get(graph_index)
                .map(String::as_str)
                .unwrap_or("<unknown>");
            warn!(
                "animation validation failed to find clip asset for graph path={} clip_id={}",
                graph_path,
                graph.clip.0
            );
            let synthetic_clip = Clip {
                meta: ClipMeta {
                    frame_size: FrameSize { w: 0, h: 0 },
                    columns: 0,
                    rows: 0,
                    total_frames: u32::MAX,
                },
                ranges: BTreeMap::from([(
                    graph.clip.0.clone(),
                    ClipRange {
                        start: 0,
                        end: u32::MAX,
                    },
                )]),
            };
            diagnostics.extend(validate_anim_graph(graph, &synthetic_clip, &catalogs).diagnostics);
            continue;
        };

        diagnostics.extend(validate_anim_graph(graph, clip, &catalogs).diagnostics);

        let graph_path = graph_paths
            .0
            .get(graph_index)
            .map(String::as_str)
            .unwrap_or("<unknown>");
        let clip_path = clip_paths
            .0
            .get(clip_index)
            .map(String::as_str)
            .unwrap_or("<unknown>");
        info!(
            "animation validation checked graph path={} against clip path={} clip_id={}",
            graph_path,
            clip_path,
            graph.clip.0
        );
    }

    let next_state = AnimationValidationState::from_report(AnimationValidationReport {
        diagnostics,
    });

    match &next_state {
        AnimationValidationState::Pending => {}
        AnimationValidationState::Ready(report) => info!(
            "animation validation ready: graphs={}, clips={}, diagnostics={}",
            graph_handles.0.len(),
            clip_handles.0.len(),
            report.diagnostics.len()
        ),
        AnimationValidationState::Failed(report) => {
            warn!(
                "animation validation failed: graphs={}, clips={}, diagnostics={}",
                graph_handles.0.len(),
                clip_handles.0.len(),
                report.diagnostics.len()
            );
            for diag in &report.diagnostics {
                warn!("  - {:?}/{:?}: {}", diag.check, diag.reason, diag.detail);
            }
        }
    }

    *validation_state = next_state;
    *dirty = false;
}

/// Populates `AnimationValidationCatalogs` from the assembled `SkillBook` and the
/// `StatusEffectKind` enum vocabulary. Runs once after `DataReady` is present.
fn sync_validation_catalogs(
    skill_book_handle: Option<Res<SkillBookHandle>>,
    books: Option<Res<Assets<SkillBook>>>,
    mut catalogs: ResMut<AnimationValidationCatalogs>,
    mut ran: Local<bool>,
) {
    if *ran {
        return;
    }

    let Some(handle) = skill_book_handle else {
        return;
    };
    let Some(books) = books else {
        return;
    };

    let Some(book) = books.get(&handle.0) else {
        return;
    };

    // All StatusEffectKind variant names — kept in sync with the enum definition.
    let status_names = [
        "Heated", "Chilled", "Paralyzed", "Slowed", "Blessed", "Burn", "Shock",
    ];
    for name in status_names {
        catalogs.statuses.insert(StatusId(name.to_string()));
    }

    for skill in &book.0 {
        catalogs.skills.insert(SkillIdRef(skill.id.0.clone()));
    }

    *ran = true;
    info!(
        "animation validation catalogs synced: statuses={}, skills={}",
        catalogs.statuses.len(),
        catalogs.skills.len()
    );
}
