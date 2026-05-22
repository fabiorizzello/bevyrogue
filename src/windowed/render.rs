use std::collections::HashSet;

use bevy::prelude::*;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles, AnimationClipLoadState,
    AnimationGraphLookupDiagnostics, AtlasGeometry, Clip, FrameCueCommand, NodeId,
    ResolvedAnimGraph, ResolvedAnimGraphSource, SkillGraphRegistry, StanceGraphRegistry,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;

use super::{
    AGUMON_SKILL_GRAPH_ID, AGUMON_STANCE_GRAPH_ID, AGUMON_ULT_SKILL_ID, BABY_BURNER_CHARGE_NODE,
    BABY_FLAME_CAST_NODE, BABY_FLAME_SKILL_ID, SHARP_CLAWS_SKILL_ID, SHARP_CLAWS_WINDUP_NODE,
};

/// Marker + FSM state for the on-screen Agumon preview actor.
#[derive(Component, Debug, Clone)]
pub(super) struct AgumonSprite {
    pub(super) unit_id: UnitId,
    pub(super) player: AnimGraphPlayer,
    graph: ResolvedAnimGraph,
    mode: AgumonPlaybackMode,
    last_release_frame: Option<ReleaseFrameKey>,
    last_missing_skill_graph_cue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgumonPlaybackMode {
    Idle,
    /// An Agumon skill is presenting. `skill_id` keys the active cast,
    /// `awaiting_cue_id` is the barrier cue currently gating the kernel (it hops
    /// within one cast for multi-barrier skills), and `start_node` is the FSM
    /// entry node the player was seeded with for this skill.
    Skill {
        skill_id: String,
        awaiting_cue_id: String,
        start_node: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReleaseFrameKey {
    pub(super) cue_id: String,
    pub(super) node: String,
    pub(super) local_frame: u32,
}

impl AgumonSprite {
    pub(super) fn idle_for(unit_id: UnitId, graph: ResolvedAnimGraph) -> Self {
        let entry = graph.graph().entry.clone();
        Self {
            unit_id,
            player: AnimGraphPlayer::new(entry),
            graph,
            mode: AgumonPlaybackMode::Idle,
            last_release_frame: None,
            last_missing_skill_graph_cue: None,
        }
    }

    fn start_skill(
        &mut self,
        skill_id: &str,
        awaiting_cue_id: &str,
        start_node: &str,
        graph: ResolvedAnimGraph,
    ) {
        self.player = AnimGraphPlayer::new(NodeId(start_node.to_string()));
        self.graph = graph;
        self.mode = AgumonPlaybackMode::Skill {
            skill_id: skill_id.to_string(),
            awaiting_cue_id: awaiting_cue_id.to_string(),
            start_node: start_node.to_string(),
        };
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }

    pub(super) fn return_to_idle(
        &mut self,
        graph: ResolvedAnimGraph,
        preserve_missing_skill_graph_cue: Option<String>,
    ) {
        let entry = graph.graph().entry.clone();
        self.player = AnimGraphPlayer::new(entry);
        self.graph = graph;
        self.mode = AgumonPlaybackMode::Idle;
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = preserve_missing_skill_graph_cue;
    }
}

/// Bound atlas handles + geometry for the on-screen Agumon sprites. Built once
/// the agumon `Clip` is readable; both actors share the same image/layout, and
/// the geometry drives the player-frame -> tile-index map each tick.
#[derive(Resource, Debug, Clone)]
pub(super) struct AgumonAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    geometry: AtlasGeometry,
}

/// Default animation playback rate (frames of clip per second) when no
/// `BEVYROGUE_ANIM_FPS` override is set. 12 fps is a classic "snappy" pixel-art
/// step: the 6-frame idle loop cycles in ~0.5s rather than ~0.1s at 60fps.
const DEFAULT_ANIM_FPS: f32 = 12.0;
/// Upper bound on animation ticks applied in a single render frame. Bounds
/// catch-up after a frame-time hitch so playback never enters a spiral.
const MAX_CATCHUP_TICKS: u32 = 4;

/// Fixed-rate animation clock for the windowed presentation layer.
///
/// `advance_agumon_presentation` previously advanced the player once per render
/// frame (~60fps), making every animation play far too fast. This accumulates
/// real render-frame deltas and emits whole animation ticks at `fps`, so
/// playback speed is decoupled from render rate.
///
/// `fps` is the global *base* rate. Per-animation speed differences are already
/// expressible per-node via `PlaybackModifier::SpeedMul` in `anim_graph.ron`; a
/// per-Digimon base rate can later move into `ClipMeta` without disturbing this
/// seam. Only Agumon has bound sprites today, so one global clock is sufficient.
#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct AnimationClock {
    fps: f32,
    accumulator: f32,
}

impl AnimationClock {
    fn new(fps: f32) -> Self {
        Self {
            fps,
            accumulator: 0.0,
        }
    }

    /// Build from `BEVYROGUE_ANIM_FPS`, falling back to `DEFAULT_ANIM_FPS` (with
    /// a one-time warning) when the value is missing or invalid.
    fn from_env() -> Self {
        match parse_anim_fps(std::env::var("BEVYROGUE_ANIM_FPS").ok().as_deref()) {
            Ok(fps) => Self::new(fps),
            Err(err) => {
                warn!(
                    target: "windowed.agumon_playback",
                    "{err}; falling back to {DEFAULT_ANIM_FPS} fps"
                );
                Self::new(DEFAULT_ANIM_FPS)
            }
        }
    }

    /// Accumulate one render-frame delta and return how many animation ticks are
    /// due this frame. Most 60fps frames return 0; the count is capped at
    /// `MAX_CATCHUP_TICKS` and any residual backlog beyond the cap is dropped.
    fn tick(&mut self, delta_secs: f32) -> u32 {
        if self.fps <= 0.0 {
            return 0;
        }
        self.accumulator += delta_secs;
        let period = 1.0 / self.fps;
        let mut ticks = 0;
        while self.accumulator >= period && ticks < MAX_CATCHUP_TICKS {
            self.accumulator -= period;
            ticks += 1;
        }
        // Past the catch-up cap, discard the backlog so a long hitch can't queue
        // an unbounded burst of ticks across subsequent frames.
        if self.accumulator >= period {
            self.accumulator = 0.0;
        }
        ticks
    }
}

/// Parse the `BEVYROGUE_ANIM_FPS` override. Absent/empty selects the default;
/// any non-finite or non-positive value is a hard error so misconfiguration is
/// loud rather than freezing or racing the animation.
fn parse_anim_fps(raw: Option<&str>) -> Result<f32, String> {
    match raw {
        None | Some("") => Ok(DEFAULT_ANIM_FPS),
        Some(other) => {
            let fps = other.parse::<f32>().map_err(|_| {
                format!("BEVYROGUE_ANIM_FPS must be a positive number (got {other:?})")
            })?;
            if fps.is_finite() && fps > 0.0 {
                Ok(fps)
            } else {
                Err(format!(
                    "BEVYROGUE_ANIM_FPS must be a positive number (got {other:?})"
                ))
            }
        }
    }
}

/// Sprite camera + Agumon presentation state machine. Feature-agnostic player,
/// windowed playback bridge.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationClock::from_env())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, build_agumon_atlas.before(spawn_unit_sprites))
            .add_systems(Update, spawn_unit_sprites)
            .add_systems(
                Update,
                advance_agumon_presentation
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Builds the shared `AgumonAtlas` (image + `TextureAtlasLayout` + geometry)
/// once the agumon `Clip` is readable. Idempotent: returns early once the
/// resource exists, so it runs at most one effective build. Emits a one-time
/// `info!` describing the grid and a one-time `warn!` if the clip never becomes
/// readable or the atlas image fails to load.
#[allow(clippy::too_many_arguments)]
fn build_agumon_atlas(
    mut commands: Commands,
    existing: Option<Res<AgumonAtlas>>,
    clip_load_state: Res<AnimationClipLoadState>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    clips: Res<Assets<Clip>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut warned: Local<bool>,
) {
    if existing.is_some() {
        return;
    }

    let Some(handles) = clip_handles else {
        return;
    };

    // The agumon clip is index 0 (DEFAULT_ANIM_CLIP_PATHS) and is the geometry
    // source for both on-screen actors.
    let clip = handles.0.first().and_then(|handle| clips.get(handle));
    let Some(clip) = clip else {
        // Only a real failure (load state reports ready but the asset is
        // missing) is worth surfacing; the transient loading state is silent.
        if clip_load_state.ready && !*warned {
            warn!(
                target: "windowed.agumon_playback",
                "agumon clip not readable after load state ready; atlas binding deferred — sprites stay blank"
            );
            *warned = true;
        }
        return;
    };

    let geometry = AtlasGeometry::from_clip_meta(&clip.meta);
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(geometry.frame_size.w, geometry.frame_size.h),
        geometry.columns,
        geometry.rows,
        None,
        None,
    );
    let layout = layouts.add(layout);
    let image = asset_server.load("digimon/agumon_atlas.png");

    if matches!(
        asset_server.load_state(image.id()),
        bevy::asset::LoadState::Failed(_)
    ) && !*warned
    {
        warn!(
            target: "windowed.agumon_playback",
            "agumon atlas image load failed: digimon/agumon_atlas.png — sprites will render blank"
        );
        *warned = true;
    }

    info!(
        target: "windowed.agumon_playback",
        frame_w = geometry.frame_size.w,
        frame_h = geometry.frame_size.h,
        columns = geometry.columns,
        rows = geometry.rows,
        total_frames = geometry.total_frames,
        "agumon atlas built (TextureAtlasLayout + image bound)"
    );

    commands.insert_resource(AgumonAtlas {
        image,
        layout,
        geometry,
    });
}

/// Spawns one `AgumonSprite` entity per unit that does not yet have one.
/// Runs every frame but is idempotent: once a sprite exists for a unit it is skipped.
/// Waits for the stance graph to be loaded before spawning anything.
fn spawn_unit_sprites(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    atlas: Option<Res<AgumonAtlas>>,
    units: Query<(&Unit, &Team)>,
    sprites: Query<&AgumonSprite>,
) {
    let Some(stance_graph) =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs)
    else {
        return;
    };

    // Both the stance graph and the bound atlas must be ready before we can
    // spawn a sprite that renders real pixels.
    let Some(atlas) = atlas else {
        return;
    };

    let spawned: HashSet<UnitId> = sprites.iter().map(|s| s.unit_id).collect();

    for (unit, team) in &units {
        if spawned.contains(&unit.id) {
            continue;
        }
        let flip_x = *team == Team::Enemy;
        let x = if flip_x { 200.0_f32 } else { -200.0_f32 };
        commands.spawn((
            AgumonSprite::idle_for(unit.id, stance_graph.clone()),
            Sprite {
                image: atlas.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: 0,
                }),
                flip_x,
                ..default()
            },
            Transform::from_xyz(x, 0.0, 0.0),
        ));
    }
}

pub(super) fn advance_agumon_presentation(
    time: Res<Time>,
    mut clock: ResMut<AnimationClock>,
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut lookup_diagnostics: ResMut<AnimationGraphLookupDiagnostics>,
    mut barrier: ResMut<SuspendedTimelineState>,
    atlas: Option<Res<AgumonAtlas>>,
    mut sprites: Query<(&mut AgumonSprite, &mut Sprite)>,
) {
    let stance_graph =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs);

    {
        let active_barrier = barrier.active_status().cloned();
        if let Some(status) = active_barrier.as_ref() {
            if status.awaiting_release && status.skill_id.0 != SHARP_CLAWS_SKILL_ID {
                debug!(
                    target: "windowed.agumon_playback",
                    skill_id = %status.skill_id.0,
                    beat_id = status.beat_id,
                    cue_id = status.cue_id,
                    hop_index = ?status.hop_index,
                    "unsupported windowed cue bridge; auto-releasing barrier to avoid stalled resolve"
                );
                let _ = barrier.request_release(status.cue_id);
                return;
            }
        }
    }

    // Advance the player at the fixed animation rate, not once per render frame.
    // Most 60fps frames yield 0 ticks; the kernel-barrier release still observes
    // the rendered impact frame — it just samples it on the animation tick.
    let ticks = clock.tick(time.delta_secs());
    for _ in 0..ticks {
        let active_barrier = barrier.active_status().cloned();
        for (mut sprite, mut render_sprite) in &mut sprites {
            sync_agumon_mode(
                &mut sprite,
                active_barrier.as_ref(),
                &skill_reg,
                &stance_reg,
                &graphs,
                &mut lookup_diagnostics,
            );

            let graph = sprite.graph.graph().clone();
            let advance = sprite.player.advance_result(&graph);
            let current_node = sprite.player.current_node.0.clone();
            let local_frame = local_frame_for(&graph, &sprite.player.current_node, advance.frame);

            // Drive the rendered tile from the player frame via the identity
            // frame -> atlas-index map. Leave the index unchanged on an
            // out-of-range frame (atlas_index == None).
            let atlas_index = atlas
                .as_ref()
                .and_then(|atlas| atlas.geometry.atlas_index(advance.frame));
            if let (Some(index), Some(texture_atlas)) =
                (atlas_index, render_sprite.texture_atlas.as_mut())
            {
                texture_atlas.index = index as usize;
            }

            if active_barrier.is_some() {
                barrier.annotate_active_animation(&current_node, advance.frame as usize);
            }

            let awaiting = active_barrier
                .as_ref()
                .is_some_and(|status| status.awaiting_release);
            let released = active_barrier
                .as_ref()
                .is_some_and(|status| status.released);
            let (mode_skill_id, mode_awaiting_cue_id) = mode_trace_fields(&sprite.mode);
            trace!(
                target: "windowed.agumon_playback",
                mode = ?sprite.mode,
                skill_id = mode_skill_id,
                awaiting_cue_id = mode_awaiting_cue_id,
                graph_source = ?sprite.graph.source,
                node = current_node.as_str(),
                clip_frame = advance.frame,
                local_frame,
                atlas_index,
                awaiting,
                released,
                barrier = ?active_barrier.as_ref().map(barrier_trace_tuple),
                "agumon windowed playback tick"
            );

            let pending_release = if let AgumonPlaybackMode::Skill {
                awaiting_cue_id, ..
            } = &sprite.mode
            {
                if let (Some(lf), Some(node)) =
                    (local_frame, graph.nodes.get(&sprite.player.current_node))
                {
                    if should_release_kernel(node, lf)
                        && !already_released_frame(
                            sprite.last_release_frame.as_ref(),
                            awaiting_cue_id,
                            &current_node,
                            lf,
                        )
                    {
                        Some((awaiting_cue_id.clone(), lf))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((cue_id, lf)) = pending_release {
                let result = barrier.request_release(&cue_id);
                trace!(
                    target: "windowed.agumon_playback",
                    cue_id = cue_id.as_str(),
                    node = current_node.as_str(),
                    clip_frame = advance.frame,
                    local_frame = lf,
                    ?result,
                    "skill release frame observed"
                );
                if matches!(
                    result,
                    CueReleaseResult::Released | CueReleaseResult::DuplicateRelease
                ) {
                    sprite.player.fire_kernel_cue();
                    sprite.last_release_frame = Some(ReleaseFrameKey {
                        cue_id,
                        node: current_node.clone(),
                        local_frame: lf,
                    });
                }
            }

            if advance.exited {
                if let Some(stance_graph) = stance_graph.clone() {
                    let preserve_missing = active_barrier.as_ref().and_then(|status| {
                        (sprite.graph.source == ResolvedAnimGraphSource::InstantFallback
                            && sprite.last_missing_skill_graph_cue.as_deref()
                                == Some(status.cue_id))
                        .then(|| status.cue_id.to_string())
                    });
                    sprite.return_to_idle(stance_graph, preserve_missing);
                    trace!(
                        target: "windowed.agumon_playback",
                        "agumon playback returned to idle"
                    );
                }
            }
        }
    }
}

fn sync_agumon_mode(
    sprite: &mut AgumonSprite,
    active_barrier: Option<&CueBarrierStatus>,
    skill_reg: &SkillGraphRegistry,
    stance_reg: &StanceGraphRegistry,
    graphs: &Assets<AnimGraph>,
    lookup_diagnostics: &mut AnimationGraphLookupDiagnostics,
) {
    let Some(status) = active_barrier else {
        return;
    };

    // Only skills with a known FSM entry node are bridged here. Unbridged skills
    // are handled by the auto-release fallback in `advance_agumon_presentation`.
    let Some(start_node) = skill_start_node(&status.skill_id.0) else {
        return;
    };

    // Same skill already presenting: keep the player advancing through the FSM
    // (do NOT reset). Multi-barrier skills hop `cue_id` within one cast, so just
    // refresh the awaiting cue and clear the dedup guard when the cue changed.
    match classify_same_skill_sync(&sprite.mode, &status.skill_id.0, status.cue_id) {
        SameSkillSync::Unchanged => {
            sprite.last_missing_skill_graph_cue = None;
            return;
        }
        SameSkillSync::CueChanged => {
            if let AgumonPlaybackMode::Skill {
                awaiting_cue_id, ..
            } = &mut sprite.mode
            {
                *awaiting_cue_id = status.cue_id.to_string();
            }
            sprite.last_release_frame = None;
            trace!(
                target: "windowed.agumon_playback",
                skill_id = %status.skill_id.0,
                awaiting_cue_id = status.cue_id,
                hop_index = ?status.hop_index,
                node = %sprite.player.current_node.0,
                "agumon multi-barrier cue advanced (player not reset)"
            );
            sprite.last_missing_skill_graph_cue = None;
            return;
        }
        SameSkillSync::DifferentSkill => {}
    }

    if sprite.last_missing_skill_graph_cue.as_deref() == Some(status.cue_id) {
        return;
    }

    let resolved_graph = skill_reg.resolve_snapshot_or_instant_fallback(
        &AnimGraphId(AGUMON_SKILL_GRAPH_ID.into()),
        graphs,
        lookup_diagnostics,
    );

    if resolved_graph.source == ResolvedAnimGraphSource::InstantFallback {
        warn!(
            target: "windowed.agumon_playback",
            cue_id = status.cue_id,
            skill_id = %status.skill_id.0,
            graph_id = AGUMON_SKILL_GRAPH_ID,
            diagnostic = lookup_diagnostics.last_message.as_deref().unwrap_or("missing"),
            "skill presentation graph missing; running deterministic instant fallback"
        );
        sprite.last_missing_skill_graph_cue = Some(status.cue_id.to_string());
    }

    sprite.start_skill(
        &status.skill_id.0,
        status.cue_id,
        start_node,
        resolved_graph,
    );
    trace!(
        target: "windowed.agumon_playback",
        cue_id = status.cue_id,
        skill_id = %status.skill_id.0,
        start_node = %sprite.player.current_node.0,
        graph_source = ?sprite.graph.source,
        "skill playback entered start node"
    );

    if sprite.graph.source == ResolvedAnimGraphSource::InstantFallback {
        if let Some(stance_graph) =
            stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), graphs)
        {
            trace!(
                target: "windowed.agumon_playback",
                graph_id = AGUMON_STANCE_GRAPH_ID,
                stance_entry = %stance_graph.graph().entry.0,
                "stance snapshot remains available for post-fallback idle restore"
            );
        }
    }
}

/// FSM entry node for a bridged Agumon skill, or `None` when the skill has no
/// windowed presentation graph (the auto-release fallback handles those).
fn skill_start_node(skill_id: &str) -> Option<&'static str> {
    match skill_id {
        SHARP_CLAWS_SKILL_ID => Some(SHARP_CLAWS_WINDUP_NODE),
        BABY_FLAME_SKILL_ID => Some(BABY_FLAME_CAST_NODE),
        AGUMON_ULT_SKILL_ID => Some(BABY_BURNER_CHARGE_NODE),
        _ => None,
    }
}

/// How an active barrier reconciles against the current playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SameSkillSync {
    /// Mode is a different skill (or Idle); the caller must (re)start the skill,
    /// seeding the player at the skill's FSM entry node.
    DifferentSkill,
    /// Same skill, same awaiting cue; the player keeps advancing untouched.
    Unchanged,
    /// Same skill, awaiting cue hopped within the cast; refresh `awaiting_cue_id`
    /// and clear the dedup guard, but do NOT reset the player node.
    CueChanged,
}

/// Classify an active barrier `(skill_id, cue_id)` against the current mode.
/// This is the load-bearing seam that lets multi-barrier skills advance their
/// FSM in place instead of restarting the player on every barrier hop.
fn classify_same_skill_sync(
    mode: &AgumonPlaybackMode,
    skill_id: &str,
    cue_id: &str,
) -> SameSkillSync {
    match mode {
        AgumonPlaybackMode::Skill {
            skill_id: active,
            awaiting_cue_id,
            ..
        } if active == skill_id => {
            if awaiting_cue_id == cue_id {
                SameSkillSync::Unchanged
            } else {
                SameSkillSync::CueChanged
            }
        }
        _ => SameSkillSync::DifferentSkill,
    }
}

/// `(skill_id, awaiting_cue_id)` for the active mode, used to enrich the
/// per-tick playback trace. `Idle` carries neither.
fn mode_trace_fields(mode: &AgumonPlaybackMode) -> (Option<&str>, Option<&str>) {
    match mode {
        AgumonPlaybackMode::Idle => (None, None),
        AgumonPlaybackMode::Skill {
            skill_id,
            awaiting_cue_id,
            ..
        } => (Some(skill_id), Some(awaiting_cue_id)),
    }
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

fn barrier_trace_tuple(status: &CueBarrierStatus) -> (&str, &str, &str, bool, bool) {
    (
        status.skill_id.0.as_str(),
        status.beat_id,
        status.cue_id,
        status.awaiting_release,
        status.released,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_start_node_maps_each_bridged_skill_to_its_fsm_entry() {
        assert_eq!(
            skill_start_node(SHARP_CLAWS_SKILL_ID),
            Some(SHARP_CLAWS_WINDUP_NODE)
        );
        assert_eq!(
            skill_start_node(BABY_FLAME_SKILL_ID),
            Some(BABY_FLAME_CAST_NODE)
        );
        assert_eq!(
            skill_start_node(AGUMON_ULT_SKILL_ID),
            Some(BABY_BURNER_CHARGE_NODE)
        );
        // Unbridged skills have no windowed presentation graph.
        assert_eq!(skill_start_node("greymon_basic"), None);
    }

    #[test]
    fn same_skill_cue_hop_advances_without_resetting_player() {
        let mode = AgumonPlaybackMode::Skill {
            skill_id: AGUMON_ULT_SKILL_ID.into(),
            awaiting_cue_id: "agumon/baby_burner/windup".into(),
            start_node: BABY_BURNER_CHARGE_NODE.into(),
        };

        // Same skill, same cue: nothing to do (player keeps advancing).
        assert_eq!(
            classify_same_skill_sync(&mode, AGUMON_ULT_SKILL_ID, "agumon/baby_burner/windup"),
            SameSkillSync::Unchanged
        );
        // Same skill, the awaiting cue hopped to the next barrier within the cast:
        // refresh the cue + dedup guard, but the player is NOT restarted.
        assert_eq!(
            classify_same_skill_sync(&mode, AGUMON_ULT_SKILL_ID, "agumon/baby_burner/impact"),
            SameSkillSync::CueChanged
        );
        // A different skill (or Idle) forces a fresh start.
        assert_eq!(
            classify_same_skill_sync(&mode, SHARP_CLAWS_SKILL_ID, "agumon/sharp_claws/impact"),
            SameSkillSync::DifferentSkill
        );
        assert_eq!(
            classify_same_skill_sync(&AgumonPlaybackMode::Idle, AGUMON_ULT_SKILL_ID, "x"),
            SameSkillSync::DifferentSkill
        );
    }

    #[test]
    fn anim_clock_accumulates_render_frames_into_anim_ticks() {
        let mut clock = AnimationClock::new(12.0);
        // 1/12s = 0.0833s; four 60fps frames (4/60 = 0.0667s) stay under the
        // period and emit no tick.
        let early: u32 = (0..4).map(|_| clock.tick(1.0 / 60.0)).sum();
        assert_eq!(early, 0);
        // The fifth frame crosses the period and emits exactly one tick.
        assert_eq!(clock.tick(1.0 / 60.0), 1);
    }

    #[test]
    fn anim_clock_caps_catchup_after_a_hitch() {
        let mut clock = AnimationClock::new(12.0);
        // A one-second hitch is 12 periods' worth, but catch-up is bounded.
        assert_eq!(clock.tick(1.0), MAX_CATCHUP_TICKS);
        // Backlog beyond the cap is dropped, so the next normal frame is quiet.
        assert_eq!(clock.tick(1.0 / 60.0), 0);
    }

    #[test]
    fn anim_clock_with_nonpositive_fps_never_ticks() {
        let mut clock = AnimationClock::new(0.0);
        assert_eq!(clock.tick(10.0), 0);
    }

    #[test]
    fn parse_anim_fps_defaults_and_validates() {
        assert!((parse_anim_fps(None).unwrap() - DEFAULT_ANIM_FPS).abs() < f32::EPSILON);
        assert!((parse_anim_fps(Some("")).unwrap() - DEFAULT_ANIM_FPS).abs() < f32::EPSILON);
        assert!((parse_anim_fps(Some("24")).unwrap() - 24.0).abs() < f32::EPSILON);
        assert!(parse_anim_fps(Some("0")).is_err());
        assert!(parse_anim_fps(Some("-5")).is_err());
        assert!(parse_anim_fps(Some("fast")).is_err());
    }

    #[test]
    fn duplicate_release_guard_matches_same_cue_node_and_local_frame_only() {
        let last = ReleaseFrameKey {
            cue_id: "agumon/sharp_claws/impact".into(),
            node: "sharp_claws_strike".into(),
            local_frame: 1,
        };

        assert!(already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            "sharp_claws_strike",
            1,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            "sharp_claws_strike",
            2,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "other/cue",
            "sharp_claws_strike",
            1,
        ));
    }
}
