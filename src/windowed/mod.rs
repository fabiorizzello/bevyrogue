//! Windowed app wiring (feature `windowed`): winit + wgpu + bevy_ui + egui.
//!
//! Provides the egui combat panel, roster/turn-order side panels, and an
//! optional validation tick that exits cleanly after a soak window.

mod digimon;
mod render;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use bevyrogue::animation::AnimationAssetPlugin;
use bevyrogue::combat::av::ActionValue;
use bevyrogue::combat::bootstrap::{
    EncounterPreset, SelectionRequest, apply_composition, bootstrap_encounter,
};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::follow_up::{
    follow_up_listener_system, form_identity_listener_system, resolve_follow_up_action_system,
};
use bevyrogue::combat::observability::{
    FrameTimeAccumulator, capture_validation_snapshot, format_frame_time_stats,
    format_validation_snapshot, parse_validation_baseline_toggle,
};
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::runtime::{Clock, SuspendedTimelineState, TimelineClock};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::{
    advance_turn_system, burst_action_system, check_victory_system,
    continue_suspended_timeline_system, resolve_action_system, resolve_enemy_turn_action_system,
};
use bevyrogue::combat::types::{Attribute, UnitId};
use bevyrogue::combat::ultimate::{flush_ult_gain_system, ult_accumulation_system};
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::{self, DataPlugin};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowedValidationConfig {
    soak_secs: u64,
    /// When true, run a kernel-only baseline (anim-graph/render path disabled) so
    /// the soak's frame-time stats isolate the anim-graph cost. Drives the
    /// `mode=full|baseline` field of the `validation_frametime:` line.
    baseline: bool,
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
struct WindowedValidationState {
    started_at_secs: Option<f32>,
    snapshot_logged: bool,
    finished: bool,
    /// Per-frame deltas accumulated after the soak start frame. The exact pure
    /// aggregator unit-proven in T01 (`bevyrogue::combat::observability`).
    frame_times: FrameTimeAccumulator,
}

impl WindowedValidationState {
    /// Record one presentation-frame delta (seconds) into the soak accumulator.
    fn record_frame(&mut self, delta_secs: f32) {
        self.frame_times.push(delta_secs);
    }
}

/// Egui panels: roster, turn order, combat panel.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<bevyrogue::ui::cues::CueRegistry>()
            .init_resource::<bevyrogue::ui::combat_panel::PendingAction>()
            .init_resource::<bevyrogue::ui::combat_panel::PreviewDamageCache>()
            .init_resource::<bevyrogue::ui::combat_panel::BabyBurnerFlashState>()
            .init_resource::<bevyrogue::ui::combat_panel::HpBarView>()
            .init_resource::<bevyrogue::ui::combat_panel::FloatingDamageView>()
            .init_resource::<bevyrogue::ui::combat_panel::TargetHurtState>()
            .init_resource::<bevyrogue::ui::combat_panel::TwinCoreBadgeState>()
            .init_resource::<bevyrogue::ui::phase_strip::PhaseStripDisplay>()
            .add_systems(
                Update,
                (
                    bevyrogue::ui::combat_panel::advance_baby_burner_flash_state,
                    bevyrogue::ui::combat_panel::refresh_preview_damage_cache,
                    bevyrogue::ui::combat_panel::observe_baby_burner_flash,
                    bevyrogue::ui::combat_panel::compute_hp_bar_view,
                    bevyrogue::ui::combat_panel::compute_floating_damage_view,
                    bevyrogue::ui::combat_panel::observe_target_hurt,
                    bevyrogue::ui::combat_panel::tick_target_hurt_state,
                    bevyrogue::ui::combat_panel::observe_twin_core_badge,
                    bevyrogue::ui::combat_panel::tick_twin_core_badge,
                )
                    .chain(),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    roster_panel,
                    turn_order_panel,
                    bevyrogue::ui::phase_strip::observe_combat_beats,
                    bevyrogue::ui::phase_strip::render_phase_strip,
                    bevyrogue::ui::combat_panel::combat_panel,
                )
                    .chain(),
            );

        // Engine owns the CueRegistry resource (init above); the agumon module
        // only populates it. Called exactly once after the resource is inited.
        crate::windowed::digimon::register_all(app);
    }
}

fn parse_windowed_validation_toggle(raw: Option<&str>) -> Result<bool, String> {
    match raw {
        None | Some("0" | "false" | "False" | "FALSE" | "no" | "No" | "NO" | "off") => Ok(false),
        Some("") | Some("1" | "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on") => Ok(true),
        Some(other) => Err(format!(
            "BEVYROGUE_VALIDATION_WINDOWED must be one of: 1,true,yes,on,0,false,no,off (got {other:?})"
        )),
    }
}

fn parse_windowed_validation_config(
    enabled_raw: Option<&str>,
    soak_secs_raw: Option<&str>,
    baseline_raw: Option<&str>,
) -> Result<Option<WindowedValidationConfig>, String> {
    if !parse_windowed_validation_toggle(enabled_raw)? {
        return Ok(None);
    }

    let soak_secs = match soak_secs_raw {
        None => 300,
        Some(raw) => raw.parse::<u64>().map_err(|_| {
            format!(
                "BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS must be a positive integer (got {raw:?})"
            )
        })?,
    };

    if soak_secs == 0 {
        return Err(
            "BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS must be greater than zero".to_string(),
        );
    }

    let baseline = parse_validation_baseline_toggle(baseline_raw)?;

    Ok(Some(WindowedValidationConfig {
        soak_secs,
        baseline,
    }))
}

pub fn config_from_env() -> Result<Option<WindowedValidationConfig>, String> {
    parse_windowed_validation_config(
        std::env::var("BEVYROGUE_VALIDATION_WINDOWED")
            .ok()
            .as_deref(),
        std::env::var("BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS")
            .ok()
            .as_deref(),
        std::env::var("BEVYROGUE_VALIDATION_BASELINE")
            .ok()
            .as_deref(),
    )
}

pub fn register(app: &mut App, validation: Option<WindowedValidationConfig>) {
    // Kernel-only baseline soak: disable the entire anim-graph/render path so the
    // measured frame-time delta vs the full run is attributable to that path alone
    // (D027). Everything else — DefaultPlugins, soak tick, exit — is identical.
    let baseline = validation.is_some_and(|config| config.baseline);

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),
        ..default()
    }))
    .add_plugins(AnimationAssetPlugin)
    .add_plugins(DataPlugin)
    .insert_resource(TimelineClock(Clock::Windowed))
    .init_resource::<SuspendedTimelineState>()
    .add_plugins(UiPlugin)
    .add_systems(Update, windowed_bootstrap_system);

    if !baseline {
        app.add_plugins(render::RenderPlugin);
    }

    if let Some(config) = validation {
        app.insert_resource(config)
            .init_resource::<WindowedValidationState>()
            .add_systems(Update, windowed_validation_tick);
    }
}

fn windowed_bootstrap_system(
    mut commands: Commands,
    data_ready: Option<Res<data::DataReady>>,
    roster_handle: Option<Res<data::UnitRosterHandle>>,
    rosters: Res<Assets<data::units_ron::UnitRoster>>,
    units: Query<&Unit>,
    mut combat_state: ResMut<CombatState>,
    mut combat_events: MessageWriter<CombatEvent>,
    mut sp: ResMut<SpPool>,
) {
    if data_ready.is_none() || !units.is_empty() {
        return;
    }

    let Some(rhandle) = roster_handle else { return };
    let Some(roster) = rosters.get(&rhandle.0) else {
        return;
    };

    match bootstrap_encounter(
        roster,
        &SelectionRequest { rookie_ids: vec![] },
        EncounterPreset::AgumonTrainingDummy,
    ) {
        Ok(composition) => {
            let ally_ids: Vec<UnitId> = composition.allies.iter().map(|d| d.id).collect();
            let seeded_ids = apply_composition(&mut commands, &composition);
            combat_state.phase = CombatPhase::WaitingForTurn;
            sp.current = 3;
            sp.max = 3;
            combat_events.write(CombatEvent {
                source: UnitId(0),
                target: UnitId(0),
                kind: CombatEventKind::PartySelected {
                    ally_ids,
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
        }
        Err(err) => {
            error!("windowed bootstrap error: {err}");
        }
    }
}

pub fn register_combat_systems(app: &mut App) {
    app.init_resource::<bevyrogue::combat::turn_system::EnemyTurnRequestQueue>()
        .add_systems(
            Update,
            (
                burst_action_system,
                resolve_action_system,
                follow_up_listener_system,
                form_identity_listener_system,
                resolve_follow_up_action_system,
                continue_suspended_timeline_system,
                ult_accumulation_system,
                flush_ult_gain_system,
                advance_turn_system,
                resolve_enemy_turn_action_system,
                check_victory_system,
            )
                .chain(),
        );
}

fn windowed_validation_tick(world: &mut World) {
    if !world.contains_resource::<data::DataReady>() {
        return;
    }

    let config = *world.resource::<WindowedValidationConfig>();
    let soak_secs = config.soak_secs;
    let time = world.resource::<Time>();
    let elapsed_secs = time.elapsed_secs();
    let delta_secs = time.delta_secs();

    let mut log_start = false;
    let mut log_snapshot = false;
    let mut log_finish = false;
    let mut frame_time_line: Option<String> = None;

    {
        let mut state = world.resource_mut::<WindowedValidationState>();
        if state.started_at_secs.is_none() {
            state.started_at_secs = Some(elapsed_secs);
            log_start = true;
        } else {
            // Accumulate every presentation frame after the soak start frame; the
            // start frame's delta belongs to bring-up, not the soak window.
            state.record_frame(delta_secs);
        }
        if !state.snapshot_logged {
            state.snapshot_logged = true;
            log_snapshot = true;
        }
        if !state.finished
            && state
                .started_at_secs
                .is_some_and(|started_at| elapsed_secs - started_at >= soak_secs as f32)
        {
            state.finished = true;
            log_finish = true;
            let stats = state.frame_times.finalise();
            let mode = if config.baseline { "baseline" } else { "full" };
            frame_time_line = Some(format_frame_time_stats(&stats, mode));
        }
    }

    if log_start {
        info!("validation_windowed:start soak_secs={soak_secs}");
    }

    if log_snapshot {
        match capture_validation_snapshot(world) {
            Ok(snapshot) => info!(
                "validation_snapshot: {}",
                format_validation_snapshot(&snapshot)
            ),
            Err(err) => error!("validation_snapshot_error: {err}"),
        }
    }

    if log_finish {
        info!("validation_windowed:finish soak_secs={soak_secs}");
        if let Some(line) = frame_time_line {
            info!("{line}");
        }
        world.write_message(AppExit::Success);
    }
}

fn roster_panel(
    mut contexts: EguiContexts,
    units: Query<&Unit>,
    asset_server: Res<AssetServer>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::SidePanel::left("roster").show(ctx, |ui| {
        ui.heading("Roster (S04 RON-driven)");
        for u in &units {
            ui.label(format!(
                "{} — {:?} — HP {}/{}",
                u.name, u.attribute, u.hp_current, u.hp_max
            ));
        }
        if ui.button("reload combat").clicked() {
            for path in bevyrogue::data::DIGIMON_UNIT_PATHS
                .iter()
                .chain(bevyrogue::data::ENEMY_UNIT_PATHS.iter())
            {
                asset_server.reload(*path);
            }
            for path in bevyrogue::data::DIGIMON_SKILL_PATHS
                .iter()
                .chain(bevyrogue::data::ENEMY_SKILL_PATHS.iter())
            {
                asset_server.reload(*path);
            }
        }
    });
    Ok(())
}

/// Computes the upcoming turn order from live Action Values: units closest to
/// acting (highest AV) first, ties broken by ascending `UnitId` to match
/// `advance_turn_system`.
fn av_preview(units: &Query<(&Unit, &ActionValue)>, limit: usize) -> Vec<UnitId> {
    let mut ranked: Vec<(UnitId, i32)> = units.iter().map(|(u, av)| (u.id, av.0)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.0.cmp(&b.0.0)));
    ranked.into_iter().take(limit).map(|(id, _)| id).collect()
}

fn turn_order_panel(
    mut contexts: EguiContexts,
    av_units: Query<(&Unit, &ActionValue)>,
    units: Query<&Unit>,
    order: Res<TurnOrder>,
    combat_state: Res<CombatState>,
    mut turn_advanced: MessageWriter<TurnAdvanced>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let preview = av_preview(&av_units, 5);
    let can_advance = combat_state.phase == CombatPhase::WaitingForTurn
        && order.active_unit.is_none()
        && !preview.is_empty();

    egui::TopBottomPanel::top("av_bar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("AV Bar (next 5)");
            ui.horizontal(|ui| {
                for id in &preview {
                    let (label, color) = unit_chip(*id, &units);
                    let bg = egui::Frame::default()
                        .fill(color)
                        .inner_margin(egui::Margin::symmetric(6, 4));
                    bg.show(ui, |ui| {
                        ui.label(label);
                    });
                }
            });
            ui.horizontal(|ui| {
                let response = ui
                    .add_enabled(can_advance, egui::Button::new("Advance (debug)"))
                    .on_hover_text(
                        "Only valid while waiting for the next turn. Once an active actor exists, choose an action instead.",
                    );
                if response.clicked() {
                    if let Some(id) = preview.first().copied() {
                        turn_advanced.write(TurnAdvanced::of(id));
                    }
                }

                match combat_state.phase {
                    CombatPhase::WaitingForTurn => {
                        ui.label("Waiting for next actor selection");
                    }
                    CombatPhase::WaitingAction => {
                        ui.label("Actor selected; choose an action and click a target");
                    }
                    CombatPhase::Resolving => {
                        ui.label("Resolving action...");
                    }
                    CombatPhase::Victory | CombatPhase::Defeat => {}
                }
            });
        });
    Ok(())
}

fn unit_chip(id: UnitId, units: &Query<&Unit>) -> (String, egui::Color32) {
    let u = units.iter().find(|u| u.id == id);
    match u {
        Some(u) => (u.name.clone(), attr_color(u.attribute)),
        None => (format!("{:?}", id), egui::Color32::from_gray(80)),
    }
}

pub(crate) fn attr_color(a: Attribute) -> egui::Color32 {
    match a {
        Attribute::Vaccine => egui::Color32::from_rgb(80, 140, 220),
        Attribute::Data => egui::Color32::from_rgb(220, 200, 60),
        Attribute::Virus => egui::Color32::from_rgb(200, 60, 180),
        Attribute::Free => egui::Color32::from_gray(160),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windowed_validation_is_disabled_when_flag_absent_or_false() {
        assert_eq!(
            parse_windowed_validation_config(None, None, None).unwrap(),
            None
        );
        assert_eq!(
            parse_windowed_validation_config(Some("false"), Some("300"), None).unwrap(),
            None
        );
    }

    #[test]
    fn windowed_validation_uses_default_soak_when_enabled_without_override() {
        assert_eq!(
            parse_windowed_validation_config(Some("1"), None, None).unwrap(),
            Some(WindowedValidationConfig {
                soak_secs: 300,
                baseline: false
            })
        );
    }

    #[test]
    fn windowed_validation_rejects_bad_inputs() {
        assert_eq!(
            parse_windowed_validation_config(Some("true"), Some("120"), None).unwrap(),
            Some(WindowedValidationConfig {
                soak_secs: 120,
                baseline: false
            })
        );
        assert!(parse_windowed_validation_config(Some("true"), Some("0"), None).is_err());
        assert!(parse_windowed_validation_config(Some("true"), Some("bogus"), None).is_err());
        assert!(parse_windowed_validation_config(Some("maybe"), None, None).is_err());
    }

    #[test]
    fn windowed_validation_threads_baseline_toggle_into_config() {
        assert_eq!(
            parse_windowed_validation_config(Some("1"), None, Some("1")).unwrap(),
            Some(WindowedValidationConfig {
                soak_secs: 300,
                baseline: true
            })
        );
        // Baseline only matters when validation is enabled.
        assert_eq!(
            parse_windowed_validation_config(Some("0"), None, Some("1")).unwrap(),
            None
        );
        // A garbage baseline value is a hard error.
        assert!(parse_windowed_validation_config(Some("1"), None, Some("maybe")).is_err());
    }
}
