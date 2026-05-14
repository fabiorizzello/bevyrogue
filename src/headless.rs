//! Headless app wiring (default build, no winit/wgpu/egui).
//!
//! Drives a deterministic smoke run via `CombatScript` for the first
//! `HEADLESS_TICK_BUDGET` ticks, then exits cleanly.

use std::{collections::VecDeque, time::Duration};

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use moonshine_kind::Instance;

use crate::combat::bootstrap::{
    EncounterPreset, SelectionRequest, apply_composition, bootstrap_encounter,
};
use crate::combat::api::intent::CastId;
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::follow_up::{
    follow_up_listener_system, form_identity_listener_system, resolve_follow_up_action_system,
};
use crate::combat::jsonl_logger::jsonl_logger_system;
use crate::combat::log::ActionLog;
use crate::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use crate::combat::rng::CombatRng;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState};
use crate::combat::stun::Stunned;
use crate::combat::team::Team;
use crate::combat::toughness::{Toughness, visible_toughness};
use crate::combat::turn_order::{TurnAdvanced, TurnOrder};
use crate::combat::turn_system::{
    ActionIntent, advance_turn_system, apply_av_ops_system, check_victory_system,
    resolve_action_system,
};
use crate::combat::types::UnitId;
use crate::combat::ultimate::{UltimateCharge, flush_ult_gain_system, ult_accumulation_system};
use crate::combat::unit::{Ko, Unit};
use crate::data::{self, DataPlugin};
use crate::party_validation;

/// Tick budget for headless smoke runs. ~2 seconds at 60 Hz — enough for
/// asset loader init and first-frame ECS snapshots, short enough that
/// `cargo run` exits cleanly without SIGTERM.
const HEADLESS_TICK_BUDGET: u32 = 120;

#[derive(Resource, Default)]
struct TickCounter(u32);

#[derive(Resource, Default)]
struct ValidationSnapshotLogged(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScriptStep {
    EmitIntent(ActionIntent),
    ReloadAssets,
    Exit,
}

#[derive(Resource, Debug, Clone)]
struct CombatScript {
    actions: VecDeque<ScriptStep>,
    pause_ticks: u8,
}

impl Default for CombatScript {
    fn default() -> Self {
        Self {
            actions: default_headless_script(),
            pause_ticks: 0,
        }
    }
}

impl CombatScript {
    fn pop(&mut self) -> Option<ScriptStep> {
        self.actions.pop_front()
    }

    fn peek(&self) -> Option<&ScriptStep> {
        self.actions.front()
    }
}

fn default_headless_script() -> VecDeque<ScriptStep> {
    use ScriptStep::{EmitIntent, Exit, ReloadAssets};

    let mut actions = VecDeque::new();
    for _ in 0..5 {
        actions.push_back(EmitIntent(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(4),
        }));
    }
    actions.push_back(EmitIntent(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(4),
    }));
    for _ in 0..6 {
        actions.push_back(EmitIntent(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(5),
        }));
    }
    actions.push_back(ReloadAssets);
    actions.push_back(Exit);
    actions
}

pub fn register(app: &mut App) {
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    )
    .add_plugins(bevy::log::LogPlugin::default())
    .add_plugins(bevy::state::app::StatesPlugin)
    .add_plugins(AssetPlugin {
        watch_for_changes_override: Some(true),
        ..default()
    })
    .add_plugins(DataPlugin)
    .init_resource::<TickCounter>()
    .init_resource::<CombatScript>()
    .init_resource::<ValidationSnapshotLogged>()
    .init_resource::<CombatRng>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        headless_validation_snapshot_once.before(headless_smoke_tick),
    )
    .add_systems(Update, headless_smoke_tick);
}

pub fn register_combat_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            resolve_action_system,
            follow_up_listener_system,
            form_identity_listener_system,
            resolve_follow_up_action_system,
            ult_accumulation_system,
            flush_ult_gain_system,
            apply_av_ops_system,
            advance_turn_system,
            check_victory_system,
            jsonl_logger_system,
        )
            .chain()
            .after(headless_smoke_tick),
    );
}

fn setup() {}

#[derive(bevy::ecs::system::SystemParam)]
struct BootstrapParams<'w> {
    rosters: Res<'w, Assets<data::units_ron::UnitRoster>>,
    roster_handle: Option<Res<'w, data::UnitRosterHandle>>,
    parties: Res<'w, Assets<data::party_ron::PartyConfig>>,
    party_handle: Option<Res<'w, data::PartyConfigHandle>>,
    combat_events: MessageWriter<'w, CombatEvent>,
}

type HeadlessUnitsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Unit,
        &'static Team,
        Option<&'static Toughness>,
        &'static UltimateCharge,
        Option<&'static Ko>,
        Option<&'static Stunned>,
    ),
>;

fn headless_validation_snapshot_once(world: &mut World) {
    if !world.contains_resource::<data::DataReady>() {
        return;
    }

    if world.resource::<ValidationSnapshotLogged>().0 {
        return;
    }

    match capture_validation_snapshot(world) {
        Ok(snapshot) => info!(
            "validation_snapshot: {}",
            format_validation_snapshot(&snapshot)
        ),
        Err(err) => error!("validation_snapshot_error: {err}"),
    }

    world.resource_mut::<ValidationSnapshotLogged>().0 = true;
}

#[allow(clippy::too_many_arguments)]
fn headless_smoke_tick(
    mut commands: Commands,
    data_ready: Option<Res<data::DataReady>>,
    mut counter: ResMut<TickCounter>,
    mut exit: MessageWriter<AppExit>,
    mut order: ResMut<TurnOrder>,
    _turn_advanced: MessageWriter<TurnAdvanced>,
    mut action_intent: MessageWriter<ActionIntent>,
    mut script: ResMut<CombatScript>,
    asset_server: Res<AssetServer>,
    mut combat_state: ResMut<CombatState>,
    mut sp: ResMut<SpPool>,
    mut log: ResMut<ActionLog>,
    unit_entities: Query<Instance<Unit>>,
    units: HeadlessUnitsQuery,
    mut bootstrap: BootstrapParams,
) {
    if data_ready.is_none() {
        return;
    }

    if units.is_empty() {
        let Some(rhandle) = bootstrap.roster_handle else {
            return;
        };
        let Some(roster) = bootstrap.rosters.get(&rhandle.0) else {
            return;
        };
        let Some(phandle) = bootstrap.party_handle else {
            return;
        };
        let Some(pcfg) = bootstrap.parties.get(&phandle.0) else {
            return;
        };

        if let Err(err) = party_validation::validate_party_config(pcfg) {
            error!("party.ron: {err}");
            exit.write(AppExit::error());
            return;
        }

        let request = SelectionRequest {
            rookie_ids: pcfg.ally_ids.to_vec(),
        };

        match bootstrap_encounter(roster, &request, EncounterPreset::BossEncounter) {
            Ok(composition) => {
                apply_composition(&mut commands, &composition, &mut order);
                info!(
                    "bootstrap: success, selected_party={:?}",
                    request.rookie_ids
                );
                bootstrap.combat_events.write(CombatEvent {
                    source: UnitId(0),
                    target: UnitId(0),
                    kind: CombatEventKind::PartySelected {
                        ally_ids: pcfg.ally_ids.to_vec(),
                        tamer_id: pcfg.tamer_id,
                    },
                    follow_up_depth: 0,
                    cast_id: CastId::ROOT,
                });
                bootstrap.combat_events.write(CombatEvent {
                    source: UnitId(0),
                    target: UnitId(0),
                    kind: CombatEventKind::TurnOrderSeeded {
                        unit_ids: order.next_unit.map(|id| vec![id]).unwrap_or_default(),
                    },
                    follow_up_depth: 0,
                    cast_id: CastId::ROOT,
                });
            }
            Err(err) => {
                error!("bootstrap_error={:?}", err);
                exit.write(AppExit::error());
            }
        }
        return;
    }

    counter.0 += 1;
    if counter.0 == 1 {
        info!(
            "headless smoke run: roster snapshot (EguiPrimaryContextPass roster_panel system is windowed-only)"
        );
        info!(
            "sp pool {}/{}, log {}",
            sp.current,
            sp.max,
            if log.events.is_empty() {
                "empty"
            } else {
                "non-empty"
            }
        );
        for (u, team, tough, ult, _, _) in &units {
            info!(
                "  {} — {:?} — HP {}/{} — res: {:?}",
                u.name, u.attribute, u.hp_current, u.hp_max, u.resists
            );
            match visible_toughness(*team, tough.as_deref()) {
                Some(view) => info!(
                    "  team={:?} tough={}/{} weak={:?} ult={}/{}->{}",
                    team,
                    view.current,
                    view.max,
                    view.weaknesses,
                    ult.current,
                    ult.trigger,
                    ult.cap
                ),
                None => info!(
                    "  team={:?} tough=N/A weak=N/A ult={}/{}->{}",
                    team, ult.current, ult.trigger, ult.cap
                ),
            }
        }
    }
    if counter.0 == 2 {
        info!(
            "turn order seeded: active={:?} next={:?}",
            order.active_unit, order.next_unit
        );
    }

    if counter.0 >= 5 {
        if script.pause_ticks > 0 {
            script.pause_ticks -= 1;
        } else {
            if matches!(
                combat_state.phase,
                CombatPhase::Victory | CombatPhase::Defeat
            ) {
                while matches!(script.peek(), Some(ScriptStep::EmitIntent(_))) {
                    debug!("script step skipped after terminal state: EmitIntent");
                    let _ = script.pop();
                }

                match script.peek().cloned() {
                    Some(ScriptStep::ReloadAssets) => {
                        let _ = script.pop();
                        debug!("script step: ReloadAssets");
                        asset_server.reload("data/units.ron");
                        for unit in &unit_entities {
                            commands.entity(unit.entity()).despawn();
                        }
                        *order = TurnOrder::default();
                        combat_state.reset();
                        log.events.clear();
                        *sp = SpPool::default();
                        script.pause_ticks = 2;
                        info!("restart: roster reloaded");
                    }
                    Some(ScriptStep::Exit) => {
                        let _ = script.pop();
                        debug!("script step: Exit");
                        exit.write(AppExit::Success);
                        return;
                    }
                    Some(ScriptStep::EmitIntent(_)) => unreachable!(),
                    None => {
                        error!("script exhausted without terminal state");
                        exit.write(AppExit::error());
                        return;
                    }
                }
            } else if script.actions.is_empty() {
                error!("script exhausted without terminal state");
                exit.write(AppExit::error());
                return;
            } else if combat_state.phase == CombatPhase::WaitingAction {
                match script.peek().cloned() {
                    Some(ScriptStep::EmitIntent(intent)) if counter.0 % 2 == 1 => {
                        let _ = script.pop();
                        debug!("script step: EmitIntent");
                        action_intent.write(intent);
                    }
                    _ => {}
                }
            }
        }
    }

    if counter.0 >= HEADLESS_TICK_BUDGET {
        info!(
            "headless tick budget ({}) reached, exiting cleanly",
            HEADLESS_TICK_BUDGET
        );
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_script_advance_yields_steps_in_order() {
        let mut script = CombatScript {
            actions: VecDeque::from([
                ScriptStep::EmitIntent(ActionIntent::Basic {
                    attacker: UnitId(1),
                    target: UnitId(4),
                }),
                ScriptStep::ReloadAssets,
                ScriptStep::Exit,
            ]),
            pause_ticks: 0,
        };

        assert_eq!(
            script.pop(),
            Some(ScriptStep::EmitIntent(ActionIntent::Basic {
                attacker: UnitId(1),
                target: UnitId(4),
            }))
        );
        assert_eq!(script.pop(), Some(ScriptStep::ReloadAssets));
        assert_eq!(script.pop(), Some(ScriptStep::Exit));
        assert_eq!(script.pop(), None);
    }
}
