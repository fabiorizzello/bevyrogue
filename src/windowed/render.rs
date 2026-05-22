use std::collections::HashSet;

use bevy::prelude::*;

use bevyrogue::animation::{
    resolve_locus, AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles,
    AnimationClipLoadState, AnimationGraphLookupDiagnostics, AtlasGeometry, Clip,
    FrameCueCommand, NodeId, ResolvedAnimGraph, ResolvedAnimGraphSource,
    SkillGraphRegistry, StanceGraphRegistry, VfxMotion, VfxSpawnDescriptor,
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

#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct VfxParticle {
    ttl_ticks: u32,
    motion: VfxMotion,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct VfxParticleTarget {
    world_xy: [f32; 2],
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct VfxParticleSource {
    unit_id: UnitId,
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

/// Display scale applied to the native 512×512 Agumon atlas frames. Authored at
/// full tile size, an unscaled sprite fills the viewport and a full 4-per-team
/// roster (8 actors) cannot fit. 0.4 → ~205px per sprite. Provisional layout
/// value pending multi-slot positioning.
const SPRITE_DISPLAY_SCALE: f32 = 0.4;
const VFX_PARTICLE_TTL_TICKS: u32 = 6;
const VFX_PARTICLE_SIZE: f32 = 18.0;
const VFX_PARTICLE_Z: f32 = 1.0;

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

#[derive(Resource, Debug, Clone, Copy, Default)]
struct PendingAnimationTicks(u32);

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
            .insert_resource(PendingAnimationTicks::default())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, build_agumon_atlas.before(spawn_unit_sprites))
            .add_systems(Update, spawn_unit_sprites)
            .add_systems(Update, sample_animation_ticks.before(advance_vfx_particles))
            .add_systems(
                Update,
                (advance_vfx_particles, advance_agumon_presentation)
                    .chain()
                    .after(sample_animation_ticks)
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn sample_animation_ticks(
    time: Res<Time>,
    mut clock: ResMut<AnimationClock>,
    mut pending_ticks: ResMut<PendingAnimationTicks>,
) {
    pending_ticks.0 = clock.tick(time.delta_secs());
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
            Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(SPRITE_DISPLAY_SCALE)),
        ));
    }
}

fn advance_agumon_presentation(
    pending_ticks: Res<PendingAnimationTicks>,
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut lookup_diagnostics: ResMut<AnimationGraphLookupDiagnostics>,
    mut barrier: ResMut<SuspendedTimelineState>,
    atlas: Option<Res<AgumonAtlas>>,
    mut sprites: ParamSet<(
        Query<(&mut AgumonSprite, &mut Sprite, &Transform)>,
        Query<(&AgumonSprite, &Transform)>,
    )>,
) {
    let stance_graph =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs);

    {
        let active_barrier = barrier.active_status().cloned();
        if let Some(status) = active_barrier.as_ref() {
            // Auto-release is now only the fallback for genuinely unbridged skills
            // (no windowed presentation graph). Bridged skills — sharp_claws,
            // baby_flame, agumon_ult — release on their rendered ReleaseKernel cue
            // in the per-tick block below instead of being auto-released here.
            if status.awaiting_release && should_auto_release_unbridged(&status.skill_id.0) {
                debug!(
                    target: "windowed.agumon_playback",
                    skill_id = %status.skill_id.0,
                    beat_id = status.beat_id,
                    cue_id = status.cue_id,
                    hop_index = ?status.hop_index,
                    "unbridged windowed skill; auto-releasing barrier to avoid stalled resolve"
                );
                let _ = barrier.request_release(status.cue_id);
                return;
            }
        }
    }

    // Advance the player at the fixed animation rate, not once per render frame.
    // Most 60fps frames yield 0 ticks; the kernel-barrier release still observes
    // the rendered impact frame — it just samples it on the animation tick.
    for _ in 0..pending_ticks.0 {
        let active_barrier = barrier.active_status().cloned();
        let sprite_positions: Vec<(UnitId, [f32; 2])> = sprites
            .p1()
            .iter()
            .map(|(sprite, transform)| {
                (
                    sprite.unit_id,
                    [transform.translation.x, transform.translation.y],
                )
            })
            .collect();
        for (mut sprite, mut render_sprite, transform) in &mut sprites.p0() {
            let prev_node = sprite.player.current_node.0.clone();

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
            let entered = entered_node(&prev_node, &current_node);
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

            // Only the caster's sprite annotates the barrier with node/frame, so
            // an idle non-caster actor can't clobber the caster's impact state.
            if active_barrier
                .as_ref()
                .is_some_and(|status| barrier_targets_sprite(status, sprite.unit_id))
            {
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

            if should_spawn_node_vfx(&sprite.mode, active_barrier.as_ref(), sprite.unit_id) {
                if let Some(node_id) = entered {
                    if let Some(node) = graph.nodes.get(&NodeId(node_id.to_string())) {
                        let caster_xy = [transform.translation.x, transform.translation.y];
                        let target_xy =
                            nearest_non_caster_target_xy(&sprite_positions, sprite.unit_id, caster_xy);

                        for command in &node.on_enter {
                            let Some(descriptor) = VfxSpawnDescriptor::from_command(command) else {
                                continue;
                            };

                            let Some(target_xy) = target_xy else {
                                debug!(
                                    target: "windowed.agumon_playback",
                                    source_unit = ?sprite.unit_id,
                                    node = node_id,
                                    particle = %descriptor.particle.0,
                                    "SpawnParticle on_enter target could not be resolved"
                                );
                                continue;
                            };

                            let entity = spawn_vfx_particle(
                                &mut commands,
                                &descriptor,
                                caster_xy,
                                target_xy,
                                sprite.unit_id,
                            );
                            let resolved_xy = resolve_vfx_spawn_xy(&descriptor, caster_xy, target_xy);
                            trace!(
                                target: "windowed.agumon_playback",
                                entity = ?entity,
                                particle = %descriptor.particle.0,
                                resolved_xy = ?resolved_xy,
                                motion = ?descriptor.motion,
                                ttl_ticks = VFX_PARTICLE_TTL_TICKS,
                                source_unit = ?sprite.unit_id,
                                "spawned windowed vfx particle"
                            );
                        }
                    }
                }
            }

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
                    // Arm the KernelCue-gated FSM transition. The node actually
                    // changes on the next tick's advance_result; this only arms the
                    // pending cue. Skills with a forward KernelCue edge advance
                    // (Baby Burner charge->launch->recovery, Sharp Claws
                    // strike->recover); Baby Flame's impact node has no KernelCue
                    // edge (the bounce is pure VFX, not an animation hop), so this is
                    // a no-op there and impact->recover proceeds via TimeInNode.
                    sprite.player.fire_kernel_cue();
                    trace!(
                        target: "windowed.agumon_playback",
                        cue_id = cue_id.as_str(),
                        node = current_node.as_str(),
                        "multi-barrier FSM advance fired (kernel cue armed)"
                    );
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

fn entered_node<'a>(prev_node: &'a str, current_node: &'a str) -> Option<&'a str> {
    (prev_node != current_node).then_some(current_node)
}

fn decrement_vfx_ttl(ttl_ticks: u32) -> u32 {
    ttl_ticks.saturating_sub(1)
}

fn resolve_vfx_spawn_xy(
    descriptor: &VfxSpawnDescriptor,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
) -> [f32; 2] {
    resolve_locus(&descriptor.locus, caster_xy, target_xy)
}

fn vfx_particle_color(descriptor: &VfxSpawnDescriptor) -> Color {
    let name = descriptor.particle.0.as_str();
    if name.contains("burner") {
        Color::srgb(1.0, 0.8, 0.2)
    } else if name.contains("flame") {
        Color::srgb(1.0, 0.45, 0.15)
    } else {
        Color::srgb(1.0, 1.0, 1.0)
    }
}

fn should_spawn_node_vfx(
    mode: &AgumonPlaybackMode,
    active_barrier: Option<&CueBarrierStatus>,
    unit_id: UnitId,
) -> bool {
    matches!(mode, AgumonPlaybackMode::Skill { .. })
        && active_barrier
            .map(|status| barrier_targets_sprite(status, unit_id))
            .unwrap_or(true)
}

fn nearest_non_caster_target_xy(
    sprite_positions: &[(UnitId, [f32; 2])],
    caster: UnitId,
    caster_xy: [f32; 2],
) -> Option<[f32; 2]> {
    sprite_positions
        .iter()
        .filter(|(unit_id, _)| *unit_id != caster)
        .map(|(_, xy)| {
            let dx = xy[0] - caster_xy[0];
            let dy = xy[1] - caster_xy[1];
            let dist2 = dx * dx + dy * dy;
            (*xy, dist2)
        })
        .min_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
        .map(|(xy, _)| xy)
}

fn spawn_vfx_particle(
    commands: &mut Commands,
    descriptor: &VfxSpawnDescriptor,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
) -> Entity {
    let resolved_xy = resolve_vfx_spawn_xy(descriptor, caster_xy, target_xy);
    commands
        .spawn((
            Sprite::from_color(vfx_particle_color(descriptor), Vec2::splat(VFX_PARTICLE_SIZE)),
            Transform::from_xyz(resolved_xy[0], resolved_xy[1], VFX_PARTICLE_Z),
            VfxParticle {
                ttl_ticks: VFX_PARTICLE_TTL_TICKS,
                motion: descriptor.motion.clone(),
            },
            VfxParticleTarget { world_xy: target_xy },
            VfxParticleSource { unit_id: source_unit },
        ))
        .id()
}

fn advance_vfx_particles(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    mut particles: Query<
        (
            Entity,
            &mut VfxParticle,
            &mut Transform,
            &VfxParticleTarget,
            &VfxParticleSource,
        ),
    >,
) {
    for _ in 0..pending_ticks.0 {
        for (entity, mut particle, mut transform, target, source) in &mut particles {
            match &particle.motion {
                VfxMotion::Static => {}
                VfxMotion::FollowTarget | VfxMotion::ArcToTarget => {
                    let dx = target.world_xy[0] - transform.translation.x;
                    let dy = target.world_xy[1] - transform.translation.y;
                    let factor = match &particle.motion {
                        VfxMotion::FollowTarget => 0.45,
                        VfxMotion::ArcToTarget => 0.3,
                        VfxMotion::Static => 0.0,
                    };
                    transform.translation.x += dx * factor;
                    transform.translation.y += dy * factor;
                }
            }

            particle.ttl_ticks = decrement_vfx_ttl(particle.ttl_ticks);
            if particle.ttl_ticks == 0 {
                trace!(
                    target: "windowed.agumon_playback",
                    entity = ?entity,
                    motion = ?particle.motion,
                    source_unit = ?source.unit_id,
                    "despawned windowed vfx particle"
                );
                commands.entity(entity).despawn();
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

    // The kernel cue barrier is global, but only the caster's sprite should
    // present the skill. Every other on-screen actor keeps cycling idle.
    if !barrier_targets_sprite(status, sprite.unit_id) {
        return;
    }

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

/// Whether an active (global) kernel barrier belongs to a given on-screen actor.
/// Gates per-sprite presentation so only the caster animates the skill while
/// every other actor keeps cycling idle.
fn barrier_targets_sprite(status: &CueBarrierStatus, unit_id: UnitId) -> bool {
    status.source == unit_id
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

/// Whether an awaiting barrier for `skill_id` must be auto-released as the
/// unbridged fallback. Bridged skills (those with a windowed FSM entry node via
/// [`skill_start_node`]) release on their rendered `ReleaseKernel` cue instead,
/// so they are NOT auto-released here.
fn should_auto_release_unbridged(skill_id: &str) -> bool {
    skill_start_node(skill_id).is_none()
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
    use bevyrogue::combat::runtime::intent::CastId;
    use bevyrogue::combat::types::SkillId;

    fn barrier_status_from(source: UnitId) -> CueBarrierStatus {
        CueBarrierStatus {
            cast_id: CastId::ROOT,
            skill_id: SkillId("sharp_claws".into()),
            source,
            timeline_id: "sharp_claws",
            beat_id: "impact_damage",
            cue_id: "agumon/sharp_claws/impact",
            awaiting_release: true,
            released: false,
            timed_out: false,
            waited_frames: 0,
            timeout_frames: 180,
            animation_node: None,
            animation_frame: None,
            hop_index: None,
        }
    }


    #[test]
    fn entered_node_only_reports_actual_node_changes() {
        assert_eq!(entered_node("baby_flame_cast", "baby_flame_cast"), None);
        assert_eq!(
            entered_node("baby_flame_cast", "baby_flame_impact"),
            Some("baby_flame_impact")
        );
    }

    #[test]
    fn vfx_particle_ttl_reaches_zero_after_configured_ticks() {
        let mut ttl = VFX_PARTICLE_TTL_TICKS;
        for _ in 0..VFX_PARTICLE_TTL_TICKS {
            ttl = decrement_vfx_ttl(ttl);
        }
        assert_eq!(ttl, 0);
        assert_eq!(decrement_vfx_ttl(ttl), 0);
    }

    #[test]
    fn resolve_vfx_spawn_xy_honors_each_supported_locus() {
        use bevyrogue::animation::{ParticleId, VfxLocus};

        let caster_xy = [-24.0, 12.0];
        let target_xy = [48.0, -6.0];

        let caster_center = VfxSpawnDescriptor {
            particle: ParticleId("baby_flame".into()),
            locus: VfxLocus::CasterCenter,
            motion: VfxMotion::ArcToTarget,
        };
        assert_eq!(resolve_vfx_spawn_xy(&caster_center, caster_xy, target_xy), caster_xy);

        let target_center = VfxSpawnDescriptor {
            particle: ParticleId("baby_flame".into()),
            locus: VfxLocus::TargetCenter,
            motion: VfxMotion::FollowTarget,
        };
        assert_eq!(resolve_vfx_spawn_xy(&target_center, caster_xy, target_xy), target_xy);

        let primary_target_center = VfxSpawnDescriptor {
            particle: ParticleId("baby_flame".into()),
            locus: bevyrogue::animation::VfxLocus::PrimaryTargetCenter,
            motion: VfxMotion::Static,
        };
        assert_eq!(
            resolve_vfx_spawn_xy(&primary_target_center, caster_xy, target_xy),
            target_xy
        );
    }

    #[test]
    fn barrier_targets_only_the_casting_sprite() {
        let caster = UnitId(7);
        let status = barrier_status_from(caster);
        // The caster's sprite presents the skill; a non-caster (e.g. the target
        // dummy) stays idle even though the barrier is globally visible.
        assert!(barrier_targets_sprite(&status, caster));
        assert!(!barrier_targets_sprite(&status, UnitId(99)));
    }

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
    fn duplicate_release_guard_matches_same_cue_node_and_local_frame() {
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

    /// Baby Flame and Baby Burner are bridged, so the fallback auto-release branch
    /// in `advance_agumon_presentation` must NOT fire for them — they release on
    /// their rendered `ReleaseKernel` cue. Only skills with no windowed FSM entry
    /// (e.g. an unbridged Greymon skill) take the auto-release fallback.
    #[test]
    fn auto_release_fallback_only_targets_unbridged_skills() {
        assert!(!should_auto_release_unbridged(SHARP_CLAWS_SKILL_ID));
        assert!(!should_auto_release_unbridged(BABY_FLAME_SKILL_ID));
        assert!(!should_auto_release_unbridged(AGUMON_ULT_SKILL_ID));
        // An unbridged skill (no windowed presentation graph) still auto-releases.
        assert!(should_auto_release_unbridged("greymon_basic"));
    }

    /// The release-frame detector fires exactly on each authored `ReleaseKernel`
    /// local frame: Baby Burner's windup/recovery end-of-node cues (local 7) and
    /// the launch/impact cues (local 1) — the frames where damage lands and the
    /// multi-barrier walk advances.
    #[test]
    fn should_release_kernel_fires_on_authored_cue_frames() {
        use bevyrogue::animation::{AnimNode, FrameCue, FrameRange, ReleaseKernelCue};

        let node_with_release_at = |local: u32| AnimNode {
            frames: FrameRange(0, 8),
            on_enter: Vec::new(),
            cues: vec![FrameCue {
                at: local,
                command: FrameCueCommand::ReleaseKernel(ReleaseKernelCue),
            }],
            modifier: None,
            reverse: false,
        };

        // baby_burner_charge / baby_burner_recovery: end-of-node release at local 7.
        let charge = node_with_release_at(7);
        assert!(should_release_kernel(&charge, 7));
        assert!(!should_release_kernel(&charge, 6));

        // baby_burner_launch / baby_flame_impact: release at local 1 (impact).
        let launch = node_with_release_at(1);
        assert!(should_release_kernel(&launch, 1));
        assert!(!should_release_kernel(&launch, 0));

        // A node with no cues never releases.
        let plain = AnimNode {
            frames: FrameRange(0, 8),
            on_enter: Vec::new(),
            cues: Vec::new(),
            modifier: None,
            reverse: false,
        };
        assert!(!should_release_kernel(&plain, 1));
    }
}
