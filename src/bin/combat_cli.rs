use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use inquire::{MultiSelect, Select};
use std::env;
use std::io::IsTerminal;
use std::time::Duration;

// Use the library modules
use bevyrogue::CombatPlugin;
use bevyrogue::combat::bootstrap::EncounterPreset;
use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::follow_up::{FollowUpIntent, FollowUpTrace};
use bevyrogue::combat::follow_up::{
    follow_up_listener_system, form_identity_listener_system, resolve_follow_up_action_system,
};
use bevyrogue::combat::jsonl_logger::jsonl_logger_system;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::turn_system::{
    advance_turn_system, check_victory_system, resolve_action_system,
    resolve_enemy_turn_action_system,
};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::ultimate::UltGainQueue;
use bevyrogue::combat::ultimate::{flush_ult_gain_system, ult_accumulation_system};
use bevyrogue::data::DataPlugin;
use bevyrogue::combat::av::ActionValueUpdated;

#[path = "combat_cli/config.rs"]
mod config;
#[path = "combat_cli/bootstrap.rs"]
mod bootstrap;
#[path = "combat_cli/dashboard.rs"]
mod dashboard;
#[path = "combat_cli/menu.rs"]
mod menu;
#[path = "combat_cli/player.rs"]
mod player;
#[path = "combat_cli/proof.rs"]
mod proof;
#[path = "combat_cli/assets.rs"]
mod assets;
#[path = "combat_cli/scenarios.rs"]
mod scenarios;

use config::{
    CliProofConfig, CliProofState, IsInteractive, PlayerActed, SelectedAllies, SelectedEncounter,
    TickCounter, cli_proof_enabled, cli_proof_tick_limit_from_env,
};
use bootstrap::{bootstrap_system, event_logger_system, timeout_system};
use dashboard::combat_dashboard_system;
use player::player_action_system;
use proof::cli_proof_system;
use assets::{load_ally_roster, manifest_assets_dir, verify_required_data_assets};
use scenarios::{run_advance_delay_cap_scenario, run_aoe_blast_scenario};

fn main() -> AppExit {
    // Handle --scenario before starting Bevy.
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--scenario") {
        match args.get(pos + 1).map(String::as_str) {
            Some("advance-delay-cap") => {
                run_advance_delay_cap_scenario();
                return AppExit::Success;
            }
            Some("aoe-blast") => {
                run_aoe_blast_scenario();
                return AppExit::Success;
            }
            Some(other) => {
                eprintln!("Unknown scenario: {other}");
                return AppExit::error();
            }
            None => {
                eprintln!("--scenario requires an argument");
                return AppExit::error();
            }
        }
    }

    println!("=== BevyRogue Combat CLI Harness ===");

    let proof_mode = cli_proof_enabled();
    let is_terminal = std::io::stdin().is_terminal() && !proof_mode;

    // Load ally roster synchronously before Bevy starts.
    if let Err(err) = verify_required_data_assets() {
        eprintln!("combat_cli startup error: {err}");
        return AppExit::error();
    }
    let ally_defs = match load_ally_roster() {
        Ok(ally_defs) => ally_defs,
        Err(err) => {
            eprintln!("combat_cli startup error: {err}");
            return AppExit::error();
        }
    };
    let selected_ids: Vec<UnitId>;

    if is_terminal && !ally_defs.is_empty() {
        let labels: Vec<String> = ally_defs
            .iter()
            .map(|u| format!("[{}] {}", u.id.0, u.name))
            .collect();

        println!("Select exactly 4 allies for your party:");
        loop {
            match MultiSelect::new("Party (pick 4):", labels.clone()).prompt() {
                Ok(chosen) if chosen.len() == 4 => {
                    selected_ids = chosen
                        .iter()
                        .map(|label| {
                            ally_defs
                                .iter()
                                .find(|u| format!("[{}] {}", u.id.0, u.name) == *label)
                                .map(|u| u.id)
                                .unwrap()
                        })
                        .collect();
                    break;
                }
                Ok(_) => {
                    println!("You must select exactly 4 allies. Try again.");
                }
                Err(_) => {
                    println!("Selection cancelled — using default party.");
                    selected_ids = ally_defs.iter().take(4).map(|u| u.id).collect();
                    break;
                }
            }
        }
        println!("Party selected: {:?}", selected_ids);
    } else {
        // Non-interactive: use the first 4 allies.
        selected_ids = ally_defs.iter().take(4).map(|u| u.id).collect();
        if !is_terminal {
            println!(
                "Non-interactive mode — using default party: {:?}",
                selected_ids
            );
        }
    }

    // --- Encounter preset selection ---
    let selected_preset: EncounterPreset;
    if is_terminal {
        let preset_options = vec![
            EncounterPreset::BossEncounter.to_string(),
            EncounterPreset::MiniBossEncounter.to_string(),
            EncounterPreset::MinionWave.to_string(),
        ];
        selected_preset = match Select::new("Choose encounter:", preset_options).prompt() {
            Ok(label) => {
                if label.contains("Mini-Boss") {
                    EncounterPreset::MiniBossEncounter
                } else if label.contains("Minion Wave") {
                    EncounterPreset::MinionWave
                } else {
                    EncounterPreset::BossEncounter
                }
            }
            Err(_) => {
                println!("Selection cancelled — defaulting to Boss Encounter.");
                EncounterPreset::BossEncounter
            }
        };
    } else {
        selected_preset = EncounterPreset::BossEncounter;
        println!(
            "Non-interactive mode — using default encounter: {}",
            EncounterPreset::BossEncounter
        );
    }

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        ))),
    )
    .add_plugins(bevyrogue::agent_tracing::log_plugin_from_env())
    .add_plugins(bevy::state::app::StatesPlugin)
    .add_plugins(AssetPlugin {
        file_path: manifest_assets_dir().to_string_lossy().into_owned(),
        ..default()
    })
    .add_plugins(DataPlugin)
    .init_resource::<TickCounter>()
    .init_resource::<TurnOrder>()
    .init_resource::<SpPool>()
    .init_resource::<ActionLog>()
    .init_resource::<CombatState>()
    .init_resource::<UltGainQueue>()
    .init_resource::<bevyrogue::combat::turn_system::EnemyTurnRequestQueue>()
    .init_resource::<PlayerActed>()
    .insert_resource(IsInteractive(is_terminal))
    .insert_resource(SelectedAllies(selected_ids))
    .insert_resource(SelectedEncounter(selected_preset))
    .add_message::<TurnAdvanced>()
    .add_message::<ActionIntent>()
    .add_message::<FollowUpIntent>()
    .add_message::<FollowUpTrace>()
    .add_message::<CombatEvent>()
    .add_message::<ActionValueUpdated>();

    if proof_mode {
        app.insert_resource(CliProofConfig {
            tick_limit: cli_proof_tick_limit_from_env(),
        })
        .init_resource::<CliProofState>();
    }

    app.add_plugins(CombatPlugin);

    app.add_systems(
        Update,
        (
            bootstrap_system,
            timeout_system,
            resolve_action_system,
            follow_up_listener_system,
            form_identity_listener_system,
            resolve_follow_up_action_system,
            ult_accumulation_system,
            flush_ult_gain_system,
            advance_turn_system,
            resolve_enemy_turn_action_system,
            check_victory_system,
            player_action_system,
            combat_dashboard_system,
            jsonl_logger_system,
            event_logger_system,
            cli_proof_system,
        )
            .chain(),
    )
    .run()
}

#[cfg(test)]
mod tests {
    use super::config::{DEFAULT_CLI_PROOF_TICK_LIMIT, parse_cli_proof_tick_limit};

    #[test]
    fn proof_tick_limit_accepts_positive_integer() {
        assert_eq!(parse_cli_proof_tick_limit(Some("42")), 42);
    }

    #[test]
    fn proof_tick_limit_falls_back_for_missing_zero_or_invalid_values() {
        assert_eq!(
            parse_cli_proof_tick_limit(None),
            DEFAULT_CLI_PROOF_TICK_LIMIT
        );
        assert_eq!(
            parse_cli_proof_tick_limit(Some("0")),
            DEFAULT_CLI_PROOF_TICK_LIMIT
        );
        assert_eq!(
            parse_cli_proof_tick_limit(Some("not-a-number")),
            DEFAULT_CLI_PROOF_TICK_LIMIT
        );
    }
}
