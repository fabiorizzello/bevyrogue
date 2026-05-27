use std::collections::HashMap;

use bevy::prelude::*;

use crate::warn_once::WarnOnce;

use super::AnimGraphId;
use super::anim_graph::{
    AnimEdge, AnimGraph, AnimNode, FrameRange, NodeId, Predicate, TransitionTarget,
};

/// Deterministic node id used by the runtime fallback graph when a requested
/// skill/stance graph is unavailable.
pub const MISSING_GRAPH_FALLBACK_NODE_ID: &str = "__missing_graph_instant__";

/// Structured classification of where a resolved graph came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedAnimGraphSource {
    Registry,
    InstantFallback,
}

/// Cloned graph snapshot handed to runtime callers.
///
/// Cloning here intentionally decouples in-flight players from subsequent asset
/// hot reloads. Existing players keep their current graph identity/state while
/// newly spawned players can resolve a fresh snapshot from the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAnimGraph {
    pub requested_id: AnimGraphId,
    pub source: ResolvedAnimGraphSource,
    graph: AnimGraph,
}

impl ResolvedAnimGraph {
    pub fn graph(&self) -> &AnimGraph {
        &self.graph
    }
}

/// Structured missing-graph diagnostic persisted for later inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGraphDiagnostic {
    pub registry: &'static str,
    pub requested_id: AnimGraphId,
    pub fallback_node: NodeId,
    pub message: String,
}

/// Inspectable lookup diagnostics for runtime graph resolution.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct AnimationGraphLookupDiagnostics {
    pub last_missing_skill_graph: Option<MissingGraphDiagnostic>,
    pub last_missing_stance_graph: Option<MissingGraphDiagnostic>,
    pub last_message: Option<String>,
}

impl AnimationGraphLookupDiagnostics {
    pub fn note_missing_skill_graph(&mut self, requested_id: &AnimGraphId) {
        self.note_missing("skill", requested_id);
    }

    pub fn note_missing_stance_graph(&mut self, requested_id: &AnimGraphId) {
        self.note_missing("stance", requested_id);
    }

    fn note_missing(&mut self, registry: &'static str, requested_id: &AnimGraphId) {
        let fallback_node = NodeId(MISSING_GRAPH_FALLBACK_NODE_ID.into());
        let message = format!(
            "animation graph runtime fallback: registry={registry} graph_id={} fallback=instant_exit entry={} reason=missing_or_unloaded",
            requested_id.0, fallback_node.0
        );
        let diagnostic = MissingGraphDiagnostic {
            registry,
            requested_id: requested_id.clone(),
            fallback_node,
            message: message.clone(),
        };
        match registry {
            "skill" => self.last_missing_skill_graph = Some(diagnostic),
            "stance" => self.last_missing_stance_graph = Some(diagnostic),
            _ => {}
        }
        self.last_message = Some(message);
    }
}

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

    pub fn resolve_snapshot(
        &self,
        id: &AnimGraphId,
        graphs: &Assets<AnimGraph>,
    ) -> Option<ResolvedAnimGraph> {
        let handle = self.resolve(id)?;
        let graph = graphs.get(handle)?.clone();
        Some(ResolvedAnimGraph {
            requested_id: id.clone(),
            source: ResolvedAnimGraphSource::Registry,
            graph,
        })
    }

    pub fn resolve_snapshot_or_instant_fallback(
        &self,
        id: &AnimGraphId,
        graphs: &Assets<AnimGraph>,
        diagnostics: &mut AnimationGraphLookupDiagnostics,
    ) -> ResolvedAnimGraph {
        self.resolve_snapshot(id, graphs).unwrap_or_else(|| {
            diagnostics.note_missing_skill_graph(id);
            ResolvedAnimGraph {
                requested_id: id.clone(),
                source: ResolvedAnimGraphSource::InstantFallback,
                graph: instant_fallback_graph(id),
            }
        })
    }
}

/// Maps `AnimGraphId` → `Handle<AnimGraph>` for graphs loaded from stance asset paths.
#[derive(Resource, Debug, Default)]
pub struct StanceGraphRegistry(pub HashMap<AnimGraphId, Handle<AnimGraph>>);

impl StanceGraphRegistry {
    pub fn resolve(&self, id: &AnimGraphId) -> Option<&Handle<AnimGraph>> {
        self.0.get(id)
    }

    pub fn resolve_snapshot(
        &self,
        id: &AnimGraphId,
        graphs: &Assets<AnimGraph>,
    ) -> Option<ResolvedAnimGraph> {
        let handle = self.resolve(id)?;
        let graph = graphs.get(handle)?.clone();
        Some(ResolvedAnimGraph {
            requested_id: id.clone(),
            source: ResolvedAnimGraphSource::Registry,
            graph,
        })
    }

    pub fn resolve_snapshot_or_instant_fallback(
        &self,
        id: &AnimGraphId,
        graphs: &Assets<AnimGraph>,
        diagnostics: &mut AnimationGraphLookupDiagnostics,
    ) -> ResolvedAnimGraph {
        self.resolve_snapshot(id, graphs).unwrap_or_else(|| {
            diagnostics.note_missing_stance_graph(id);
            ResolvedAnimGraph {
                requested_id: id.clone(),
                source: ResolvedAnimGraphSource::InstantFallback,
                graph: instant_fallback_graph(id),
            }
        })
    }
}

fn instant_fallback_graph(requested_id: &AnimGraphId) -> AnimGraph {
    let entry = NodeId(MISSING_GRAPH_FALLBACK_NODE_ID.into());
    AnimGraph {
        id: requested_id.clone(),
        clip: super::anim_graph::ClipId("missing_graph_fallback".into()),
        entry: entry.clone(),
        nodes: [(
            entry.clone(),
            AnimNode {
                frames: FrameRange(0, 0),
                on_enter: Vec::new(),
                cues: Vec::new(),
                modifier: None,
                reverse: false,
            },
        )]
        .into_iter()
        .collect(),
        transitions: vec![AnimEdge {
            from: entry,
            to: TransitionTarget::Exit,
            when: Predicate::Always,
            priority: None,
        }],
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
    // Asset ids we have already warned about, so an unbuildable graph logs once
    // rather than every frame it re-emits a Modified event.
    mut warned: Local<WarnOnce<AssetId<AnimGraph>>>,
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

        // Only graphs we configured are our responsibility; events for other
        // `AnimGraph` assets are silently ignored (not a spawn-miss).
        let handle = handles.0.iter().find(|h| h.id() == asset_id).cloned();
        let Some(handle) = handle else {
            continue;
        };

        let mut built = false;
        if let Some(p) = &path_str {
            if skill_paths.0.iter().any(|sp| sp == p) {
                skill_reg.0.insert(graph.id.clone(), handle);
                built = true;
            } else if stance_paths.0.iter().any(|sp| sp == p) {
                stance_reg.0.insert(graph.id.clone(), handle);
                built = true;
            }
        }

        // A configured graph loaded but matched neither path list, so no
        // registry entry can be built and its sprite will never resolve a
        // graph. Warn once per asset id to make this regression visible.
        if !built && warned.should_warn(asset_id) {
            warn!(
                "animation graph loaded but no registry entry could be built: \
                 graph_id={:?} path={:?} — matched neither SkillGraphPaths nor \
                 StanceGraphPaths; this sprite will not animate",
                graph.id, path_str
            );
        }
    }
}
