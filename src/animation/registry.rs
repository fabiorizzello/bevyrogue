use std::collections::HashMap;

use bevy::prelude::*;

use super::AnimGraphId;
use super::anim_graph::AnimGraph;

/// Maps `AnimGraphId` → `Handle<AnimGraph>` for graphs loaded from skill asset paths.
///
/// Populated by `populate_graph_registries` once each handle's asset resolves.
/// Lookup is a plain map get — no if-else dispatch.
#[derive(Resource, Debug, Default)]
pub struct SkillGraphRegistry(pub HashMap<AnimGraphId, Handle<AnimGraph>>);

impl SkillGraphRegistry {
    pub fn resolve(&self, id: &AnimGraphId) -> Option<&Handle<AnimGraph>> {
        self.0.get(id)
    }
}

/// Maps `AnimGraphId` → `Handle<AnimGraph>` for graphs loaded from stance asset paths.
#[derive(Resource, Debug, Default)]
pub struct StanceGraphRegistry(pub HashMap<AnimGraphId, Handle<AnimGraph>>);

impl StanceGraphRegistry {
    pub fn resolve(&self, id: &AnimGraphId) -> Option<&Handle<AnimGraph>> {
        self.0.get(id)
    }
}

/// Default stance graph asset paths loaded at boot (relative to `assets/`).
pub const DEFAULT_ANIM_STANCE_PATHS: &[&str] = &["digimon/agumon/stance.ron"];

/// Data-driven list of stance graph asset paths.
#[derive(Resource, Debug, Clone)]
pub struct AnimationStancePaths(pub Vec<String>);

impl Default for AnimationStancePaths {
    fn default() -> Self {
        Self(
            DEFAULT_ANIM_STANCE_PATHS
                .iter()
                .map(|p| p.to_string())
                .collect(),
        )
    }
}

/// Asset paths whose loaded graphs are classified as skill graphs.
#[derive(Resource, Debug, Default, Clone)]
pub struct SkillGraphPaths(pub Vec<String>);

/// Asset paths whose loaded graphs are classified as stance graphs.
#[derive(Resource, Debug, Default, Clone)]
pub struct StanceGraphPaths(pub Vec<String>);

/// Returns true if any relevant asset event (loaded or modified) matches one of the given handles.
pub fn has_matching_asset_event<T: Asset>(
    events: &mut MessageReader<AssetEvent<T>>,
    handles: &[Handle<T>],
) -> bool {
    events.read().any(|event| {
        let id = match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => *id,
            _ => return false,
        };
        handles.iter().any(|handle| handle.id() == id)
    })
}

/// Inserts entries into the appropriate registry once each `AnimGraph` asset resolves.
pub fn populate_graph_registries(
    mut events: MessageReader<AssetEvent<AnimGraph>>,
    graphs: Res<Assets<AnimGraph>>,
    handles: Option<Res<super::plugin::AnimationGraphHandles>>,
    skill_paths: Res<SkillGraphPaths>,
    stance_paths: Res<StanceGraphPaths>,
    asset_server: Res<AssetServer>,
    mut skill_reg: ResMut<SkillGraphRegistry>,
    mut stance_reg: ResMut<StanceGraphRegistry>,
) {
    let Some(handles) = handles else {
        return;
    };

    for event in events.read() {
        let asset_id = match event {
            AssetEvent::LoadedWithDependencies { id } => *id,
            AssetEvent::Modified { id } => *id,
            _ => continue,
        };

        let Some(graph) = graphs.get(asset_id) else {
            continue;
        };

        let path = asset_server.get_path(asset_id);
        let path_str = path
            .as_ref()
            .map(|p| p.path().to_string_lossy().into_owned());

        let handle = handles.0.iter().find(|h| h.id() == asset_id).cloned();
        let Some(handle) = handle else {
            continue;
        };

        if let Some(p) = &path_str {
            if skill_paths.0.iter().any(|sp| sp == p) {
                skill_reg.0.insert(graph.id.clone(), handle);
                return;
            }
            if stance_paths.0.iter().any(|sp| sp == p) {
                stance_reg.0.insert(graph.id.clone(), handle);
                return;
            }
        }
    }
}
