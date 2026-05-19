use super::{AnimationValidationCatalogs, AnimationValidationDiagnostic};
use crate::animation::{
    AnimGraph, AnimationValidationCheck, AnimationValidationContext, AnimationValidationReason,
    AnimationValidationSeverity, ClipId, KernelEventFilter, Predicate,
};

pub(crate) fn validate_predicate(
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
        | Predicate::Always
        | Predicate::KernelCue => {}
    }
}
