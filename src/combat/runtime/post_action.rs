use crate::combat::{
    events::CombatKernelTransition,
    runtime::{intent::CastId, intent::Intent, registry::ExtRegistries},
    status_effect::StatusEffectKind,
    team::Team,
    types::{SkillId, UnitId},
};

/// Stable snapshot of the `UnitDied` payload available to post-action reactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostActionUnitDied {
    pub status_remaining: Vec<StatusEffectKind>,
    pub heated_remaining: u32,
}

impl PostActionUnitDied {
    pub fn new(status_remaining: Vec<StatusEffectKind>, heated_remaining: u32) -> Self {
        Self {
            status_remaining,
            heated_remaining,
        }
    }
}

/// Minimal roster presence snapshot captured around a resolved action.
///
/// Keeps the post-action seam generic while still giving blueprints enough
/// deterministic context to reason about relative targets (for example,
/// same-team slot adjacency) without reaching into Bevy world internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostActionUnitSnapshot {
    pub unit_id: UnitId,
    pub team: Team,
    pub slot_index: Option<u8>,
    pub hp_current: i32,
    pub hp_max: i32,
    pub alive: bool,
}

impl PostActionUnitSnapshot {
    pub fn new(
        unit_id: UnitId,
        team: Team,
        slot_index: Option<u8>,
        hp_current: i32,
        hp_max: i32,
        alive: bool,
    ) -> Self {
        Self {
            unit_id,
            team,
            slot_index,
            hp_current,
            hp_max,
            alive,
        }
    }
}

/// Owner-neutral context emitted immediately after an action's primary effects
/// have been committed on the legacy single-target path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostActionContext {
    pub skill_id: SkillId,
    pub source: UnitId,
    pub primary_target: UnitId,
    pub cast_id: CastId,
    pub follow_up_depth: u8,
    pub unit_died: Option<PostActionUnitDied>,
    pub roster: Vec<PostActionUnitSnapshot>,
}

impl PostActionContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        skill_id: SkillId,
        source: UnitId,
        primary_target: UnitId,
        cast_id: CastId,
        follow_up_depth: u8,
        unit_died: Option<PostActionUnitDied>,
        roster: Vec<PostActionUnitSnapshot>,
    ) -> Self {
        Self {
            skill_id,
            source,
            primary_target,
            cast_id,
            follow_up_depth,
            unit_died,
            roster,
        }
    }

    pub fn unit(&self, unit_id: UnitId) -> Option<&PostActionUnitSnapshot> {
        self.roster.iter().find(|unit| unit.unit_id == unit_id)
    }

    pub fn source_unit(&self) -> Option<&PostActionUnitSnapshot> {
        self.unit(self.source)
    }

    pub fn primary_target_unit(&self) -> Option<&PostActionUnitSnapshot> {
        self.unit(self.primary_target)
    }
}

/// Accumulator filled by registered post-action reactions.
#[derive(Debug, Clone, Default)]
pub struct PostActionQueue {
    pub intents: Vec<Intent>,
    pub transitions: Vec<CombatKernelTransition>,
}

impl PostActionQueue {
    pub fn push_intent(&mut self, intent: Intent) {
        self.intents.push(intent);
    }

    pub fn push_transition(&mut self, transition: CombatKernelTransition) {
        self.transitions.push(transition);
    }

    pub fn is_empty(&self) -> bool {
        self.intents.is_empty() && self.transitions.is_empty()
    }
}

/// Execute every registered post-action reaction for the provided context.
///
/// Empty registries are a deterministic no-op.
pub fn dispatch_post_action_reactions(
    regs: &ExtRegistries,
    ctx: &PostActionContext,
) -> PostActionQueue {
    let mut out = PostActionQueue::default();
    for (_, reaction) in regs.post_action_reactions.iter() {
        reaction(ctx, &mut out);
    }
    out
}
