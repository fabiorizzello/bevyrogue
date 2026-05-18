use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use inquire::{MultiSelect, Select};
use moonshine_kind::Instance;
use std::env;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::Duration;

// Use the library modules
use bevyrogue::CombatPlugin;
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::bootstrap::{
    EncounterPreset, SelectionRequest, apply_composition, bootstrap_encounter,
};
use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::events::CombatEventKind;
use bevyrogue::combat::follow_up::{FollowUpIntent, FollowUpTrace};
use bevyrogue::combat::follow_up::{
    follow_up_listener_system, form_identity_listener_system, resolve_follow_up_action_system,
};
use bevyrogue::combat::jsonl_logger::jsonl_logger_system;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::turn_system::{
    advance_turn_system, check_victory_system, resolve_action_system,
    resolve_enemy_turn_action_system,
};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::ultimate::UltGainQueue;
use bevyrogue::combat::ultimate::{flush_ult_gain_system, ult_accumulation_system};
use bevyrogue::combat::unit::{Commander, Ko, Unit};
use bevyrogue::data::DataPlugin;
use bevyrogue::data::{
    DataReady, SkillBookHandle, UnitRosterHandle, units_ron::UnitDef, units_ron::UnitRoster,
};

use bevyrogue::combat::action_query::{
    ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, TargetAffordance,
    TargetStatus, build_snapshot_from_ecs_with_sp, first_enabled_target_id, query_action_affordance,
};
use bevyrogue::combat::av::{ActionValue, ActionValueUpdated, MAX_AV};
use bevyrogue::combat::energy::{Energy, RoundEnergyTracker};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::resistance::{apply_advance, apply_delay};
use bevyrogue::combat::resolution::{TargetEntry, TargetableSnapshot, resolve_targets};
use bevyrogue::combat::stun::Stunned;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::data::skills_ron::SkillBook;
use bevyrogue::data::skills_ron::TargetShape;

const DEFAULT_CLI_PROOF_TICK_LIMIT: u32 = 120;

#[derive(Resource, Default)]
struct TickCounter(u32);

/// Tracks whether a player action was just submitted and the next WaitingAction
/// tick should advance the turn instead of prompting again.
#[derive(Resource, Default)]
struct PlayerActed(bool);

/// Set once terminal mode is determined so we don't re-check stdin every tick.
#[derive(Resource)]
struct IsInteractive(bool);

/// The 4 ally UnitIds chosen by the player at startup.
#[derive(Resource)]
struct SelectedAllies(Vec<UnitId>);

/// The encounter preset chosen by the player at startup (or defaulted to BossEncounter in CI).
#[derive(Resource)]
struct SelectedEncounter(EncounterPreset);

/// Env-gated proof-mode settings. This is an observation/exit surface only;
/// gameplay still flows through the shared ECS systems registered below.
#[derive(Resource, Debug, Clone, Copy)]
struct CliProofConfig {
    tick_limit: u32,
}

#[derive(Resource, Debug, Default)]
struct CliProofState {
    ticks: u32,
    finished: bool,
}

fn cli_proof_enabled() -> bool {
    env::var("BEVYROGUE_CLI_PROOF")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn parse_cli_proof_tick_limit(raw: Option<&str>) -> u32 {
    raw.and_then(|value| value.parse::<u32>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_CLI_PROOF_TICK_LIMIT)
}

fn cli_proof_tick_limit_from_env() -> u32 {
    parse_cli_proof_tick_limit(env::var("BEVYROGUE_CLI_TICK_LIMIT").ok().as_deref())
}

fn timeout_system(
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

fn bootstrap_system(
    mut commands: Commands,
    data_ready: Option<Res<DataReady>>,
    roster_handle: Option<Res<UnitRosterHandle>>,
    rosters: Res<Assets<UnitRoster>>,
    selected_allies: Res<SelectedAllies>,
    selected_encounter: Res<SelectedEncounter>,
    mut order: ResMut<TurnOrder>,
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
            apply_composition(&mut commands, &composition, &mut order);
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
                    unit_ids: order.next_unit.map(|id| vec![id]).unwrap_or_default(),
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

fn event_logger_system(mut events: MessageReader<CombatEvent>) {
    for event in events.read() {
        println!(
            "  [EVENT] {:?} (source: {:?}, target: {:?}, depth: {})",
            event.kind, event.source, event.target, event.follow_up_depth
        );
    }
}

fn combat_dashboard_system(
    mut turn_advanced: MessageReader<TurnAdvanced>,
    mut combat_events: MessageReader<CombatEvent>,
    units: Query<(&Unit, &Team, Option<&UltimateCharge>, Option<&Toughness>)>,
    turn_order: Res<TurnOrder>,
    sp_pool: Res<SpPool>,
) {
    let mut should_draw = false;
    for _ in turn_advanced.read() {
        should_draw = true;
    }
    for event in combat_events.read() {
        if matches!(
            event.kind,
            CombatEventKind::OnActionResolved | CombatEventKind::TurnOrderSeeded { .. }
        ) {
            should_draw = true;
        }
    }

    if !should_draw {
        return;
    }

    println!("\n{}", "=".repeat(60));
    println!(
        " COMBAT DASHBOARD | SP: {}/{}",
        sp_pool.current, sp_pool.max
    );
    println!("{}", "-".repeat(60));

    // Turn Order (AV system: show active unit)
    let mut names = Vec::new();
    if let Some(active_id) = turn_order.active_unit {
        if let Some((unit, _, _, _)) = units.iter().find(|(u, _, _, _)| u.id == active_id) {
            names.push(unit.name.clone());
        }
    }
    println!(" TURN ORDER: {}", names.join(" -> "));
    println!("{}", "-".repeat(60));

    // Units
    let mut all_units: Vec<_> = units.iter().collect();
    all_units.sort_by_key(|(_, team, _, _)| match team {
        Team::Ally => 0,
        Team::Enemy => 1,
    });

    for (unit, team, ult, toughness) in all_units {
        let team_str = match team {
            Team::Ally => "[ALLY]",
            Team::Enemy => "[ENEM]",
        };

        let ult_str = if let Some(u) = ult {
            format!("ULT: {}/{}", u.current, u.trigger)
        } else {
            "ULT: N/A".to_string()
        };

        let toughness_str = if let Some(t) = toughness {
            format!("TGH: {}/{}", t.current, t.max)
        } else {
            "TGH: N/A".to_string()
        };

        println!(
            "{:<6} {:<12} | HP: {:>4}/{:<4} | {} | {}",
            team_str, unit.name, unit.hp_current, unit.hp_max, ult_str, toughness_str
        );
    }
    println!("{}\n", "=".repeat(60));
}

/// Interactive action selection: presents inquire menus when it is an ally's
/// turn and stdin is a terminal.  In non-interactive mode it emits a default
/// BasicAttack so the CI verification loop can still run.
///
/// State machine driven by `PlayerActed`:
///   false → prompt (or auto-act) then set true
///   true  → action was resolved; advance turn queue and reset to false
#[derive(Clone)]
struct ActionMenuEntry<'a> {
    kind: ActionQueryKind<'a>,
    label: String,
    affordance: ActionAffordance<'a>,
}

fn action_kind_label(kind: ActionQueryKind<'_>) -> String {
    match kind {
        ActionQueryKind::Basic => "Basic Attack".to_string(),
        ActionQueryKind::Skill(skill_id) => format!("Skill: {}", skill_id.0),
        ActionQueryKind::Ultimate => "Ultimate".to_string(),
    }
}

fn action_status_label(status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => "enabled".to_string(),
        ActionStatus::Disabled { reason } => format!("disabled({reason:?})"),
        ActionStatus::Deferred { reason } => format!("deferred({reason:?})"),
        ActionStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

fn target_status_label(status: &TargetStatus) -> String {
    match status {
        TargetStatus::Enabled => "enabled".to_string(),
        TargetStatus::Disabled { reason } => format!("disabled({reason:?})"),
        TargetStatus::Deferred { reason } => format!("deferred({reason:?})"),
        TargetStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

fn action_entry_label(entry: &ActionMenuEntry<'_>) -> String {
    format!(
        "{} [{}]",
        action_kind_label(entry.kind),
        action_status_label(&entry.affordance.action),
    )
}

fn target_entry_label(unit: &Unit, team: &Team, affordance: &TargetAffordance) -> String {
    let side = match team {
        Team::Ally => "ALLY",
        Team::Enemy => "ENEM",
    };

    format!(
        "[{side}] {} HP:{}/{} [{}]",
        unit.name,
        unit.hp_current,
        unit.hp_max,
        target_status_label(&affordance.status)
    )
}

fn build_action_entries<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    actor_skills: Option<&'a UnitSkills>,
) -> Vec<ActionMenuEntry<'a>> {
    let mut entries = Vec::new();

    entries.push(ActionMenuEntry {
        kind: ActionQueryKind::Basic,
        label: String::new(),
        affordance: query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Basic),
    });

    if let Some(skills) = actor_skills {
        for skill_id in &skills.skills {
            let kind = ActionQueryKind::Skill(skill_id);
            entries.push(ActionMenuEntry {
                kind,
                label: String::new(),
                affordance: query_action_affordance(snapshot, skill_book, actor_id, kind),
            });
        }
    }

    entries.push(ActionMenuEntry {
        kind: ActionQueryKind::Ultimate,
        label: String::new(),
        affordance: query_action_affordance(
            snapshot,
            skill_book,
            actor_id,
            ActionQueryKind::Ultimate,
        ),
    });

    for entry in &mut entries {
        entry.label = action_entry_label(entry);
    }

    entries
}

fn print_action_entries(entries: &[ActionMenuEntry<'_>]) {
    println!("\n  Action affordances:");
    for entry in entries {
        println!("    - {}", entry.label);
    }
}

fn print_target_entries(target_entries: &[(UnitId, String)]) {
    println!("\n  Target affordances:");
    for (_, label) in target_entries {
        println!("    - {}", label);
    }
}

fn player_action_system(
    state: Res<CombatState>,
    mut player_acted: ResMut<PlayerActed>,
    mut order: ResMut<TurnOrder>,
    mut intent_writer: MessageWriter<ActionIntent>,
    units: Query<(
        &Unit,
        &Team,
        Option<&UltimateCharge>,
        Option<&UnitSkills>,
        Option<&Ko>,
        Option<&Commander>,
        Option<&Toughness>,
        Option<&bevyrogue::combat::counterplay::EnemyCounterplayKit>,
        Option<&Stunned>,
        Option<&Energy>,
        Option<&RoundEnergyTracker>,
    )>,
    sp_pool: Res<SpPool>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    interactive: Res<IsInteractive>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.phase == CombatPhase::Victory {
        println!("\n[VICTORY] All enemies defeated!");
        exit.write(AppExit::Success);
        return;
    }
    if state.phase == CombatPhase::Defeat {
        println!("\n[DEFEAT] All allies fallen!");
        exit.write(AppExit::Success);
        return;
    }
    if state.phase != CombatPhase::WaitingAction {
        return;
    }

    if player_acted.0 {
        player_acted.0 = false;
        order.active_unit = None;
        return;
    }

    let Some(actor_id) = order.active_unit else {
        return;
    };

    let Some((
        actor_unit,
        _actor_team,
        _actor_ult,
        actor_skills,
        actor_ko,
        _actor_commander,
        _,
        _,
        _,
        _,
        _,
    )) = units
        .iter()
        .find(|(u, _, _, _, _, _, _, _, _, _, _)| u.id == actor_id)
    else {
        return;
    };

    if actor_ko.is_some() {
        order.active_unit = None;
        return;
    }

    let Some(skill_book) = skill_book_handle
        .as_ref()
        .and_then(|handle| skill_books.get(&handle.0))
    else {
        println!("[QUERY] Skill book loading; waiting for shared SkillBook asset.");
        return;
    };

    let units_data: Vec<_> = units
        .iter()
        .map(
            |(
                unit,
                team,
                ult,
                skills,
                ko,
                commander,
                toughness,
                counterplay,
                stunned,
                energy,
                tracker,
            )| {
                (
                    unit.id,
                    *team,
                    unit,
                    skills,
                    ult,
                    toughness,
                    counterplay,
                    ko.is_some(),
                    stunned.is_some(),
                    commander.is_some(),
                    energy,
                    tracker,
                )
            },
        )
        .collect();

    let snapshot = build_snapshot_from_ecs_with_sp(
        &state,
        &order,
        sp_pool.current,
        actor_id,
        actor_id,
        units_data,
    );

    let action_entries = build_action_entries(&snapshot, skill_book, actor_id, actor_skills);
    print_action_entries(&action_entries);

    let basic_entry = action_entries
        .iter()
        .find(|entry| matches!(entry.kind, ActionQueryKind::Basic));

    if !interactive.0 {
        let preferred_entry = action_entries
            .iter()
            .find(|entry| {
                matches!(entry.affordance.action, ActionStatus::Enabled)
                    && matches!(entry.kind, ActionQueryKind::Skill(_))
            })
            .or_else(|| {
                action_entries.iter().find(|entry| {
                    matches!(entry.affordance.action, ActionStatus::Enabled)
                        && matches!(entry.kind, ActionQueryKind::Ultimate)
                })
            })
            .or_else(|| {
                action_entries.iter().find(|entry| {
                    matches!(entry.affordance.action, ActionStatus::Enabled)
                        && matches!(entry.kind, ActionQueryKind::Basic)
                })
            });

        if let Some(entry) = preferred_entry {
            if let Some(target_id) = first_enabled_target_id(&entry.affordance) {
                let intent = match entry.kind {
                    ActionQueryKind::Basic => ActionIntent::Basic {
                        attacker: actor_id,
                        target: target_id,
                    },
                    ActionQueryKind::Skill(skill_id) => {
                        println!("[CLI_PROOF] OnSkillCast intent skill_id={}", skill_id.0);
                        ActionIntent::Skill {
                            attacker: actor_id,
                            skill_id: skill_id.clone(),
                            target: target_id,
                        }
                    }
                    ActionQueryKind::Ultimate => {
                        println!("[CLI_PROOF] OnSkillCast intent skill_id=ultimate");
                        ActionIntent::Ultimate {
                            attacker: actor_id,
                            target: target_id,
                        }
                    }
                };
                intent_writer.write(intent);
                player_acted.0 = true;
                return;
            }
            println!(
                "[QUERY] Auto-selected action has no enabled target: {}",
                action_status_label(&entry.affordance.action)
            );
        } else if let Some(entry) = basic_entry {
            println!(
                "[QUERY] Basic Attack unavailable: {}",
                action_status_label(&entry.affordance.action)
            );
        }

        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    let enabled_actions: Vec<_> = action_entries
        .iter()
        .filter(|entry| matches!(entry.affordance.action, ActionStatus::Enabled))
        .collect();

    if enabled_actions.is_empty() {
        if let Some(entry) = basic_entry {
            println!(
                "[QUERY] No enabled actions; Basic Attack state: {}",
                action_status_label(&entry.affordance.action)
            );
        } else {
            println!("[QUERY] No enabled actions available.");
        }
        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    println!(
        "\n>>> {}'s turn  (SP: {}/{})",
        actor_unit.name, sp_pool.current, sp_pool.max
    );
    let enabled_action_labels: Vec<String> = enabled_actions
        .iter()
        .map(|entry| entry.label.clone())
        .collect();
    let selected_label = match Select::new("Action:", enabled_action_labels.clone()).prompt() {
        Ok(label) => label,
        Err(_) => {
            println!("Input cancelled — defaulting to first enabled action.");
            enabled_action_labels[0].clone()
        }
    };

    let selected_entry = enabled_actions
        .iter()
        .find(|entry| entry.label == selected_label)
        .copied()
        .unwrap_or(enabled_actions[0]);

    let target_entries: Vec<(UnitId, String)> = selected_entry
        .affordance
        .targets
        .iter()
        .filter(|(_, affordance)| matches!(affordance.status, TargetStatus::Enabled))
        .filter_map(|(target_id, affordance)| {
            units
                .iter()
                .find(|(unit, _, _, _, _, _, _, _, _, _, _)| unit.id == *target_id)
                .map(|(unit, team, _, _, _, _, _, _, _, _, _)| {
                    (*target_id, target_entry_label(unit, team, affordance))
                })
        })
        .collect();

    print_target_entries(&target_entries);

    if target_entries.is_empty() {
        println!(
            "[QUERY] No enabled targets for {}: {}",
            action_kind_label(selected_entry.kind),
            action_status_label(&selected_entry.affordance.action)
        );
        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    let enabled_target_labels: Vec<String> = target_entries
        .iter()
        .map(|(_, label)| label.clone())
        .collect();

    let selected_target_label = match Select::new("Target:", enabled_target_labels.clone()).prompt()
    {
        Ok(label) => label,
        Err(_) => {
            println!("Input cancelled — defaulting to first enabled target.");
            enabled_target_labels[0].clone()
        }
    };

    let target_id = target_entries
        .iter()
        .find(|(_, label)| *label == selected_target_label)
        .map(|(id, _)| *id)
        .unwrap_or(target_entries[0].0);

    let intent = match selected_entry.kind {
        ActionQueryKind::Basic => ActionIntent::Basic {
            attacker: actor_id,
            target: target_id,
        },
        ActionQueryKind::Skill(skill_id) => ActionIntent::Skill {
            attacker: actor_id,
            skill_id: skill_id.clone(),
            target: target_id,
        },
        ActionQueryKind::Ultimate => ActionIntent::Ultimate {
            attacker: actor_id,
            target: target_id,
        },
    };

    intent_writer.write(intent);
    player_acted.0 = true;
}

fn cli_proof_system(world: &mut World) {
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

fn manifest_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn manifest_asset_path(relative_path: &str) -> PathBuf {
    manifest_assets_dir().join(relative_path)
}

fn verify_required_data_assets() -> Result<(), String> {
    // Check that per-digimon asset directories exist instead of monolithic files.
    for relative_path in [
        "data/digimon/agumon/unit.ron",
        "data/digimon/gabumon/unit.ron",
        "data/party.ron",
    ] {
        let path = manifest_asset_path(relative_path);
        if !path.is_file() {
            return Err(format!("required data asset missing: {}", path.display()));
        }
    }
    Ok(())
}

fn load_ally_roster() -> Result<Vec<UnitDef>, String> {
    let roster = bevyrogue::data::aggregate_unit_roster();
    Ok(roster
        .0
        .into_iter()
        .filter(|u| u.team == bevyrogue::combat::team::Team::Ally)
        .collect())
}

/// Standalone scenario: advance-delay-cap
///
/// Demonstrates AdvanceTurn / DelayTurn cap (≤50%) and AV clamp [0, 2*MAX_AV].
/// Prints a human-readable AV gauge step-by-step and emits JSONL to stdout.
/// Does not start Bevy — exits after printing.
fn run_advance_delay_cap_scenario() {
    println!("=== scenario: advance-delay-cap ===");
    println!("MAX_AV = {MAX_AV}");

    struct MockUnit {
        name: &'static str,
        av: ActionValue,
    }

    let mut units = [
        MockUnit {
            name: "Agumon",
            av: ActionValue(0),
        },
        MockUnit {
            name: "Gabumon",
            av: ActionValue(MAX_AV / 2),
        },
    ];

    #[derive(serde::Serialize)]
    struct JsonlEntry {
        kind: &'static str,
        target: &'static str,
        amount_pct_requested: u32,
        amount_pct_capped: u32,
        av_pre: i32,
        av_delta: i32,
        av_post: i32,
    }

    let steps: &[(&str, usize, u32)] = &[
        ("AdvanceTurn", 0, 50),
        ("AdvanceTurn", 0, 50),
        ("DelayTurn", 1, 80),
        ("DelayTurn", 1, 50),
    ];

    for (kind, unit_idx, amount_pct) in steps {
        let unit = &mut units[*unit_idx];
        let av_pre = unit.av.0;
        let capped = (*amount_pct).min(50);

        let delta = if *kind == "AdvanceTurn" {
            apply_advance(&mut unit.av, *amount_pct)
        } else {
            apply_delay(&mut unit.av, *amount_pct, None)
        };

        let av_post = unit.av.0;
        let bar_width = 40usize;
        let fill = ((av_post.max(0) as f64 / (2.0 * MAX_AV as f64)) * bar_width as f64) as usize;
        let bar: String =
            "#".repeat(fill.min(bar_width)) + &".".repeat(bar_width - fill.min(bar_width));

        println!(
            "[{kind}] {name} pct={amount_pct}(cap={capped}) AV {av_pre:>6} → {av_post:>6} (Δ{delta:+}) [{bar}]",
            name = unit.name,
        );

        let entry = JsonlEntry {
            kind,
            target: unit.name,
            amount_pct_requested: *amount_pct,
            amount_pct_capped: capped,
            av_pre,
            av_delta: delta,
            av_post,
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
    }

    println!("=== scenario complete ===");
}

/// Standalone scenario: aoe-blast
///
/// Builds a deterministic 3-enemy encounter, casts one Blast skill (slot-1 primary)
/// then one AllEnemies skill. Prints resolved target list + per-target damage,
/// final HP per enemy, and emits one JSONL line per OnDamageDealt event.
fn run_aoe_blast_scenario() {
    println!("=== scenario: aoe-blast ===");

    #[derive(Clone)]
    struct MockUnit {
        id: UnitId,
        name: &'static str,
        slot_index: u8,
        hp: i32,
    }

    #[derive(serde::Serialize)]
    struct JsonlDamageEvent {
        event: &'static str,
        source_id: u32,
        target_id: u32,
        target_slot: u8,
        amount: i32,
        skill_id: &'static str,
    }

    let attacker_id = UnitId(0);
    let mut enemies = vec![
        MockUnit {
            id: UnitId(1),
            name: "GobA",
            slot_index: 0,
            hp: 60,
        },
        MockUnit {
            id: UnitId(2),
            name: "GobB",
            slot_index: 1,
            hp: 60,
        },
        MockUnit {
            id: UnitId(3),
            name: "GobC",
            slot_index: 2,
            hp: 60,
        },
    ];

    let build_snapshot = |units: &[MockUnit]| TargetableSnapshot {
        entries: units
            .iter()
            .map(|u| TargetEntry {
                id: u.id,
                team: Team::Enemy,
                slot_index: u.slot_index,
                alive: u.hp > 0,
                hp_per_mille: 1000, // mock: full HP for shape-only scenarios
            })
            .collect(),
    };

    // --- Cast 1: Blast on slot-1 primary (GobB) ---
    println!("\n--- Cast 1: blast_demo (Blast, primary=GobB slot-1) ---");
    let primary_blast = UnitId(2);
    let snapshot1 = build_snapshot(&enemies);
    let blast_targets = resolve_targets(&TargetShape::Blast, primary_blast, &snapshot1);

    let blast_damage_per_target = 20i32;
    let blast_toughness_per_target = 10i32;
    println!(
        "Resolved targets (slot_index asc): {:?}",
        blast_targets
            .iter()
            .filter_map(|id| enemies.iter().find(|u| u.id == *id))
            .map(|u| format!("{}(slot{})", u.name, u.slot_index))
            .collect::<Vec<_>>()
    );

    for &target_id in &blast_targets {
        let enemy = enemies.iter_mut().find(|u| u.id == target_id).unwrap();
        enemy.hp -= blast_damage_per_target;
        println!(
            "  {} (slot {}) takes {} dmg → HP {}",
            enemy.name, enemy.slot_index, blast_damage_per_target, enemy.hp
        );
        let entry = JsonlDamageEvent {
            event: "OnDamageDealt",
            source_id: attacker_id.0,
            target_id: enemy.id.0,
            target_slot: enemy.slot_index,
            amount: blast_damage_per_target,
            skill_id: "blast_demo",
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
        let _ = blast_toughness_per_target;
    }

    // --- Cast 2: AllEnemies (aoe_demo) ---
    println!("\n--- Cast 2: aoe_demo (AllEnemies) ---");
    let primary_all = UnitId(1);
    let snapshot2 = build_snapshot(&enemies);
    let all_targets = resolve_targets(&TargetShape::AllEnemies, primary_all, &snapshot2);

    let all_damage_per_target = 15i32;
    let all_toughness_per_target = 8i32;
    println!(
        "Resolved targets (slot_index asc): {:?}",
        all_targets
            .iter()
            .filter_map(|id| enemies.iter().find(|u| u.id == *id))
            .map(|u| format!("{}(slot{})", u.name, u.slot_index))
            .collect::<Vec<_>>()
    );

    for &target_id in &all_targets {
        let enemy = enemies.iter_mut().find(|u| u.id == target_id).unwrap();
        enemy.hp -= all_damage_per_target;
        println!(
            "  {} (slot {}) takes {} dmg → HP {}",
            enemy.name, enemy.slot_index, all_damage_per_target, enemy.hp
        );
        let entry = JsonlDamageEvent {
            event: "OnDamageDealt",
            source_id: attacker_id.0,
            target_id: enemy.id.0,
            target_slot: enemy.slot_index,
            amount: all_damage_per_target,
            skill_id: "aoe_demo",
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
        let _ = all_toughness_per_target;
    }

    // --- Final HP gauge ---
    println!("\n--- Final HP ---");
    for enemy in &enemies {
        println!(
            "  {} (slot {}): HP {}",
            enemy.name, enemy.slot_index, enemy.hp
        );
    }
    println!("=== scenario complete ===");
}

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
    .add_plugins(bevy::log::LogPlugin::default())
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
    use super::*;

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
