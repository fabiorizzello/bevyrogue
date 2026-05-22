use std::collections::BTreeMap;

use bevy::{ecs::system::Local, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;

use super::registry::{
    AnimationGraphLookupDiagnostics, AnimationStancePaths, SkillGraphPaths, SkillGraphRegistry,
    StanceGraphPaths, StanceGraphRegistry, has_matching_asset_event, populate_graph_registries,
};
use super::{
    AnimGraph, AnimationValidationCatalogs, AnimationValidationCheck, AnimationValidationContext,
    AnimationValidationDiagnostic, AnimationValidationReason, AnimationValidationReport,
    AnimationValidationSeverity, AnimationValidationState, Clip, ClipMeta, ClipRange, FrameSize,
    SkillIdRef, StatusId, validate_anim_graph,
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

/// Default animation graph assets to load at boot (relative to `assets/`).
pub const DEFAULT_ANIM_GRAPH_PATHS: &[&str] = &[
    "digimon/agumon/anim_graph.ron",
    "digimon/renamon/anim_graph.ron",
];
/// Default clip geometry assets to load at boot (relative to `assets/`).
pub const DEFAULT_ANIM_CLIP_PATHS: &[&str] =
    &["digimon/agumon/clip.ron", "digimon/renamon/clip.ron"];

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
    /// Configured asset paths that failed during boot-time load and therefore
    /// left the canonical registry surface incomplete.
    pub failed_paths: Vec<String>,
    /// True only after every configured asset has emitted a load/modify event and is readable
    /// through `Assets<AnimGraph>`.
    pub ready: bool,
}

impl AnimationGraphLoadState {
    pub fn has_boot_failures(&self) -> bool {
        !self.failed_paths.is_empty()
    }

    pub fn failed_paths(&self) -> &[String] {
        &self.failed_paths
    }
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
        .init_resource::<AnimationStancePaths>()
        .init_resource::<SkillGraphPaths>()
        .init_resource::<StanceGraphPaths>()
        .init_resource::<SkillGraphRegistry>()
        .init_resource::<StanceGraphRegistry>()
        .init_resource::<AnimationGraphLookupDiagnostics>()
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
    stance_paths: Res<AnimationStancePaths>,
) {
    commands.insert_resource(SkillGraphPaths(paths.0.clone()));
    commands.insert_resource(StanceGraphPaths(stance_paths.0.clone()));

    let all_paths: Vec<String> = paths
        .0
        .iter()
        .chain(stance_paths.0.iter())
        .cloned()
        .collect();
    let handles: Vec<Handle<AnimGraph>> = all_paths
        .iter()
        .map(|p| asset_server.load(p.clone()))
        .collect();

    commands.insert_resource(AnimationGraphLoadState {
        loaded: vec![false; handles.len()],
        failed_paths: Vec::new(),
        ready: false,
    });
    commands.insert_resource(AnimationGraphHandles(handles));

    info!(
        "animation graph load requested: skill={}, stance={}, total={}",
        paths.0.len(),
        stance_paths.0.len(),
        all_paths.len()
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
                if !state.failed_paths.iter().any(|failed| failed == path) {
                    state.failed_paths.push(path.to_string());
                }
                warn!("animation graph missing or failed: {path} — canonical boot registry entry unavailable");
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
        "animation graphs ready: loaded={}, total={}, failed_paths={}",
        loaded_count,
        handles.0.len(),
        state.failed_paths.len()
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

/// Returns the asset directory that owns `path` — everything before the final
/// `/` — or `None` for a bare filename. Bevy asset paths are always
/// `/`-separated regardless of host platform, so we split on `/` directly
/// rather than going through `std::path` (which would use `\` on Windows).
fn asset_owner_dir(path: &str) -> Option<&str> {
    path.rfind('/').map(|idx| &path[..idx])
}

/// Pairs an animation graph to the clip living in the *same asset directory*
/// (i.e. the same Digimon), returning that clip's index into `clip_paths`.
///
/// Pairing by owning directory — not by range name — is what keeps
/// `renamon/anim_graph.ron` validated against `renamon/clip.ron`. Several
/// Digimon expose an identically named `"skill"` range, so the previous
/// "first clip that contains the range" rule silently validated every later
/// Digimon's graph against the first-loaded clip's frame numbers.
fn pair_graph_to_clip(graph_path: &str, clip_paths: &[String]) -> Option<usize> {
    let graph_owner = asset_owner_dir(graph_path)?;
    clip_paths
        .iter()
        .position(|clip_path| asset_owner_dir(clip_path) == Some(graph_owner))
}

fn validate_animation_assets(
    graph_handles: Option<Res<AnimationGraphHandles>>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    graph_state: Res<AnimationGraphLoadState>,
    clip_state: Res<AnimationClipLoadState>,
    graphs: Res<Assets<AnimGraph>>,
    clips: Res<Assets<Clip>>,
    catalogs: Res<AnimationValidationCatalogs>,
    asset_server: Res<AssetServer>,
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

    for graph_handle in graph_handles.0.iter() {
        let Some(graph) = graphs.get(graph_handle) else {
            // Graph failed to load; skip it without blocking other graphs.
            continue;
        };

        // `graph_handles` interleaves skill and stance graphs, so it is NOT
        // index-aligned with `AnimationGraphPaths`. The asset server is the only
        // reliable per-handle path source here (same approach as
        // `populate_graph_registries`).
        let graph_path = asset_server
            .get_path(graph_handle.id())
            .map(|path| path.path().to_string_lossy().into_owned());
        let graph_path_ref = graph_path.as_deref().unwrap_or("<unknown>");

        // Pair the graph to the clip in its own directory, then resolve that
        // clip asset. `clip_paths` is index-aligned with `clip_handles`.
        let paired = graph_path
            .as_deref()
            .and_then(|path| pair_graph_to_clip(path, &clip_paths.0))
            .and_then(|clip_index| {
                clip_handles
                    .0
                    .get(clip_index)
                    .and_then(|handle| clips.get(handle))
                    .map(|clip| (clip_index, clip))
            });

        let Some((clip_index, clip)) = paired else {
            diagnostics.push(AnimationValidationDiagnostic {
                severity: AnimationValidationSeverity::Error,
                check: AnimationValidationCheck::GraphClipRange,
                reason: AnimationValidationReason::MissingClipRange,
                context: AnimationValidationContext {
                    clip_id: Some(graph.clip.clone()),
                    ..Default::default()
                },
                detail: format!(
                    "graph clip '{}' has no clip asset in its own directory (graph={graph_path_ref})",
                    graph.clip.0
                ),
            });
            warn!(
                "animation validation found no clip asset alongside graph path={} clip_id={}",
                graph_path_ref, graph.clip.0
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

        let clip_path = clip_paths
            .0
            .get(clip_index)
            .map(String::as_str)
            .unwrap_or("<unknown>");
        info!(
            "animation validation checked graph path={} against clip path={} clip_id={}",
            graph_path_ref, clip_path, graph.clip.0
        );
    }

    let next_state =
        AnimationValidationState::from_report(AnimationValidationReport { diagnostics });

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
        "Heated",
        "Chilled",
        "Paralyzed",
        "Slowed",
        "Blessed",
        "Burn",
        "Shock",
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

#[cfg(test)]
mod tests {
    use super::{asset_owner_dir, pair_graph_to_clip};

    #[test]
    fn asset_owner_dir_returns_parent_or_none() {
        assert_eq!(
            asset_owner_dir("digimon/renamon/anim_graph.ron"),
            Some("digimon/renamon")
        );
        assert_eq!(asset_owner_dir("clip.ron"), None);
    }

    #[test]
    fn pair_graph_to_clip_matches_by_owning_directory_not_range_name() {
        // Order mirrors DEFAULT_ANIM_CLIP_PATHS: Agumon is index 0.
        let clip_paths = vec![
            "digimon/agumon/clip.ron".to_string(),
            "digimon/renamon/clip.ron".to_string(),
        ];

        // Renamon's graph must pair with Renamon's clip (index 1), even though
        // Agumon's clip is loaded first and also exposes a "skill" range.
        assert_eq!(
            pair_graph_to_clip("digimon/renamon/anim_graph.ron", &clip_paths),
            Some(1)
        );
        // Both the skill graph and the stance graph in a directory pair to the
        // single clip in that same directory.
        assert_eq!(
            pair_graph_to_clip("digimon/agumon/anim_graph.ron", &clip_paths),
            Some(0)
        );
        assert_eq!(
            pair_graph_to_clip("digimon/agumon/stance.ron", &clip_paths),
            Some(0)
        );
        // A graph with no clip alongside it pairs with nothing.
        assert_eq!(
            pair_graph_to_clip("digimon/gabumon/anim_graph.ron", &clip_paths),
            None
        );
    }
}
