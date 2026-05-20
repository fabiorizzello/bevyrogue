//! Windowed app wiring (feature `windowed`): winit + wgpu + bevy_ui + egui.
//!
//! Provides the egui combat panel, roster/turn-order side panels, and an
//! optional validation tick that exits cleanly after a soak window.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationAssetPlugin, FrameCueCommand, NodeId,
    SkillGraphRegistry, StanceGraphRegistry,
};
use bevyrogue::combat::av::ActionValue;
use bevyrogue::combat::follow_up::{
    follow_up_listener_system, form_identity_listener_system, resolve_follow_up_action_system,
};
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::runtime::{Clock, CueBarrierStatus, CueReleaseResult, SuspendedTimelineState, TimelineClock};
use bevyrogue::combat::turn_order::TurnAdvanced;
use bevyrogue::combat::turn_system::{
    advance_turn_system, check_victory_system, continue_suspended_timeline_system,
    resolve_action_system, resolve_enemy_turn_action_system,
};
use bevyrogue::combat::types::{Attribute, UnitId};
use bevyrogue::combat::ultimate::{flush_ult_gain_system, ult_accumulation_system};
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::{self, DataPlugin};

const AGUMON_STANCE_GRAPH_ID: &str = "agumon_stance";
const AGUMON_SKILL_GRAPH_ID: &str = "agumon_skill";
const SHARP_CLAWS_SKILL_ID: &str = "sharp_claws";
const SHARP_CLAWS_WINDUP_NODE: &str = "sharp_claws_windup";
const SHARP_CLAWS_STRIKE_NODE: &str = "sharp_claws_strike";

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowedValidationConfig {
    soak_secs: u64,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
struct WindowedValidationState {
    started_at_secs: Option<f32>,
    snapshot_logged: bool,
    finished: bool,
}

/// Marker + FSM state for the on-screen Agumon preview actor.
#[derive(Component, Debug, Clone)]
struct AgumonSprite {
    player: AnimGraphPlayer,
    mode: AgumonPlaybackMode,
    last_release_frame: Option<ReleaseFrameKey>,
    last_missing_skill_graph_cue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgumonPlaybackMode {
    Idle,
    SharpClaws {
        cue_id: String,
        presentation_node: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseFrameKey {
    cue_id: String,
    node: String,
    local_frame: u32,
}

impl AgumonSprite {
    fn idle(entry: NodeId) -> Self {
        Self {
            player: AnimGraphPlayer::new(entry),
            mode: AgumonPlaybackMode::Idle,
            last_release_frame: None,
            last_missing_skill_graph_cue: None,
        }
    }

    fn start_sharp_claws(&mut self, cue_id: &str, presentation_node: &str) {
        self.player = AnimGraphPlayer::new(sharp_claws_start_node(presentation_node));
        self.mode = AgumonPlaybackMode::SharpClaws {
            cue_id: cue_id.to_string(),
            presentation_node: presentation_node.to_string(),
        };
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }

    fn return_to_idle(&mut self, stance_entry: NodeId) {
        self.player = AnimGraphPlayer::new(stance_entry);
        self.mode = AgumonPlaybackMode::Idle;
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }
}

/// Sprite camera + Agumon presentation state machine. Feature-agnostic player,
/// windowed playback bridge.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            advance_agumon_presentation
                .after(resolve_action_system)
                .before(continue_suspended_timeline_system),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn advance_agumon_presentation(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut barrier: ResMut<SuspendedTimelineState>,
    mut sprites: Query<&mut AgumonSprite>,
) {
    let stance_graph = stance_reg
        .resolve(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()))
        .and_then(|handle| graphs.get(handle));
    let skill_graph = skill_reg
        .resolve(&AnimGraphId(AGUMON_SKILL_GRAPH_ID.into()))
        .and_then(|handle| graphs.get(handle));
    let active_barrier = barrier.active_status().cloned();

    if sprites.is_empty() {
        if let Some(stance_graph) = stance_graph {
            commands.spawn(AgumonSprite::idle(stance_graph.entry.clone()));
        } else {
            return;
        }
    }

    for mut sprite in &mut sprites {
        sync_agumon_mode(&mut sprite, active_barrier.as_ref(), skill_graph, stance_graph);

        let Some(graph) = current_graph_for_mode(&sprite, stance_graph, skill_graph) else {
            continue;
        };

        let advance = sprite.player.advance_result(graph);
        let current_node = sprite.player.current_node.0.clone();
        let local_frame = local_frame_for(graph, &sprite.player.current_node, advance.frame);

        if active_barrier.is_some() {
            barrier.annotate_active_animation(&current_node, advance.frame as usize);
        }

        let awaiting = active_barrier
            .as_ref()
            .is_some_and(|status| status.awaiting_release);
        let released = active_barrier.as_ref().is_some_and(|status| status.released);
        trace!(
            target: "windowed.agumon_playback",
            mode = ?sprite.mode,
            node = current_node.as_str(),
            clip_frame = advance.frame,
            local_frame,
            awaiting,
            released,
            barrier = ?active_barrier.as_ref().map(barrier_trace_tuple),
            "agumon windowed playback tick"
        );

        if let (
            AgumonPlaybackMode::SharpClaws { cue_id, .. },
            Some(local_frame),
            Some(node),
        ) = (
            &sprite.mode,
            local_frame,
            graph.nodes.get(&sprite.player.current_node),
        ) {
            if should_release_kernel(node, local_frame)
                && !already_released_frame(
                    sprite.last_release_frame.as_ref(),
                    cue_id,
                    &current_node,
                    local_frame,
                )
            {
                let result = barrier.request_release(cue_id);
                trace!(
                    target: "windowed.agumon_playback",
                    cue_id,
                    node = current_node.as_str(),
                    clip_frame = advance.frame,
                    local_frame,
                    ?result,
                    "sharp claws release frame observed"
                );
                if matches!(
                    result,
                    CueReleaseResult::Released | CueReleaseResult::DuplicateRelease
                ) {
                    sprite.player.fire_kernel_cue();
                    sprite.last_release_frame = Some(ReleaseFrameKey {
                        cue_id: cue_id.clone(),
                        node: current_node.clone(),
                        local_frame,
                    });
                }
            }
        }

        if advance.exited {
            if let Some(stance_graph) = stance_graph {
                sprite.return_to_idle(stance_graph.entry.clone());
                trace!(
                    target: "windowed.agumon_playback",
                    "agumon playback returned to idle"
                );
            }
        }
    }
}

fn sync_agumon_mode(
    sprite: &mut AgumonSprite,
    active_barrier: Option<&CueBarrierStatus>,
    skill_graph: Option<&AnimGraph>,
    stance_graph: Option<&AnimGraph>,
) {
    let Some(status) = sharp_claws_barrier(active_barrier) else {
        return;
    };

    let presentation_node = status
        .animation_node
        .as_deref()
        .unwrap_or(SHARP_CLAWS_STRIKE_NODE);

    if matches!(
        &sprite.mode,
        AgumonPlaybackMode::SharpClaws { cue_id, .. } if cue_id == status.cue_id
    ) {
        sprite.last_missing_skill_graph_cue = None;
        return;
    }

    if skill_graph.is_none() {
        if sprite.last_missing_skill_graph_cue.as_deref() != Some(status.cue_id) {
            warn!(
                target: "windowed.agumon_playback",
                cue_id = status.cue_id,
                skill_id = %status.skill_id.0,
                graph_id = AGUMON_SKILL_GRAPH_ID,
                "Sharp Claws presentation graph missing; staying/falling back to idle while the kernel barrier remains suspended"
            );
        }
        sprite.last_missing_skill_graph_cue = Some(status.cue_id.to_string());
        if let Some(stance_graph) = stance_graph {
            sprite.return_to_idle(stance_graph.entry.clone());
        }
        return;
    }

    sprite.start_sharp_claws(status.cue_id, presentation_node);
    trace!(
        target: "windowed.agumon_playback",
        cue_id = status.cue_id,
        skill_id = %status.skill_id.0,
        presentation_node,
        start_node = %sprite.player.current_node.0,
        "Sharp Claws playback entered windup"
    );
}

fn current_graph_for_mode<'a>(
    sprite: &AgumonSprite,
    stance_graph: Option<&'a AnimGraph>,
    skill_graph: Option<&'a AnimGraph>,
) -> Option<&'a AnimGraph> {
    match sprite.mode {
        AgumonPlaybackMode::Idle => stance_graph,
        AgumonPlaybackMode::SharpClaws { .. } => skill_graph.or(stance_graph),
    }
}

fn sharp_claws_barrier(status: Option<&CueBarrierStatus>) -> Option<&CueBarrierStatus> {
    status.filter(|status| status.skill_id.0 == SHARP_CLAWS_SKILL_ID)
}

fn local_frame_for(graph: &AnimGraph, node_id: &NodeId, clip_frame: u32) -> Option<u32> {
    let node = graph.nodes.get(node_id)?;
    Some(if node.reverse {
        node.frames.end().saturating_sub(clip_frame)
    } else {
        clip_frame.saturating_sub(node.frames.start())
    })
}

fn should_release_kernel(node: &bevyrogue::animation::AnimNode, local_frame: u32) -> bool {
    node.cues.iter().any(|cue| {
        cue.at == local_frame && matches!(cue.command, FrameCueCommand::ReleaseKernel(_))
    })
}

fn already_released_frame(
    last_release_frame: Option<&ReleaseFrameKey>,
    cue_id: &str,
    node: &str,
    local_frame: u32,
) -> bool {
    last_release_frame.is_some_and(|last| {
        last.cue_id == cue_id && last.node == node && last.local_frame == local_frame
    })
}

fn sharp_claws_start_node(presentation_node: &str) -> NodeId {
    if presentation_node == SHARP_CLAWS_STRIKE_NODE {
        NodeId(SHARP_CLAWS_WINDUP_NODE.into())
    } else {
        NodeId(presentation_node.to_string())
    }
}

fn barrier_trace_tuple(status: &CueBarrierStatus) -> (&str, &str, &str, bool, bool) {
    (
        status.skill_id.0.as_str(),
        status.beat_id,
        status.cue_id,
        status.awaiting_release,
        status.released,
    )
}

/// Egui panels: roster, turn order, combat panel.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<bevyrogue::ui::combat_panel::PendingAction>()
            .init_resource::<bevyrogue::ui::combat_panel::PreviewDamageCache>()
            .add_systems(Update, bevyrogue::ui::combat_panel::refresh_preview_damage_cache)
            .add_systems(EguiPrimaryContextPass, roster_panel)
            .add_systems(EguiPrimaryContextPass, turn_order_panel)
            .add_systems(EguiPrimaryContextPass, bevyrogue::ui::combat_panel::combat_panel);
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

    Ok(Some(WindowedValidationConfig { soak_secs }))
}

pub fn config_from_env() -> Result<Option<WindowedValidationConfig>, String> {
    parse_windowed_validation_config(
        std::env::var("BEVYROGUE_VALIDATION_WINDOWED")
            .ok()
            .as_deref(),
        std::env::var("BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS")
            .ok()
            .as_deref(),
    )
}

pub fn register(app: &mut App, validation: Option<WindowedValidationConfig>) {
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),
        ..default()
    }))
    .add_plugins(AnimationAssetPlugin)
    .add_plugins(DataPlugin)
    .insert_resource(TimelineClock(Clock::Windowed))
    .init_resource::<SuspendedTimelineState>()
    .add_plugins(RenderPlugin)
    .add_plugins(UiPlugin);

    if let Some(config) = validation {
        app.insert_resource(config)
            .init_resource::<WindowedValidationState>()
            .add_systems(Update, windowed_validation_tick);
    }
}

pub fn register_combat_systems(app: &mut App) {
    app.init_resource::<bevyrogue::combat::turn_system::EnemyTurnRequestQueue>()
        .add_systems(
            Update,
            (
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

    let soak_secs = world.resource::<WindowedValidationConfig>().soak_secs;
    let elapsed_secs = world.resource::<Time>().elapsed_secs();

    let mut log_start = false;
    let mut log_snapshot = false;
    let mut log_finish = false;

    {
        let mut state = world.resource_mut::<WindowedValidationState>();
        if state.started_at_secs.is_none() {
            state.started_at_secs = Some(elapsed_secs);
            log_start = true;
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
    mut turn_advanced: MessageWriter<TurnAdvanced>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let preview = av_preview(&av_units, 5);
    egui::TopBottomPanel::top("av_bar").show(ctx, |ui| {
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
            if ui.button("Advance").clicked() {
                if let Some(id) = preview.first().copied() {
                    turn_advanced.write(TurnAdvanced::of(id));
                }
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
        assert_eq!(parse_windowed_validation_config(None, None).unwrap(), None);
        assert_eq!(
            parse_windowed_validation_config(Some("false"), Some("300")).unwrap(),
            None
        );
    }

    #[test]
    fn windowed_validation_uses_default_soak_when_enabled_without_override() {
        assert_eq!(
            parse_windowed_validation_config(Some("1"), None).unwrap(),
            Some(WindowedValidationConfig { soak_secs: 300 })
        );
    }

    #[test]
    fn windowed_validation_rejects_bad_inputs() {
        assert_eq!(
            parse_windowed_validation_config(Some("true"), Some("120")).unwrap(),
            Some(WindowedValidationConfig { soak_secs: 120 })
        );
        assert!(parse_windowed_validation_config(Some("true"), Some("0")).is_err());
        assert!(parse_windowed_validation_config(Some("true"), Some("bogus")).is_err());
        assert!(parse_windowed_validation_config(Some("maybe"), None).is_err());
    }

    #[test]
    fn sharp_claws_start_node_rewinds_strike_presentation_back_to_windup() {
        assert_eq!(
            sharp_claws_start_node(SHARP_CLAWS_STRIKE_NODE),
            NodeId(SHARP_CLAWS_WINDUP_NODE.into())
        );
        assert_eq!(
            sharp_claws_start_node("sharp_claws_recover"),
            NodeId("sharp_claws_recover".into())
        );
    }

    #[test]
    fn duplicate_release_guard_matches_same_cue_node_and_local_frame_only() {
        let last = ReleaseFrameKey {
            cue_id: "agumon/sharp_claws/impact".into(),
            node: SHARP_CLAWS_STRIKE_NODE.into(),
            local_frame: 1,
        };

        assert!(already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            SHARP_CLAWS_STRIKE_NODE,
            1,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            SHARP_CLAWS_STRIKE_NODE,
            2,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "other/cue",
            SHARP_CLAWS_STRIKE_NODE,
            1,
        ));
    }
}
