use std::collections::BTreeSet;

use bevy::prelude::Resource;

use crate::animation::{ClipId, NodeId, ParamKey, ParticleId, SkillIdRef, StatusId};

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
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
    GameplayCommandForbidden,
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
    GameplayCommandInAnimGraph,
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
        writeln!(
            f,
            "animation validation failed ({} diagnostic(s))",
            self.report.diagnostics.len()
        )?;
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

    pub fn has_run(&self) -> bool {
        !matches!(self, Self::Pending)
    }

    pub fn report(&self) -> Option<&AnimationValidationReport> {
        match self {
            Self::Pending => None,
            Self::Ready(report) | Self::Failed(report) => Some(report),
        }
    }

    pub fn diagnostics(&self) -> &[AnimationValidationDiagnostic] {
        self.report()
            .map(|report| report.diagnostics.as_slice())
            .unwrap_or(&[])
    }

    pub fn blocking_diagnostics(&self) -> &[AnimationValidationDiagnostic] {
        self.report()
            .map(|report| report.diagnostics.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}
