use std::collections::BTreeSet;

use bevy::prelude::Resource;

use super::{
    AnimGraph, Clip, ClipId, Command, KernelEventFilter, NodeId, ParamKey, ParamRef, ParticleId,
    Predicate, SkillIdRef, StatusId, TransitionTarget,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimationValidationCatalogs {
    pub params: BTreeSet<ParamKey>,
    pub statuses: BTreeSet<StatusId>,
    pub particles: BTreeSet<ParticleId>,
    pub skills: BTreeSet<SkillIdRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationValidationSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationValidationCheck {
    GraphClipRange,
    EntryNode,
    NodeFrames,
    TransitionSource,
    TransitionTarget,
    PredicateUnlock,
    PredicateStatus,
    CommandParam,
    CommandStatus,
    CommandParticle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationValidationReason {
    MissingClipRange,
    MissingEntryNode,
    FrameRangeOutOfOrder,
    FrameOutsideClipTotal,
    FrameOutsideNamedClipRange,
    UnknownNodeReference,
    UnknownParamReference,
    UnknownStatusReference,
    UnknownParticleReference,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimationValidationContext {
    pub clip_id: Option<ClipId>,
    pub node_id: Option<NodeId>,
    pub transition_index: Option<usize>,
    pub command_index: Option<usize>,
    pub command_field: Option<String>,
    pub predicate_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationValidationDiagnostic {
    pub severity: AnimationValidationSeverity,
    pub check: AnimationValidationCheck,
    pub reason: AnimationValidationReason,
    pub context: AnimationValidationContext,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimationValidationReport {
    pub diagnostics: Vec<AnimationValidationDiagnostic>,
}

impl AnimationValidationReport {
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diag| diag.severity == AnimationValidationSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationValidationFailure {
    pub report: AnimationValidationReport,
}

impl std::fmt::Display for AnimationValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "animation validation failed ({} diagnostic(s))", self.report.diagnostics.len())?;
        for diagnostic in &self.report.diagnostics {
            writeln!(
                f,
                "- {:?}/{:?}: {}",
                diagnostic.check, diagnostic.reason, diagnostic.detail
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for AnimationValidationFailure {}

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub enum AnimationValidationState {
    #[default]
    Pending,
    Ready(AnimationValidationReport),
    Failed(AnimationValidationReport),
}

impl AnimationValidationState {
    pub fn from_report(report: AnimationValidationReport) -> Self {
        if report.has_errors() {
            Self::Failed(report)
        } else {
            Self::Ready(report)
        }
    }

    pub fn report(&self) -> Option<&AnimationValidationReport> {
        match self {
            Self::Pending => None,
            Self::Ready(report) | Self::Failed(report) => Some(report),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

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
            detail: format!(
                "graph clip '{}' is missing from Clip.ranges",
                graph.clip.0
            ),
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
            validate_command(
                command,
                catalogs,
                &clip_id,
                node_id,
                command_index,
                &mut diagnostics,
            );
        }
    }

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
            &clip_id,
            transition_index,
            "when",
            &mut diagnostics,
        );
    }

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

fn validate_predicate(
    predicate: &Predicate,
    graph: &AnimGraph,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    transition_index: usize,
    path: &str,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    match predicate {
        Predicate::Unlock(node_id) => {
            if !graph.nodes.contains_key(node_id) {
                diagnostics.push(AnimationValidationDiagnostic {
                    severity: AnimationValidationSeverity::Error,
                    check: AnimationValidationCheck::PredicateUnlock,
                    reason: AnimationValidationReason::UnknownNodeReference,
                    context: AnimationValidationContext {
                        clip_id: Some(clip_id.clone()),
                        node_id: Some(node_id.clone()),
                        transition_index: Some(transition_index),
                        predicate_path: Some(path.to_string()),
                        ..Default::default()
                    },
                    detail: format!(
                        "transition[{transition_index}] predicate {path} unlocks missing node '{}'",
                        node_id.0
                    ),
                });
            }
        }
        Predicate::And(left, right) => {
            validate_predicate(
                left,
                graph,
                catalogs,
                clip_id,
                transition_index,
                &format!("{path}.and_left"),
                diagnostics,
            );
            validate_predicate(
                right,
                graph,
                catalogs,
                clip_id,
                transition_index,
                &format!("{path}.and_right"),
                diagnostics,
            );
        }
        Predicate::Or(left, right) => {
            validate_predicate(
                left,
                graph,
                catalogs,
                clip_id,
                transition_index,
                &format!("{path}.or_left"),
                diagnostics,
            );
            validate_predicate(
                right,
                graph,
                catalogs,
                clip_id,
                transition_index,
                &format!("{path}.or_right"),
                diagnostics,
            );
        }
        Predicate::Not(inner) => validate_predicate(
            inner,
            graph,
            catalogs,
            clip_id,
            transition_index,
            &format!("{path}.not"),
            diagnostics,
        ),
        Predicate::KernelEvent(KernelEventFilter::StatusApplied { status }) => {
            if !catalogs.statuses.contains(status) {
                diagnostics.push(AnimationValidationDiagnostic {
                    severity: AnimationValidationSeverity::Error,
                    check: AnimationValidationCheck::PredicateStatus,
                    reason: AnimationValidationReason::UnknownStatusReference,
                    context: AnimationValidationContext {
                        clip_id: Some(clip_id.clone()),
                        transition_index: Some(transition_index),
                        predicate_path: Some(path.to_string()),
                        ..Default::default()
                    },
                    detail: format!(
                        "transition[{transition_index}] predicate {path} references unknown status '{}'",
                        status.0
                    ),
                });
            }
        }
        Predicate::TimeInNode
        | Predicate::KernelEvent(_)
        | Predicate::UserInput(_)
        | Predicate::Always => {}
    }
}

fn validate_command(
    command: &Command,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    node_id: &NodeId,
    command_index: usize,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    match command {
        Command::EmitDamage {
            hits,
            mul,
            status,
            chance_pct,
            duration,
            ..
        } => {
            validate_param_ref(
                hits,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "hits",
                diagnostics,
            );
            validate_param_ref(
                mul,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "mul",
                diagnostics,
            );
            if let Some(status) = status {
                validate_status_ref(
                    status,
                    catalogs,
                    clip_id,
                    node_id,
                    command_index,
                    "status",
                    diagnostics,
                );
            }
            if let Some(chance_pct) = chance_pct {
                validate_param_ref(
                    chance_pct,
                    catalogs,
                    clip_id,
                    node_id,
                    command_index,
                    "chance_pct",
                    diagnostics,
                );
            }
            if let Some(duration) = duration {
                validate_param_ref(
                    duration,
                    catalogs,
                    clip_id,
                    node_id,
                    command_index,
                    "duration",
                    diagnostics,
                );
            }
        }
        Command::EmitStatus {
            id,
            duration,
            chance_pct,
            ..
        } => {
            validate_status_ref(
                id,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "id",
                diagnostics,
            );
            validate_param_ref(
                duration,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "duration",
                diagnostics,
            );
            if let Some(chance_pct) = chance_pct {
                validate_param_ref(
                    chance_pct,
                    catalogs,
                    clip_id,
                    node_id,
                    command_index,
                    "chance_pct",
                    diagnostics,
                );
            }
        }
        Command::SpawnParticle { name, .. } => validate_particle_ref(
            name,
            catalogs,
            clip_id,
            node_id,
            command_index,
            "name",
            diagnostics,
        ),
        Command::Shake {
            intensity,
            duration_ms,
        } => {
            validate_param_ref(
                intensity,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "intensity",
                diagnostics,
            );
            validate_param_ref(
                duration_ms,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "duration_ms",
                diagnostics,
            );
        }
        Command::StartQte { window, .. } => validate_param_ref(
            window,
            catalogs,
            clip_id,
            node_id,
            command_index,
            "window",
            diagnostics,
        ),
        Command::EmitHeal { amount, .. } | Command::EmitSpGrant { amount, .. } => {
            validate_param_ref(
                amount,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "amount",
                diagnostics,
            );
        }
        Command::EmitCleanse { count, .. } => validate_param_ref(
            count,
            catalogs,
            clip_id,
            node_id,
            command_index,
            "count",
            diagnostics,
        ),
        Command::AdvanceTurn { pct, .. } | Command::DelayTurn { pct, .. } => validate_param_ref(
            pct,
            catalogs,
            clip_id,
            node_id,
            command_index,
            "pct",
            diagnostics,
        ),
        Command::ApplyBuff { id, duration, .. } => {
            validate_status_ref(
                id,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "id",
                diagnostics,
            );
            validate_param_ref(
                duration,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "duration",
                diagnostics,
            );
        }
        Command::Reposition { .. } => {}
        Command::BlockReaction {
            damage_mult,
            duration,
            ..
        } => {
            validate_param_ref(
                damage_mult,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "damage_mult",
                diagnostics,
            );
            validate_param_ref(
                duration,
                catalogs,
                clip_id,
                node_id,
                command_index,
                "duration",
                diagnostics,
            );
        }
    }
}

fn validate_param_ref(
    param: &ParamRef,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    node_id: &NodeId,
    command_index: usize,
    field: &str,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    let key = match param {
        ParamRef::Static(key) | ParamRef::Snapshot(key) | ParamRef::BlueprintState(key) => key,
        ParamRef::Literal(_) => return,
    };

    if !catalogs.params.contains(key) {
        diagnostics.push(AnimationValidationDiagnostic {
            severity: AnimationValidationSeverity::Error,
            check: AnimationValidationCheck::CommandParam,
            reason: AnimationValidationReason::UnknownParamReference,
            context: AnimationValidationContext {
                clip_id: Some(clip_id.clone()),
                node_id: Some(node_id.clone()),
                command_index: Some(command_index),
                command_field: Some(field.to_string()),
                ..Default::default()
            },
            detail: format!(
                "node '{}' command[{command_index}] field '{}' references unknown param '{}'",
                node_id.0, field, key.0
            ),
        });
    }
}

fn validate_status_ref(
    status: &StatusId,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    node_id: &NodeId,
    command_index: usize,
    field: &str,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    if !catalogs.statuses.contains(status) {
        diagnostics.push(AnimationValidationDiagnostic {
            severity: AnimationValidationSeverity::Error,
            check: AnimationValidationCheck::CommandStatus,
            reason: AnimationValidationReason::UnknownStatusReference,
            context: AnimationValidationContext {
                clip_id: Some(clip_id.clone()),
                node_id: Some(node_id.clone()),
                command_index: Some(command_index),
                command_field: Some(field.to_string()),
                ..Default::default()
            },
            detail: format!(
                "node '{}' command[{command_index}] field '{}' references unknown status '{}'",
                node_id.0, field, status.0
            ),
        });
    }
}

fn validate_particle_ref(
    particle: &ParticleId,
    catalogs: &AnimationValidationCatalogs,
    clip_id: &ClipId,
    node_id: &NodeId,
    command_index: usize,
    field: &str,
    diagnostics: &mut Vec<AnimationValidationDiagnostic>,
) {
    if !catalogs.particles.contains(particle) {
        diagnostics.push(AnimationValidationDiagnostic {
            severity: AnimationValidationSeverity::Error,
            check: AnimationValidationCheck::CommandParticle,
            reason: AnimationValidationReason::UnknownParticleReference,
            context: AnimationValidationContext {
                clip_id: Some(clip_id.clone()),
                node_id: Some(node_id.clone()),
                command_index: Some(command_index),
                command_field: Some(field.to_string()),
                ..Default::default()
            },
            detail: format!(
                "node '{}' command[{command_index}] field '{}' references unknown particle '{}'",
                node_id.0, field, particle.0
            ),
        });
    }
}
