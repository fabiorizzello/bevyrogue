use bevy::prelude::*;
use moonshine_kind::Instance;

use bevyrogue::combat::bootstrap::{SelectionRequest, apply_composition, bootstrap_encounter};
use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::events::CombatEventKind;
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::{DataReady, UnitRosterHandle, units_ron::UnitRoster};

use super::config::{CliProofConfig, IsInteractive, SelectedAllies, SelectedEncounter, TickCounter};

pub fn timeout_system(
    mut counter: ResMut<TickCounter>,
    mut exit: MessageWriter<AppExit>,
    interactive: Res<IsInteractive>,
    proof_config: Option<Res<CliProofConfig>>,
) {
    if proof_config.is_some() {
        return;
    }

    counter.0 += 1;
    // Non-interactive CI runs exit after ~6 seconds. Interactive runs don't time out.
    if !interactive.0 && counter.0 > 60 {
        exit.write(AppExit::Success);
    }
}

pub fn bootstrap_system(
    mut commands: Commands,
    data_ready: Option<Res<DataReady>>,
    roster_handle: Option<Res<UnitRosterHandle>>,
    rosters: Res<Assets<UnitRoster>>,
    selected_allies: Res<SelectedAllies>,
    selected_encounter: Res<SelectedEncounter>,
    mut combat_state: ResMut<CombatState>,
    mut combat_events: MessageWriter<CombatEvent>,
    units: Query<Instance<Unit>>,
    mut exit: MessageWriter<AppExit>,
) {
    if data_ready.is_none() || !units.is_empty() {
        return;
    }

    let Some(rhandle) = roster_handle else { return };
    let Some(roster) = rosters.get(&rhandle.0) else {
        return;
    };

    let request = SelectionRequest {
        rookie_ids: selected_allies.0.clone(),
    };

    println!("Encounter preset: {}", selected_encounter.0);
    match bootstrap_encounter(roster, &request, selected_encounter.0) {
        Ok(composition) => {
            let seeded_ids = apply_composition(&mut commands, &composition);
            combat_state.phase = CombatPhase::WaitingForTurn;
            println!("Bootstrap successful. Party: {:?}", request.rookie_ids);
            combat_events.write(CombatEvent {
                source: UnitId(0),
                target: UnitId(0),
                kind: CombatEventKind::PartySelected {
                    ally_ids: selected_allies.0.clone(),
                    tamer_id: UnitId(0),
                },
                follow_up_depth: 0,
                cast_id: CastId::ROOT,
            });
            combat_events.write(CombatEvent {
                source: UnitId(0),
                target: UnitId(0),
                kind: CombatEventKind::TurnOrderSeeded {
                    unit_ids: seeded_ids,
                },
                follow_up_depth: 0,
                cast_id: CastId::ROOT,
            });
            // AV system: advance_turn_system handles initial turn selection automatically.
        }
        Err(err) => {
            eprintln!("Bootstrap error: {:?}", err);
            exit.write(AppExit::error());
        }
    }
}

pub fn event_logger_system(mut events: MessageReader<CombatEvent>) {
    for event in events.read() {
        println!(
            "  [EVENT] {:?} (source: {:?}, target: {:?}, depth: {})",
            event.kind, event.source, event.target, event.follow_up_depth
        );
    }
}
