//! Pure event-to-stance-reaction mapping.
//!
//! Mirrors the R009 `AnimGraphInput` purity seam: the classification from a
//! combat event kind to a stance reaction is a deterministic, headless lib
//! function with no windowed/bevy-render dependency. The windowed bridge (K001)
//! consumes this mapping; integration tests link only against the lib crate.

use crate::animation::anim_graph::NodeId;
use crate::combat::observability::events::CombatEventKind;

/// Closed vocabulary of stance reactions a combat event can drive.
///
/// Intentionally not derived from the open `CombatEventKind` taxonomy: only the
/// events that should perturb a unit's stance graph are represented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StanceReaction {
    /// The unit took a non-lethal hit and should play its `hurt` node.
    Hurt,
    /// The unit died and should play its `death` node.
    Death,
}

impl StanceReaction {
    /// The stance graph node this reaction drives.
    ///
    /// Node names match `assets/digimon/agumon/stance.ron` (`hurt` / `death`).
    pub fn stance_node(self) -> NodeId {
        match self {
            StanceReaction::Hurt => NodeId("hurt".to_string()),
            StanceReaction::Death => NodeId("death".to_string()),
        }
    }
}

/// Map a single combat event kind to its stance reaction, if any.
///
/// Total over the `CombatEventKind` taxonomy: reaction-bearing events map to
/// `Some(..)`, every other variant maps to `None`. Matched explicitly so adding
/// a new variant forces a compile error here rather than silently defaulting.
pub fn stance_reaction_for(kind: &CombatEventKind) -> Option<StanceReaction> {
    match kind {
        CombatEventKind::OnHitTaken { .. } => Some(StanceReaction::Hurt),
        CombatEventKind::UnitDied { .. } => Some(StanceReaction::Death),
        // Non-reaction events: explicitly enumerated so a future variant must be
        // classified deliberately instead of being swallowed by a catch-all.
        CombatEventKind::OnSkillCast { .. }
        | CombatEventKind::IncomingDamage { .. }
        | CombatEventKind::BlockReactionTriggered { .. }
        | CombatEventKind::OnDamageDealt { .. }
        | CombatEventKind::OnBreak { .. }
        | CombatEventKind::OnRevive { .. }
        | CombatEventKind::OnActionFailed { .. }
        | CombatEventKind::OnStatusTick { .. }
        | CombatEventKind::OnStatusExpired { .. }
        | CombatEventKind::OnAllyLowHp
        | CombatEventKind::OnEnemyKill
        | CombatEventKind::PartySelected { .. }
        | CombatEventKind::TurnOrderSeeded { .. }
        | CombatEventKind::UltGain { .. }
        | CombatEventKind::UltimateUsed { .. }
        | CombatEventKind::OnStatusApplied { .. }
        | CombatEventKind::OnStatusResisted { .. }
        | CombatEventKind::AdvanceTurn { .. }
        | CombatEventKind::DelayTurn { .. }
        | CombatEventKind::OnActionDeclared { .. }
        | CombatEventKind::OnActionPreApp
        | CombatEventKind::OnCombatBeat { .. }
        | CombatEventKind::OnKernelTransition { .. }
        | CombatEventKind::OnActionApplied
        | CombatEventKind::OnActionResolved
        | CombatEventKind::EnergyGained { .. }
        | CombatEventKind::OnHealed { .. }
        | CombatEventKind::OnCleansed { .. } => None,
    }
}

/// Resolve the dominant stance reaction over a batch of event kinds.
///
/// Death-precedence: if any kind maps to `Death`, the batch resolves to
/// `Death` regardless of co-occurring hits; otherwise `Hurt` if any maps to
/// `Hurt`; otherwise `None`. This keeps a unit that died in the same window it
/// was struck from playing `hurt` instead of `death`.
pub fn resolve_stance_reaction<'a>(
    kinds: impl IntoIterator<Item = &'a CombatEventKind>,
) -> Option<StanceReaction> {
    let mut saw_hurt = false;
    for kind in kinds {
        match stance_reaction_for(kind) {
            Some(StanceReaction::Death) => return Some(StanceReaction::Death),
            Some(StanceReaction::Hurt) => saw_hurt = true,
            _ => {}
        }
    }
    saw_hurt.then_some(StanceReaction::Hurt)
}
