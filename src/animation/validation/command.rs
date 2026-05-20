use super::{AnimationValidationCatalogs, AnimationValidationDiagnostic};
use crate::animation::{
    AnimationValidationCheck, AnimationValidationContext, AnimationValidationReason,
    AnimationValidationSeverity, ClipId, Command, NodeId, ParamRef, ParticleId, StatusId,
};

pub(crate) fn validate_command(
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
