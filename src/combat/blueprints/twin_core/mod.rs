use crate::combat::bevy_types::*;
use serde::{Deserialize, Serialize};

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{
    CombatBeatId, CombatKernelHook, CombatKernelHookDomain, CombatKernelRegistry,
    CombatKernelTransition, CombatTagChangeKind, CombatTagId, CombatTagState, CombatTagTransition,
    TacticalCycleTransition,
};
use crate::combat::runtime::registry::{ValidationField, ValidationSection};
use crate::combat::observability::format_unit_ids;
use crate::combat::types::UnitId;

pub const OWNER: &str = "twin_core";

const SIG_BUILD_CROSS_RESONANCE: &str = "build_cross_resonance";
const SIG_SPEND_CROSS_RESONANCE: &str = "spend_cross_resonance";
const SIG_THERMAL_SPARK: &str = "thermal_spark";
const SIG_TWIN_BURST: &str = "twin_burst";
const SIG_SHATTER: &str = "shatter";
const SIG_FIRE_SPEND_MARKER: &str = "fire_spend_marker";
const SIG_ICE_SPEND_MARKER: &str = "ice_spend_marker";
const SIG_CYCLE_RESET: &str = "cycle_reset";

pub const TAG_HEATED: &str = "Heated";
pub const TAG_CHILLED: &str = "Chilled";
pub const TAG_THERMAL_SPARK: &str = "Thermal Spark";
pub const TAG_PRIMED: &str = "Primed";
pub const TAG_MELTDOWN_CRACK: &str = "Meltdown Crack";
pub const TAG_DEEP_CRACK: &str = "Deep Crack";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCoreDesignTag {
    Heated,
    Chilled,
    ThermalSpark,
    Primed,
    MeltdownCrack,
    DeepCrack,
}

pub fn twin_core_design_tag_name(tag: TwinCoreDesignTag) -> &'static str {
    match tag {
        TwinCoreDesignTag::Heated => TAG_HEATED,
        TwinCoreDesignTag::Chilled => TAG_CHILLED,
        TwinCoreDesignTag::ThermalSpark => TAG_THERMAL_SPARK,
        TwinCoreDesignTag::Primed => TAG_PRIMED,
        TwinCoreDesignTag::MeltdownCrack => TAG_MELTDOWN_CRACK,
        TwinCoreDesignTag::DeepCrack => TAG_DEEP_CRACK,
    }
}

pub fn twin_core_design_tag(tag: TwinCoreDesignTag) -> CombatTagId {
    CombatTagId(twin_core_design_tag_name(tag).to_string())
}

pub fn classify_twin_core_tag(tag: &CombatTagId) -> Option<TwinCoreDesignTag> {
    match tag.0.as_str() {
        TAG_HEATED => Some(TwinCoreDesignTag::Heated),
        TAG_CHILLED => Some(TwinCoreDesignTag::Chilled),
        TAG_THERMAL_SPARK => Some(TwinCoreDesignTag::ThermalSpark),
        TAG_PRIMED => Some(TwinCoreDesignTag::Primed),
        TAG_MELTDOWN_CRACK => Some(TwinCoreDesignTag::MeltdownCrack),
        TAG_DEEP_CRACK => Some(TwinCoreDesignTag::DeepCrack),
        _ => None,
    }
}

pub fn twin_core_added_tag_transition(
    tag: TwinCoreDesignTag,
    turns_left: u8,
) -> CombatKernelTransition {
    let id = twin_core_design_tag(tag);
    let before = CombatTagState::new(id.clone(), turns_left);
    let after = CombatTagState::new(id, turns_left);
    CombatKernelTransition::Tag(CombatTagTransition {
        before,
        after,
        kind: CombatTagChangeKind::Added,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCoreSignal {
    BuildCrossResonance,
    SpendCrossResonance,
    ThermalSpark,
    TwinBurst,
    Shatter,
    FireSpendMarker,
    IceSpendMarker,
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwinCoreTransition {
    pub signal: TwinCoreSignal,
    pub amount: u8,
}

#[allow(dead_code)] // consumed by integration tests
impl TwinCoreTransition {
    pub const fn build_cross_resonance(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::BuildCrossResonance,
            amount,
        }
    }

    pub const fn spend_cross_resonance(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::SpendCrossResonance,
            amount,
        }
    }

    pub const fn thermal_spark(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::ThermalSpark,
            amount,
        }
    }

    pub const fn twin_burst(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::TwinBurst,
            amount,
        }
    }

    pub const fn shatter(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::Shatter,
            amount,
        }
    }

    pub const fn fire_spend_marker(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::FireSpendMarker,
            amount,
        }
    }

    pub const fn ice_spend_marker(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::IceSpendMarker,
            amount,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: TwinCoreSignal::CycleReset,
            amount: 0,
        }
    }
}

#[derive(Debug, Resource, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TwinCoreState {
    pub active_thermal_spark_targets: Vec<UnitId>,
    pub cross_resonance: u8,
    pub fire_spend_markers: u8,
    pub ice_spend_markers: u8,
    pub twin_burst_used_this_cycle: bool,
    pub shatter_used_this_cycle: bool,
    pub last_signal: Option<TwinCoreTransition>,
}

impl Default for TwinCoreState {
    fn default() -> Self {
        Self {
            active_thermal_spark_targets: Vec::new(),
            cross_resonance: 0,
            fire_spend_markers: 0,
            ice_spend_markers: 0,
            twin_burst_used_this_cycle: false,
            shatter_used_this_cycle: false,
            last_signal: None,
        }
    }
}

impl TwinCoreState {
    pub fn validation_section(&self) -> ValidationSection {
        let mut spark_targets = self.active_thermal_spark_targets.clone();
        spark_targets.sort_by_key(|unit_id| unit_id.0);

        ValidationSection::new(
            OWNER,
            vec![
                ValidationField::new("cr", self.cross_resonance.to_string()),
                ValidationField::new("spark_targets", format_unit_ids(&spark_targets)),
                ValidationField::new("fire", self.fire_spend_markers.to_string()),
                ValidationField::new("ice", self.ice_spend_markers.to_string()),
                ValidationField::new(
                    "burst_guard",
                    self.twin_burst_used_this_cycle.to_string(),
                ),
                ValidationField::new(
                    "shatter_guard",
                    self.shatter_used_this_cycle.to_string(),
                ),
                ValidationField::new(
                    "last",
                    self.last_signal
                        .map(format_twin_core_transition)
                        .unwrap_or_else(|| "none".to_string()),
                ),
            ],
        )
    }
}

pub fn register_validation_ext(regs: &mut crate::combat::runtime::ExtRegistries) {
    regs.validation
        .register("twin_core/validation", twin_core_validation_section);
}

fn twin_core_validation_section(world: &World) -> Option<ValidationSection> {
    world
        .get_resource::<TwinCoreState>()
        .map(TwinCoreState::validation_section)
}

fn blueprint_transition(name: &'static str, amount: i64) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_string(),
        name: name.to_string(),
        payload: crate::combat::runtime::SignalPayload::Amount(amount),
    }
}

pub struct TwinCoreHook;

impl CombatKernelHook for TwinCoreHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        transition: &CombatKernelTransition,
        out: &mut Vec<CombatKernelTransition>,
    ) {
        match transition {
            CombatKernelTransition::Tag(tag_transition)
                if tag_transition.kind == CombatTagChangeKind::Added =>
            {
                if let Some(tag) = classify_twin_core_tag(&tag_transition.after.id) {
                    match tag {
                        TwinCoreDesignTag::Heated
                        | TwinCoreDesignTag::Chilled
                        | TwinCoreDesignTag::Primed => {
                            out.push(blueprint_transition(SIG_BUILD_CROSS_RESONANCE, 1))
                        }
                        TwinCoreDesignTag::ThermalSpark => {
                            out.push(blueprint_transition(SIG_THERMAL_SPARK, 1))
                        }
                        TwinCoreDesignTag::MeltdownCrack => {
                            out.push(blueprint_transition(SIG_FIRE_SPEND_MARKER, 1))
                        }
                        TwinCoreDesignTag::DeepCrack => {
                            out.push(blueprint_transition(SIG_ICE_SPEND_MARKER, 1))
                        }
                    }
                }
            }
            CombatKernelTransition::Tag(tag_transition)
                if tag_transition.kind == CombatTagChangeKind::Consumed
                    || tag_transition.kind == CombatTagChangeKind::Expired =>
            {
                if matches!(
                    classify_twin_core_tag(&tag_transition.after.id),
                    Some(TwinCoreDesignTag::ThermalSpark)
                ) {
                    out.push(blueprint_transition(SIG_SHATTER, 1));
                }
            }
            CombatKernelTransition::Beat(CombatBeatId::Damage) => {
                out.push(blueprint_transition(SIG_TWIN_BURST, 1));
            }
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            }) => {
                out.push(blueprint_transition(SIG_CYCLE_RESET, 0));
            }
            _ => {}
        }
    }
}

pub fn apply_twin_core_transitions_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<TwinCoreState>,
) {
    for event in events.read() {
        let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
            continue;
        };

        let CombatKernelTransition::Blueprint {
            owner,
            name,
            payload,
        } = transition
        else {
            continue;
        };

        if owner != OWNER {
            continue;
        }

        let signal = match name.as_str() {
            SIG_BUILD_CROSS_RESONANCE => TwinCoreSignal::BuildCrossResonance,
            SIG_SPEND_CROSS_RESONANCE => TwinCoreSignal::SpendCrossResonance,
            SIG_THERMAL_SPARK => TwinCoreSignal::ThermalSpark,
            SIG_TWIN_BURST => TwinCoreSignal::TwinBurst,
            SIG_SHATTER => TwinCoreSignal::Shatter,
            SIG_FIRE_SPEND_MARKER => TwinCoreSignal::FireSpendMarker,
            SIG_ICE_SPEND_MARKER => TwinCoreSignal::IceSpendMarker,
            SIG_CYCLE_RESET => TwinCoreSignal::CycleReset,
            _ => continue,
        };

        let amount = match payload {
            crate::combat::runtime::SignalPayload::Amount(a) => *a as u8,
            _ => 0,
        };

        let twin_trans = TwinCoreTransition { signal, amount };
        apply_twin_core_transition(&mut state, twin_trans, event.target);
    }
}

fn apply_twin_core_transition(
    state: &mut TwinCoreState,
    transition: TwinCoreTransition,
    target: UnitId,
) {
    match transition.signal {
        TwinCoreSignal::BuildCrossResonance => {
            state.cross_resonance = state
                .cross_resonance
                .saturating_add(transition.amount)
                .min(2);
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::SpendCrossResonance => {
            state.cross_resonance = state.cross_resonance.saturating_sub(transition.amount);
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::ThermalSpark => {
            if !state.active_thermal_spark_targets.contains(&target) {
                state.active_thermal_spark_targets.push(target);
            }
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::TwinBurst => {
            if !state.twin_burst_used_this_cycle && state.cross_resonance == 2 {
                state.twin_burst_used_this_cycle = true;
                state.cross_resonance = state.cross_resonance.saturating_sub(2);
                if state.active_thermal_spark_targets.contains(&target) {
                    state
                        .active_thermal_spark_targets
                        .retain(|spark_target| *spark_target != target);
                }
            }
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::Shatter => {
            if !state.shatter_used_this_cycle {
                state.shatter_used_this_cycle = true;
                if state.active_thermal_spark_targets.contains(&target) {
                    state
                        .active_thermal_spark_targets
                        .retain(|spark_target| *spark_target != target);
                }
            }
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::FireSpendMarker => {
            state.fire_spend_markers = state.fire_spend_markers.saturating_add(transition.amount);
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::IceSpendMarker => {
            state.ice_spend_markers = state.ice_spend_markers.saturating_add(transition.amount);
            state.last_signal = Some(transition);
        }
        TwinCoreSignal::CycleReset => {
            state.fire_spend_markers = 0;
            state.ice_spend_markers = 0;
            state.twin_burst_used_this_cycle = false;
            state.shatter_used_this_cycle = false;
            state.last_signal = Some(transition);
        }
    }

    debug!(
        "TwinCoreState signal={:?} resonance={} spark_targets={:?} fire_spend={} ice_spend={} burst_guard={} shatter_guard={}",
        state.last_signal,
        state.cross_resonance,
        state.active_thermal_spark_targets,
        state.fire_spend_markers,
        state.ice_spend_markers,
        state.twin_burst_used_this_cycle,
        state.shatter_used_this_cycle,
    );
}

pub struct TwinCorePlugin;

impl Plugin for TwinCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TwinCoreState>()
            .add_systems(Update, apply_twin_core_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(TwinCoreHook);
    }
}

pub(crate) fn format_twin_core_transition(transition: TwinCoreTransition) -> String {
    let signal = match transition.signal {
        TwinCoreSignal::BuildCrossResonance => "build",
        TwinCoreSignal::SpendCrossResonance => "spend",
        TwinCoreSignal::ThermalSpark => "spark",
        TwinCoreSignal::TwinBurst => "twin-burst",
        TwinCoreSignal::Shatter => "shatter",
        TwinCoreSignal::FireSpendMarker => "fire-spend",
        TwinCoreSignal::IceSpendMarker => "ice-spend",
        TwinCoreSignal::CycleReset => "cycle-reset",
    };
    format!("{signal}({})", transition.amount)
}
