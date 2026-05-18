use super::av::ActionValue;
use super::resistance::TempoResistance;
use super::*;
use crate::combat::{
    state::{CombatPhase, CombatState},
    team::Team,
    unit::{Ko, Unit},
};
use bevy::prelude::*;

/// Processes `CombatEvent::AdvanceTurn` and `DelayTurn` messages, applying the
/// corresponding AV delta via the T01 pure-logic primitives.
pub fn apply_av_ops_system(
    mut events: MessageReader<crate::combat::events::CombatEvent>,
    mut units: Query<(
        &crate::combat::unit::Unit,
        &mut ActionValue,
        Option<&mut TempoResistance>,
    )>,
) {
    use crate::combat::events::CombatEventKind;
    for event in events.read() {
        match &event.kind {
            CombatEventKind::AdvanceTurn { target, amount_pct } => {
                for (unit, mut av, _) in &mut units {
                    if unit.id == *target {
                        resistance::apply_advance(&mut av, *amount_pct);
                        break;
                    }
                }
            }
            CombatEventKind::DelayTurn { target, amount_pct } => {
                for (unit, mut av, mut res) in &mut units {
                    if unit.id == *target {
                        resistance::apply_delay(&mut av, *amount_pct, res.as_deref_mut());
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn check_victory_system(
    mut state: ResMut<CombatState>,
    roster: Query<(&Unit, &Team, Option<&Ko>)>,
) {
    if state.winner.is_some() {
        return;
    }

    let mut ally_alive = false;
    let mut enemy_alive = false;
    let mut saw_ally = false;
    let mut saw_enemy = false;
    for (unit, team, ko) in &roster {
        let alive = ko.is_none() && unit.hp_current > 0;
        match team {
            Team::Ally => {
                saw_ally = true;
                ally_alive |= alive;
            }
            Team::Enemy => {
                saw_enemy = true;
                enemy_alive |= alive;
            }
        }
    }

    if !saw_ally || !saw_enemy {
        return;
    }

    if !enemy_alive {
        set_phase(&mut state, CombatPhase::Victory);
        state.winner = Some(Team::Ally);
        info!("victory: team=Ally");
    } else if !ally_alive {
        set_phase(&mut state, CombatPhase::Defeat);
        state.winner = Some(Team::Enemy);
        info!("defeat: team=Enemy");
    }
}
