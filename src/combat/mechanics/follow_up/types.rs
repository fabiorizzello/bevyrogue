use bevy::prelude::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::FollowUpTrigger,
    types::{SkillId, UnitId},
};

/// Distinguishes standard follow-ups from Form Identity reactive actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FollowUpOriginKind {
    #[default]
    FollowUp,
    FormIdentity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FollowUpIntent {
    pub attacker: UnitId,
    pub skill_id: SkillId,
    pub target: UnitId,
    pub origin: CombatEvent,
    pub origin_kind: FollowUpOriginKind,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FollowUpTrace {
    pub follower: UnitId,
    pub trigger: FollowUpTrigger,
    pub action: SkillId,
    pub origin_kind: CombatEventKind,
    pub origin_source: UnitId,
    pub origin_target: UnitId,
    pub follow_up_target: Option<UnitId>,
    pub decision: FollowUpDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowUpDecision {
    Scheduled,
    Suppressed { reason: FollowUpSkipReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUpSkipReason {
    TriggerMismatch,
    WrongTeam,
    FollowerKo,
    FollowerStunned,
    MissingTarget,
}
