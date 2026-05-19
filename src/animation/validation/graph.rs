use super::{
    command::validate_command, predicate::validate_predicate, AnimationValidationCatalogs,
    AnimationValidationDiagnostic, AnimationValidationFailure, AnimationValidationReport,
};
use crate::animation::{
    AnimGraph, AnimationValidationCheck, AnimationValidationContext,
    AnimationValidationReason, AnimationValidationSeverity, Clip, ClipId, ClipRange,
    TransitionTarget,
};

pub fn validate_anim_graph(
    graph: &AnimGraph,
    clip: &Clip,
    catalogs: &AnimationValidationCatalogs,
) -> AnimationValidationReport {
    let mut diagnostics = Vec::new();
    let clip_id = graph.clip.clone();
    let clip_range = clip.ranges.get(&graph.clip.0).copied();

    if clip_range.is_none() {
        diagnostics.push(AnimationValidationDiagnostic {
            severity: AnimationValidationSeverity::Error,
            check: AnimationValidationCheck::GraphClipRange,
            reason: AnimationValidationReason::MissingClipRange,
            context: AnimationValidationContext {
                clip_id: Some(clip_id.clone()),
                ..Default::default()
            },
            detail: format!("graph clip '{}' is missing from Clip.ranges", graph.clip.0),
        });
    }

    if !graph.nodes.contains_key(&graph.entry) {
        diagnostics.push(AnimationValidationDiagnostic {
            severity: AnimationValidationSeverity::Error,
            check: AnimationValidationCheck::EntryNode,
            reason: AnimationValidationReason::MissingEntryNode,
            context: AnimationValidationContext {
                clip_id: Some(clip_id.clone()),
                node_id: Some(graph.entry.clone()),
                ..Default::default()
            },
            detail: format!("entry node '{}' is missing from graph.nodes", graph.entry.0),
        });
    }

    validate_graph_nodes(graph, clip, catalogs, &clip_id, clip_range, &mut diagnostics);
    validate_graph_transitions(graph, catalogs, &clip_id, &mut diagnostics);

    AnimationValidationReport { diagnostics }
}

pub fn validate_anim_graph_blocking(
    graph: &AnimGraph,
    clip: &Clip,
    catalogs: &AnimationValidationCatalogs,
) -> Result<(), AnimationValidationFailure> {
    let report = validate_anim_graph(graph, clip, catalogs);
    if report.has_errors() {
        Err(AnimationValidationFailure { report })
    } else {
        Ok(())
    }
}

fn validate_graph_nodes(
    graph: &AnimGraph,
    clip: &Clip,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    clip_range: Option<ClipRange>,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    for (node_id, node) in &graph.nodes {
        let start = node.frames.start();
        let end = node.frames.end();

        if start > end {
            diagnostics.push(AnimationValidationDiagnostic {
                severity: AnimationValidationSeverity::Error,
                check: AnimationValidationCheck::NodeFrames,
                reason: AnimationValidationReason::FrameRangeOutOfOrder,
                context: AnimationValidationContext {
                    clip_id: Some(clip_id.clone()),
                    node_id: Some(node_id.clone()),
                    ..Default::default()
                },
                detail: format!(
                    "node '{}' frames are out of order: start={} end={}",
                    node_id.0, start, end
                ),
            });
        }

        if start >= clip.meta.total_frames || end >= clip.meta.total_frames {
            diagnostics.push(AnimationValidationDiagnostic {
                severity: AnimationValidationSeverity::Error,
                check: AnimationValidationCheck::NodeFrames,
                reason: AnimationValidationReason::FrameOutsideClipTotal,
                context: AnimationValidationContext {
                    clip_id: Some(clip_id.clone()),
                    node_id: Some(node_id.clone()),
                    ..Default::default()
                },
                detail: format!(
                    "node '{}' frames [{start}, {end}] exceed clip total_frames={} (max index {})",
                    node_id.0,
                    clip.meta.total_frames,
                    clip.meta.total_frames.saturating_sub(1)
                ),
            });
        }

        if let Some(named_range) = clip_range {
            if !named_range.contains(start) || !named_range.contains(end) {
                diagnostics.push(AnimationValidationDiagnostic {
                    severity: AnimationValidationSeverity::Error,
                    check: AnimationValidationCheck::NodeFrames,
                    reason: AnimationValidationReason::FrameOutsideNamedClipRange,
                    context: AnimationValidationContext {
                        clip_id: Some(clip_id.clone()),
                        node_id: Some(node_id.clone()),
                        ..Default::default()
                    },
                    detail: format!(
                        "node '{}' frames [{start}, {end}] fall outside clip range '{}'=[{}, {}]",
                        node_id.0, graph.clip.0, named_range.start, named_range.end
                    ),
                });
            }
        }

        for (command_index, command) in node.on_enter.iter().enumerate() {
            validate_command(command, catalogs, clip_id, node_id, command_index, diagnostics);
        }
    }
}

fn validate_graph_transitions(
    graph: &AnimGraph,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    for (transition_index, edge) in graph.transitions.iter().enumerate() {
        if !graph.nodes.contains_key(&edge.from) {
            diagnostics.push(AnimationValidationDiagnostic {
                severity: AnimationValidationSeverity::Error,
                check: AnimationValidationCheck::TransitionSource,
                reason: AnimationValidationReason::UnknownNodeReference,
                context: AnimationValidationContext {
                    clip_id: Some(clip_id.clone()),
                    node_id: Some(edge.from.clone()),
                    transition_index: Some(transition_index),
                    ..Default::default()
                },
                detail: format!(
                    "transition[{transition_index}] references missing source node '{}'",
                    edge.from.0
                ),
            });
        }

        if let TransitionTarget::Node(target) = &edge.to {
            if !graph.nodes.contains_key(target) {
                diagnostics.push(AnimationValidationDiagnostic {
                    severity: AnimationValidationSeverity::Error,
                    check: AnimationValidationCheck::TransitionTarget,
                    reason: AnimationValidationReason::UnknownNodeReference,
                    context: AnimationValidationContext {
                        clip_id: Some(clip_id.clone()),
                        node_id: Some(target.clone()),
                        transition_index: Some(transition_index),
                        ..Default::default()
                    },
                    detail: format!(
                        "transition[{transition_index}] references missing target node '{}'",
                        target.0
                    ),
                });
            }
        }

        validate_predicate(
            &edge.when,
            graph,
            catalogs,
            clip_id,
            transition_index,
            "when",
            diagnostics,
        );
    }
}
