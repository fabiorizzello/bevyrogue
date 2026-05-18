use bevy::prelude::*;

use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::SkillBookHandle;
use bevyrogue::data::DataReady;
use bevyrogue::data::skills_ron::SkillBook;

use super::config::{CliProofConfig, CliProofState};

pub fn cli_proof_system(world: &mut World) {
    if !world.contains_resource::<CliProofConfig>() {
        return;
    }

    {
        let mut proof_state = world.resource_mut::<CliProofState>();
        if proof_state.finished {
            return;
        }
        proof_state.ticks += 1;
    }

    let ticks = world.resource::<CliProofState>().ticks;
    let tick_limit = world.resource::<CliProofConfig>().tick_limit;
    let data_ready = world.contains_resource::<DataReady>();
    let units_spawned = {
        let mut units = world.query::<&Unit>();
        units.iter(world).next().is_some()
    };
    let action_log_events = world
        .get_resource::<ActionLog>()
        .map(|log| log.events.len())
        .unwrap_or(0);
    let skill_book_ready = match (
        world.get_resource::<SkillBookHandle>(),
        world.get_resource::<Assets<SkillBook>>(),
    ) {
        (Some(handle), Some(skill_books)) => skill_books.get(&handle.0).is_some(),
        _ => false,
    };
    let action_resolved_without_log = match (
        world.get_resource::<CombatState>(),
        world.get_resource::<TurnOrder>(),
    ) {
        (Some(state), Some(order)) => {
            state.phase == CombatPhase::WaitingAction && order.active_unit.is_none() && ticks > 1
        }
        _ => false,
    };
    let proof_ready = data_ready
        && units_spawned
        && skill_book_ready
        && (action_log_events > 0 || action_resolved_without_log);

    if proof_ready {
        match capture_validation_snapshot(world) {
            Ok(snapshot) => {
                println!(
                    "[CLI_PROOF] validation_snapshot: {}",
                    format_validation_snapshot(&snapshot)
                );
                world.resource_mut::<CliProofState>().finished = true;
                world.write_message(AppExit::Success);
            }
            Err(err) => {
                eprintln!("[CLI_PROOF] validation_snapshot_error: {err}");
                world.resource_mut::<CliProofState>().finished = true;
                world.write_message(AppExit::error());
            }
        }
        return;
    }

    if ticks >= tick_limit {
        eprintln!(
            "[CLI_PROOF] readiness_timeout: ticks={ticks}/{tick_limit} data_ready={data_ready} units_spawned={units_spawned} action_log_events={action_log_events} skill_book_ready={skill_book_ready}"
        );
        world.resource_mut::<CliProofState>().finished = true;
        world.write_message(AppExit::error());
    }
}
