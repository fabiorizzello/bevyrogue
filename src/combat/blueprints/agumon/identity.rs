use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{
    CombatBeatId, CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition,
    CombatTagChangeKind, CombatTagId, CombatTagState, CombatTagTransition, TacticalCycleTransition,
    TwinCoreSignal, TwinCoreTransition,
};
use crate::combat::types::UnitId;

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
                        | TwinCoreDesignTag::Primed => out.push(CombatKernelTransition::TwinCore(
                            TwinCoreTransition::build_cross_resonance(1),
                        )),
                        TwinCoreDesignTag::ThermalSpark => out.push(
                            CombatKernelTransition::TwinCore(TwinCoreTransition::thermal_spark(1)),
                        ),
                        TwinCoreDesignTag::MeltdownCrack => {
                            out.push(CombatKernelTransition::TwinCore(
                                TwinCoreTransition::fire_spend_marker(1),
                            ))
                        }
                        TwinCoreDesignTag::DeepCrack => out.push(CombatKernelTransition::TwinCore(
                            TwinCoreTransition::ice_spend_marker(1),
                        )),
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
                    out.push(CombatKernelTransition::TwinCore(
                        TwinCoreTransition::shatter(1),
                    ));
                }
            }
            CombatKernelTransition::Beat(CombatBeatId::Damage) => {
                out.push(CombatKernelTransition::TwinCore(
                    TwinCoreTransition::twin_burst(1),
                ));
            }
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            }) => {
                out.push(CombatKernelTransition::TwinCore(
                    TwinCoreTransition::cycle_reset(),
                ));
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

        let CombatKernelTransition::TwinCore(twin_trans) = transition else {
            continue;
        };

        apply_twin_core_transition(&mut state, *twin_trans, event.target);
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
