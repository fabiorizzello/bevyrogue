#[cfg(feature = "windowed")]
mod display;
#[cfg(feature = "windowed")]
mod labels;
#[cfg(feature = "windowed")]
mod preview_cache;
#[cfg(feature = "windowed")]
mod render;
#[cfg(feature = "windowed")]
mod widgets;

#[cfg(feature = "windowed")]
pub use labels::{attr_color, query_pending_action_affordance};
#[cfg(feature = "windowed")]
pub use preview_cache::refresh_preview_damage_cache;
#[cfg(feature = "windowed")]
pub use render::combat_panel;

#[cfg(feature = "windowed")]
use crate::combat::{
    counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    kit::UnitSkills,
    preview::PreviewDamageSummary,
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    types::{SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{Commander, Ko, Unit},
};

#[cfg(feature = "windowed")]
use bevy::prelude::*;

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingKind {
    Basic,
    Skill(SkillId),
    Ultimate,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingAction {
    pub kind: Option<PendingKind>,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PreviewDamageCache {
    pub actor_id: Option<UnitId>,
    pub pending_kind: Option<PendingKind>,
    pub skill_id: Option<SkillId>,
    pub target_id: Option<UnitId>,
    pub summary: Option<PreviewDamageSummary>,
}

#[cfg(feature = "windowed")]
impl PreviewDamageCache {
    pub(crate) fn matches_context(
        &self,
        actor_id: Option<UnitId>,
        pending_kind: Option<&PendingKind>,
        target_id: Option<UnitId>,
    ) -> bool {
        self.actor_id == actor_id
            && self.target_id == target_id
            && self.pending_kind.as_ref() == pending_kind
    }

    pub(crate) fn label_for(
        &self,
        actor_id: Option<UnitId>,
        pending_kind: Option<&PendingKind>,
        target_id: Option<UnitId>,
    ) -> Option<String> {
        let summary = self.summary.as_ref()?;
        if !self.matches_context(actor_id, pending_kind, target_id) {
            return None;
        }

        Some(format!(
            "preview: {} dmg across {} hit(s)",
            summary.total_damage, summary.deal_damage_intents
        ))
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn pending_kind_skill_id(kind: &PendingKind, kit: &UnitSkills) -> SkillId {
    match kind {
        PendingKind::Basic => kit.basic.clone(),
        PendingKind::Skill(skill_id) => skill_id.clone(),
        PendingKind::Ultimate => kit.ultimate.clone(),
    }
}

#[cfg(feature = "windowed")]
pub(crate) type CombatPanelUnitsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Unit,
        &'static Team,
        Option<&'static Toughness>,
        Option<&'static EnemyCounterplayKit>,
        &'static UltimateCharge,
        &'static UnitSkills,
        Option<&'static Ko>,
        Option<&'static Commander>,
        Option<&'static Stunned>,
        Option<&'static Energy>,
        Option<&'static RoundEnergyTracker>,
    ),
>;
