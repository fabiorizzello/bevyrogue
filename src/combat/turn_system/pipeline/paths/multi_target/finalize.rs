use bevy::prelude::*;

use crate::combat::energy::{Energy, EnergyGainSource, RoundEnergyTracker};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::CombatKernelRegistry;
use crate::combat::log::ActionLog;
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::status_effect::StatusEffectKind;
use crate::combat::types::UnitId;
use crate::combat::ult_gauge::{UltGaugeMetadata, drain_energy_on_ult_reset};

use super::super::super::super::{ResolveActorsQuery, emit_combat_event, set_phase};
use super::super::dispatch_blueprint_transitions;

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize(
    inflight: &InFlightAction,
    state: &mut ResMut<CombatState>,
    sp: &mut ResMut<SpPool>,
    log: &mut ResMut<ActionLog>,
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    actors: &mut ResolveActorsQuery,
    energy_q: &mut Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    gauge_meta_q: &Query<&UltGaugeMetadata>,
    cast_id: CastId,
    attacker_entity: Entity,
    attacker_id: UnitId,
    target_id: UnitId,
) -> bool {
    // Phase 3: post-loop attacker resource effects + once-per-cast events
    let ult_delta = {
        let Ok((_, _, _, _, att_ult_opt, _, _, _, _, _, att_bag_opt, mut att_streak_opt, _, _, _)) =
            actors.get_mut(attacker_entity)
        else {
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        };

        let Some(mut att_ult) = att_ult_opt else {
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        };

        let before = att_ult.current;

        match inflight.action.ult_effect {
            UltEffect::GainFromBasic => {
                sp.gain(1);
                let cpe = att_ult.charge_per_event;
                att_ult.try_add(cpe);
                if let Some(ref mut streak) = att_streak_opt {
                    streak.increment();
                }
            }
            UltEffect::None => {}
            UltEffect::Reset => {
                att_ult.current = 0;
            }
        }

        if inflight.action.ult_effect != UltEffect::Reset {
            if let Some(bag) = att_bag_opt.as_deref() {
                if bag.has(&StatusEffectKind::Blessed) {
                    att_ult.try_add(1);
                }
            }
        }

        att_ult.current - before
    };

    // S07/T03: drain `Energy.current` alongside legacy `UltimateCharge`
    // when the attacker is energy-backed. Legacy units are unaffected.
    if matches!(inflight.action.ult_effect, UltEffect::Reset) {
        let meta = gauge_meta_q.get(attacker_entity).ok();
        if let Ok((mut energy, _)) = energy_q.get_mut(attacker_entity) {
            drain_energy_on_ult_reset(meta, Some(energy.as_mut()));
        }
    }

    // Once-per-cast events (not per-target)
    emit_combat_event(
        event_writer,
        CombatEventKind::OnSkillCast {
            skill_id: inflight.action.skill_id.clone(),
        },
        attacker_id,
        target_id,
        inflight.follow_up_depth,
        cast_id,
    );

    if inflight.action.advance_pct != 0 {
        emit_combat_event(
            event_writer,
            CombatEventKind::AdvanceTurn {
                target: inflight.action.target,
                amount_pct: inflight.action.advance_pct,
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }
    if inflight.action.delay_pct != 0 {
        emit_combat_event(
            event_writer,
            CombatEventKind::DelayTurn {
                target: inflight.action.target,
                amount_pct: inflight.action.delay_pct,
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }
    if inflight.action.self_advance_pct != 0 {
        let capped = (inflight.action.self_advance_pct.max(0) as u32).min(50);
        if capped != 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::AdvanceTurn {
                    target: inflight.action.source,
                    amount_pct: capped,
                },
                attacker_id,
                attacker_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
    }

    if matches!(inflight.action.ult_effect, UltEffect::GainFromBasic) && ult_delta > 0 {
        emit_combat_event(
            event_writer,
            CombatEventKind::UltGain {
                unit_id: attacker_id,
                amount: ult_delta,
            },
            attacker_id,
            attacker_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }
    if matches!(inflight.action.ult_effect, UltEffect::Reset) {
        emit_combat_event(
            event_writer,
            CombatEventKind::UltimateUsed {
                unit_id: attacker_id,
            },
            attacker_id,
            attacker_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }

    if inflight.action.energy_grant > 0 {
        if let Ok((mut energy, mut tracker)) = energy_q.get_mut(attacker_entity) {
            let granted_by_round_cap = tracker
                .as_deref_mut()
                .map(|tracker| {
                    tracker.try_gain(
                        EnergyGainSource::SecondaryAction,
                        inflight.action.energy_grant,
                    )
                })
                .unwrap_or(inflight.action.energy_grant);
            let applied = energy.gain_capped(granted_by_round_cap);
            if applied > 0 {
                emit_combat_event(
                    event_writer,
                    CombatEventKind::EnergyGained {
                        unit_id: attacker_id,
                        amount: applied,
                    },
                    attacker_id,
                    attacker_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
    }

    dispatch_blueprint_transitions(inflight, log, event_writer, registry, cast_id);
    set_phase(state, CombatPhase::WaitingAction);
    true
}
