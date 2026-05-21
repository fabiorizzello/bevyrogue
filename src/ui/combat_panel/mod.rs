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
pub use labels::{
    TelegraphChip, attr_color, baby_burner_flash_chip, baby_burner_flash_text,
    baby_burner_flash_tooltip, cue_barrier_chip, query_pending_action_affordance,
    telegraph_chip_text, telegraph_chip_tooltip,
};
#[cfg(feature = "windowed")]
pub use preview_cache::refresh_preview_damage_cache;
#[cfg(feature = "windowed")]
pub use render::combat_panel;

#[cfg(feature = "windowed")]
use crate::combat::{
    blueprints::agumon::{OWNER as AGUMON_BLUEPRINT_OWNER, baby_burner::DETONATE_SIGNAL_NAME},
    counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    kit::UnitSkills,
    preview::PreviewDamageSummary,
    runtime::{CastId, SignalPayload},
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
pub const BABY_BURNER_FLASH_LIFETIME_FRAMES: u8 = 6;

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BabyBurnerFlashTrigger {
    pub source: UnitId,
    pub cast_id: CastId,
    pub targets: Vec<UnitId>,
    pub signal_owner: String,
    pub signal_name: String,
}

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BabyBurnerFlashDisplay {
    pub source: UnitId,
    pub cast_id: CastId,
    pub targets: Vec<UnitId>,
    pub signal_owner: String,
    pub signal_name: String,
    pub remaining_frames: u8,
    pub total_frames: u8,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct BabyBurnerFlashState {
    pub active: Option<BabyBurnerFlashDisplay>,
}

#[cfg(feature = "windowed")]
impl BabyBurnerFlashState {
    pub fn observe(&mut self, trigger: BabyBurnerFlashTrigger) {
        self.active = Some(BabyBurnerFlashDisplay {
            source: trigger.source,
            cast_id: trigger.cast_id,
            targets: trigger.targets,
            signal_owner: trigger.signal_owner,
            signal_name: trigger.signal_name,
            remaining_frames: BABY_BURNER_FLASH_LIFETIME_FRAMES,
            total_frames: BABY_BURNER_FLASH_LIFETIME_FRAMES,
        });
    }

    pub fn advance_frame(&mut self) {
        let Some(active) = self.active.as_mut() else {
            return;
        };

        if active.remaining_frames > 1 {
            active.remaining_frames -= 1;
        } else {
            self.active = None;
        }
    }

    pub fn active(&self) -> Option<&BabyBurnerFlashDisplay> {
        self.active.as_ref()
    }
}

#[cfg(feature = "windowed")]
pub fn latest_baby_burner_flash_trigger<'a>(
    events: impl IntoIterator<Item = &'a CombatEvent>,
) -> Option<BabyBurnerFlashTrigger> {
    let mut latest: Option<BabyBurnerFlashTrigger> = None;

    for event in events {
        let CombatEventKind::OnKernelTransition {
            transition:
                CombatKernelTransition::Blueprint {
                    owner,
                    name,
                    payload: SignalPayload::UnitTarget(target),
                },
        } = &event.kind
        else {
            continue;
        };

        if owner != AGUMON_BLUEPRINT_OWNER || name != DETONATE_SIGNAL_NAME {
            continue;
        }

        let needs_reset = match latest.as_ref() {
            None => true,
            Some(trigger) => trigger.cast_id != event.cast_id || trigger.source != event.source,
        };
        if needs_reset {
            latest = Some(BabyBurnerFlashTrigger {
                source: event.source,
                cast_id: event.cast_id,
                targets: vec![*target],
                signal_owner: owner.clone(),
                signal_name: name.clone(),
            });
            continue;
        }

        let trigger = latest.as_mut().expect("flash trigger initialized");
        if !trigger.targets.contains(target) {
            trigger.targets.push(*target);
        }
    }

    latest
}

#[cfg(feature = "windowed")]
pub fn observe_baby_burner_flash(
    mut events: MessageReader<CombatEvent>,
    mut flash: ResMut<BabyBurnerFlashState>,
) {
    if let Some(trigger) = latest_baby_burner_flash_trigger(events.read()) {
        flash.observe(trigger);
    }
}

#[cfg(feature = "windowed")]
pub fn advance_baby_burner_flash_state(mut flash: ResMut<BabyBurnerFlashState>) {
    flash.advance_frame();
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
