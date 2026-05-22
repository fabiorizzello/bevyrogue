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
    telegraph_chip_text, telegraph_chip_tooltip, twin_core_badge_chip, twin_core_badge_text,
    twin_core_badge_tooltip,
};
#[cfg(feature = "windowed")]
pub use preview_cache::refresh_preview_damage_cache;
#[cfg(feature = "windowed")]
pub use render::combat_panel;

#[cfg(feature = "windowed")]
use crate::combat::ult_gauge::UltGaugeMetadata;
#[cfg(feature = "windowed")]
use crate::combat::{
    blueprints::agumon::{OWNER as AGUMON_BLUEPRINT_OWNER, baby_burner::DETONATE_SIGNAL_NAME},
    counterplay::EnemyCounterplayKit,
    energy::Energy,
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
        Option<&'static UltGaugeMetadata>,
    ),
>;

// ── HUD display-state resources ──────────────────────────────────────────────
// Pure computed projections: read Unit/FloatingDamage, write view resources.
// No CombatState mutations. Testable without a display.

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq)]
pub struct HpBarEntry {
    pub unit_id: UnitId,
    pub cur: i32,
    pub max: i32,
    /// hp_current / hp_max clamped to [0, 1].
    pub pct: f32,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone)]
pub struct HpBarView {
    pub bars: Vec<HpBarEntry>,
}

#[cfg(feature = "windowed")]
pub fn compute_hp_bar_view(units: Query<&Unit>, mut view: ResMut<HpBarView>) {
    view.bars.clear();
    for unit in &units {
        let pct = if unit.hp_max > 0 {
            (unit.hp_current as f32 / unit.hp_max as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        view.bars.push(HpBarEntry {
            unit_id: unit.id,
            cur: unit.hp_current,
            max: unit.hp_max,
            pct,
        });
    }
}

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq)]
pub struct FdViewEntry {
    pub unit_id: UnitId,
    pub text: String,
    /// 1.0 at spawn, fading to 0.0 at FLOATING_LIFETIME_SECS.
    pub alpha: f32,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone)]
pub struct FloatingDamageView {
    pub entries: Vec<FdViewEntry>,
}

// ── Target hurt state ────────────────────────────────────────────────────────

#[cfg(feature = "windowed")]
pub const HURT_FRAMES: u32 = 12;

/// Per-unit frame countdown driven by `CombatEventKind::OnHitTaken`.
/// Windowed-only projection resource; never touches `CombatState`.
#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct TargetHurtState {
    pub entries: std::collections::HashMap<UnitId, u32>,
}

#[cfg(feature = "windowed")]
impl TargetHurtState {
    /// Returns `true` if the unit is currently in a hurt blink window.
    pub fn is_hurt(&self, id: UnitId) -> bool {
        self.entries.get(&id).copied().unwrap_or(0) > 0
    }
}

#[cfg(feature = "windowed")]
pub fn observe_target_hurt(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<TargetHurtState>,
) {
    for event in events.read() {
        if let CombatEventKind::OnHitTaken { .. } = &event.kind {
            let entry = state.entries.entry(event.target).or_insert(0);
            if *entry < HURT_FRAMES {
                *entry = HURT_FRAMES;
            }
        }
    }
}

// ── Twin Core synergy badge ──────────────────────────────────────────────────

/// Frames a Twin Core synergy chip remains visible after the blueprint emits
/// any signal projection.
#[cfg(feature = "windowed")]
pub const TWIN_CORE_BADGE_FRAMES: u32 = 60;

/// Owner string for Twin Core blueprint signals (mirrors `blueprints::twin_core::OWNER`).
/// Kept here as a literal so this module avoids depending on the blueprint
/// internals for what is a windowed-only presentation projection.
#[cfg(feature = "windowed")]
const TWIN_CORE_OWNER: &str = "twin_core";

/// Windowed-only frame countdown set when any `twin_core` blueprint signal is
/// projected through `OnKernelTransition`. Never mutates `CombatState`.
///
/// Priming is one-shot per active window: re-priming while `primed_for_frames`
/// is non-zero is a no-op so a single Ultimate that fans out multiple Twin Core
/// signals only triggers the chip once.
#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct TwinCoreBadgeState {
    pub primed_for_frames: u32,
    pub last_signal_name: Option<String>,
}

#[cfg(feature = "windowed")]
impl TwinCoreBadgeState {
    pub fn is_primed(&self) -> bool {
        self.primed_for_frames > 0
    }
}

#[cfg(feature = "windowed")]
pub fn observe_twin_core_badge(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<TwinCoreBadgeState>,
) {
    for event in events.read() {
        let CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, .. },
        } = &event.kind
        else {
            continue;
        };
        if owner != TWIN_CORE_OWNER {
            continue;
        }
        if state.primed_for_frames == 0 {
            state.primed_for_frames = TWIN_CORE_BADGE_FRAMES;
            state.last_signal_name = Some(name.clone());
        }
    }
}

#[cfg(feature = "windowed")]
pub fn tick_twin_core_badge(mut state: ResMut<TwinCoreBadgeState>) {
    if state.primed_for_frames > 0 {
        state.primed_for_frames -= 1;
        if state.primed_for_frames == 0 {
            state.last_signal_name = None;
        }
    }
}

#[cfg(feature = "windowed")]
pub fn tick_target_hurt_state(mut state: ResMut<TargetHurtState>) {
    state.entries.retain(|_, remaining| {
        if *remaining > 0 {
            *remaining -= 1;
        }
        *remaining > 0
    });
}

#[cfg(feature = "windowed")]
pub fn compute_floating_damage_view(
    time: Res<Time>,
    fds: Query<&crate::combat::floating::FloatingDamage>,
    mut view: ResMut<FloatingDamageView>,
) {
    use crate::combat::floating::FLOATING_LIFETIME_SECS;
    use crate::combat::toughness::DamageKind;
    let now = time.elapsed_secs();
    view.entries.clear();
    for fd in &fds {
        let elapsed = now - fd.spawn_time;
        if elapsed >= FLOATING_LIFETIME_SECS {
            continue;
        }
        let alpha = 1.0 - (elapsed / FLOATING_LIFETIME_SECS).clamp(0.0, 1.0);
        let prefix = match fd.kind {
            DamageKind::Normal => "",
            DamageKind::Weak => "WEAK ",
            DamageKind::Resist => "RES ",
            DamageKind::Break => "BRK ",
        };
        view.entries.push(FdViewEntry {
            unit_id: fd.target,
            text: format!("{}{}", prefix, fd.amount),
            alpha,
        });
    }
}
