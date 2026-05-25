use std::collections::HashSet;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles, AnimationClipLoadState,
    AnimationGraphLookupDiagnostics, AtlasGeometry, Clip, Command, EffectDef, FrameCueCommand,
    NodeId, ParticleId, ResolvedAnimGraph, ResolvedAnimGraphSource, SkillGraphRegistry,
    StanceGraphRegistry, VfxAsset, VfxLocus, VfxMotion, VfxSpawnDescriptor, eval_color, eval_scale,
    resolve_effect, resolve_locus, spawn_plan,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;
use bevyrogue::ui::combat_panel::latest_baby_burner_flash_trigger;

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

#[derive(Component, Debug, Clone, PartialEq)]
struct VfxParticle {
    ttl_ticks: u32,
    age_ticks: u32,
    motion: VfxMotion,
    kind: VfxParticleKind,
    anchor: Option<VfxAnchor>,
    /// Per-particle seed angle (radians). Charge embers use it as the starting
    /// orbit angle; impact shards use it as the fan-out direction. 0.0 for the
    /// single-quad kinds, which derive everything from `age_ticks`.
    phase: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VfxParticleKind {
    Generic,
    BabyFlameCharge,
    /// Small orbiter spiralling inward toward the mouth during the charge,
    /// reading as a vortex feeding the core flame. Reuses the charge sprite.
    BabyFlameEmber,
    BabyFlameProjectile,
    BabyFlameImpact,
    /// Dissolving fragment that fans outward from the impact point and fades.
    /// Reuses the impact sprite.
    BabyFlameImpactShard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VfxAnchor {
    Mouth,
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

#[derive(Resource, Debug, Clone)]
struct VfxVisuals {
    baby_flame_charge: Handle<Image>,
    baby_flame_projectile: Handle<Image>,
    baby_flame_impact: Handle<Image>,
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
const VFX_MOUTH_OFFSET_X_PX: f32 = 92.0;
const VFX_MOUTH_OFFSET_Y_PX: f32 = 24.0;

// --- Baby Flame three-phase VFX polish (tunable; verify visually via `cargo winx`) ---
/// Embers spawned in a ring around the mouth at charge start, spiralling inward.
const BABY_FLAME_EMBER_COUNT: u32 = 7;
/// Embers live the full charge window alongside the core flame quad.
const BABY_FLAME_EMBER_TTL: u32 = 24;
/// Starting orbit radius (world px) of the ember ring before it spirals in.
const BABY_FLAME_EMBER_RADIUS_PX: f32 = 58.0;
/// Angular velocity of the swirl, radians per animation tick.
const BABY_FLAME_EMBER_OMEGA: f32 = 0.9;
/// On-screen size of a single ember quad.
const BABY_FLAME_EMBER_SIZE: f32 = 11.0;
/// Shards spawned at the impact point, fanning outward and fading.
const BABY_FLAME_IMPACT_SHARD_COUNT: u32 = 8;
/// How long the dissolve fan lingers (animation ticks).
const BABY_FLAME_IMPACT_SHARD_TTL: u32 = 5;
/// Maximum outward travel (world px) a shard reaches at end of life.
const BABY_FLAME_IMPACT_SHARD_SPREAD_PX: f32 = 64.0;
/// On-screen size of a single dissolve shard.
const BABY_FLAME_IMPACT_SHARD_SIZE: f32 = 14.0;

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
        // Owned per-Digimon VFX schema (M004/S01, D033/D034). Mirrors the
        // AnimGraph loader: a `.ron` -> typed `VfxAsset` loader, windowed-gated so
        // no rendering/asset dependency leaks into the headless build (R016).
        app.add_plugins(RonAssetPlugin::<VfxAsset>::new(&["ron"]))
            .insert_resource(AnimationClock::from_env())
            .insert_resource(PendingAnimationTicks::default())
            .add_systems(Startup, (setup_camera, load_vfx_visuals, load_agumon_vfx))
            .add_systems(Update, diagnose_agumon_vfx_load)
            .add_systems(Update, build_agumon_atlas.before(spawn_unit_sprites))
            .add_systems(Update, spawn_unit_sprites)
            .add_systems(Update, sample_animation_ticks.before(advance_vfx_particles))
            .add_systems(
                Update,
                spawn_detonate_particles
                    .after(resolve_action_system)
                    .after(spawn_unit_sprites)
                    .before(continue_suspended_timeline_system),
            )
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

fn load_vfx_visuals(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(VfxVisuals {
        baby_flame_charge: asset_server.load("vfx/baby_flame_charge.png"),
        baby_flame_projectile: asset_server.load("vfx/baby_flame_projectile.png"),
        baby_flame_impact: asset_server.load("vfx/baby_flame_impact.png"),
    });
}

/// Path (relative to `assets/`) of Agumon's owned VFX asset.
const AGUMON_VFX_PATH: &str = "digimon/agumon/vfx.ron";
/// Namespaced effect id of the Baby Flame impact fan-out within the asset.
const AGUMON_IMPACT_EFFECT_ID: &str = "baby_flame.impact";

/// Handle to Agumon's owned `VfxAsset` (M004/S01). Held in a resource so the
/// impact fan-out can source its spawn plan and appearance curves from data.
#[derive(Resource, Debug, Clone)]
struct AgumonVfx {
    handle: Handle<VfxAsset>,
}

fn load_agumon_vfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AgumonVfx {
        handle: asset_server.load(AGUMON_VFX_PATH),
    });
    info!(
        target: "windowed.agumon_playback",
        path = AGUMON_VFX_PATH,
        "agumon vfx asset load requested"
    );
}

/// Surface a load failure for Agumon's `vfx.ron` once, mirroring the missing
/// anim-graph diagnostic. A failed/missing asset never enters `Assets<VfxAsset>`,
/// so the impact path silently falls back to the hardcoded constants; this makes
/// that fallback visible rather than mysterious (slice failure-visibility).
fn diagnose_agumon_vfx_load(
    agumon_vfx: Option<Res<AgumonVfx>>,
    asset_server: Res<AssetServer>,
    mut warned: Local<bool>,
) {
    if *warned {
        return;
    }
    let Some(agumon_vfx) = agumon_vfx else {
        return;
    };
    if matches!(
        asset_server.load_state(agumon_vfx.handle.id()),
        bevy::asset::LoadState::Failed(_)
    ) {
        warn!(
            target: "windowed.agumon_playback",
            effect_id = AGUMON_IMPACT_EFFECT_ID,
            path = AGUMON_VFX_PATH,
            reason = "vfx.ron failed to load or parse",
            "falling back to hardcoded Baby Flame impact path"
        );
        *warned = true;
    }
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
    vfx_visuals: Option<Res<VfxVisuals>>,
    vfx_particles: Query<(Entity, &VfxParticle, &VfxParticleSource)>,
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
                        let target_xy = nearest_non_caster_target_xy(
                            &sprite_positions,
                            sprite.unit_id,
                            caster_xy,
                        );

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
                                vfx_visuals.as_deref(),
                                caster_xy,
                                target_xy,
                                sprite.unit_id,
                                render_sprite.flip_x,
                                transform.scale.x,
                                0.0,
                                None,
                            );
                            // The charge core is joined by a ring of embers that
                            // spiral inward, reading as a vortex feeding the flame.
                            if vfx_particle_kind(&descriptor) == VfxParticleKind::BabyFlameCharge {
                                spawn_baby_flame_embers(
                                    &mut commands,
                                    vfx_visuals.as_deref(),
                                    caster_xy,
                                    target_xy,
                                    sprite.unit_id,
                                    render_sprite.flip_x,
                                    transform.scale.x,
                                );
                            }
                            let resolved_xy =
                                resolve_vfx_spawn_xy(&descriptor, caster_xy, target_xy);
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
                    if mode_skill_id == Some(BABY_FLAME_SKILL_ID) {
                        for (entity, particle, source) in &vfx_particles {
                            if source.unit_id == sprite.unit_id
                                && matches!(
                                    particle.kind,
                                    VfxParticleKind::BabyFlameCharge
                                        | VfxParticleKind::BabyFlameEmber
                                )
                            {
                                commands.entity(entity).despawn();
                            }
                        }

                        if let Some(target_xy) = nearest_non_caster_target_xy(
                            &sprite_positions,
                            sprite.unit_id,
                            [transform.translation.x, transform.translation.y],
                        ) {
                            let descriptor = baby_flame_projectile_descriptor();
                            let entity = spawn_vfx_particle(
                                &mut commands,
                                &descriptor,
                                vfx_visuals.as_deref(),
                                [transform.translation.x, transform.translation.y],
                                target_xy,
                                sprite.unit_id,
                                render_sprite.flip_x,
                                transform.scale.x,
                                0.0,
                                None,
                            );
                            trace!(
                                target: "windowed.agumon_playback",
                                entity = ?entity,
                                particle = %descriptor.particle.0,
                                source_unit = ?sprite.unit_id,
                                target_xy = ?target_xy,
                                "spawned Baby Flame projectile on release"
                            );
                        } else {
                            debug!(
                                target: "windowed.agumon_playback",
                                source_unit = ?sprite.unit_id,
                                "Baby Flame projectile target could not be resolved on release"
                            );
                        }
                    }

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

fn mouth_anchor_xy(caster_xy: [f32; 2], flip_x: bool, sprite_scale: f32) -> [f32; 2] {
    let dir = if flip_x { -1.0 } else { 1.0 };
    [
        caster_xy[0] + ((VFX_MOUTH_OFFSET_X_PX * sprite_scale) * dir),
        caster_xy[1] + (VFX_MOUTH_OFFSET_Y_PX * sprite_scale),
    ]
}

fn vfx_particle_kind(descriptor: &VfxSpawnDescriptor) -> VfxParticleKind {
    match descriptor.particle.0.as_str() {
        "baby_flame_charge" => VfxParticleKind::BabyFlameCharge,
        "baby_flame_ember" => VfxParticleKind::BabyFlameEmber,
        "baby_flame_projectile" => VfxParticleKind::BabyFlameProjectile,
        "baby_flame_impact" => VfxParticleKind::BabyFlameImpact,
        "baby_flame_impact_shard" => VfxParticleKind::BabyFlameImpactShard,
        _ => VfxParticleKind::Generic,
    }
}

fn vfx_particle_ttl(kind: VfxParticleKind) -> u32 {
    match kind {
        VfxParticleKind::Generic => VFX_PARTICLE_TTL_TICKS,
        VfxParticleKind::BabyFlameCharge => 24,
        VfxParticleKind::BabyFlameEmber => BABY_FLAME_EMBER_TTL,
        // Shorter flight = the launch reads as fast and snappy.
        VfxParticleKind::BabyFlameProjectile => 4,
        VfxParticleKind::BabyFlameImpact => 2,
        VfxParticleKind::BabyFlameImpactShard => BABY_FLAME_IMPACT_SHARD_TTL,
    }
}

fn vfx_particle_size(kind: VfxParticleKind) -> f32 {
    match kind {
        VfxParticleKind::Generic => VFX_PARTICLE_SIZE,
        VfxParticleKind::BabyFlameCharge => 22.0,
        VfxParticleKind::BabyFlameEmber => BABY_FLAME_EMBER_SIZE,
        VfxParticleKind::BabyFlameProjectile => 16.0,
        VfxParticleKind::BabyFlameImpact => 26.0,
        VfxParticleKind::BabyFlameImpactShard => BABY_FLAME_IMPACT_SHARD_SIZE,
    }
}

fn vfx_particle_anchor(kind: VfxParticleKind) -> Option<VfxAnchor> {
    match kind {
        VfxParticleKind::BabyFlameCharge | VfxParticleKind::BabyFlameEmber => {
            Some(VfxAnchor::Mouth)
        }
        VfxParticleKind::Generic
        | VfxParticleKind::BabyFlameProjectile
        | VfxParticleKind::BabyFlameImpact
        | VfxParticleKind::BabyFlameImpactShard => None,
    }
}

fn baby_flame_projectile_descriptor() -> VfxSpawnDescriptor {
    VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("baby_flame_projectile".into()),
        origin: VfxLocus::CasterCenter,
        motion: VfxMotion::ArcToTarget,
    })
    .expect("SpawnParticle baby_flame_projectile must distill to a renderable VFX descriptor")
}

fn baby_flame_impact_descriptor() -> VfxSpawnDescriptor {
    VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("baby_flame_impact".into()),
        origin: VfxLocus::TargetCenter,
        motion: VfxMotion::Static,
    })
    .expect("SpawnParticle baby_flame_impact must distill to a renderable VFX descriptor")
}

fn baby_flame_ember_descriptor() -> VfxSpawnDescriptor {
    VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("baby_flame_ember".into()),
        origin: VfxLocus::CasterCenter,
        motion: VfxMotion::Static,
    })
    .expect("SpawnParticle baby_flame_ember must distill to a renderable VFX descriptor")
}

fn baby_flame_impact_shard_descriptor() -> VfxSpawnDescriptor {
    VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("baby_flame_impact_shard".into()),
        origin: VfxLocus::TargetCenter,
        motion: VfxMotion::Static,
    })
    .expect("SpawnParticle baby_flame_impact_shard must distill to a renderable VFX descriptor")
}

/// Offset (world px) of a charge ember relative to the mouth anchor at `age`
/// ticks into its life. Radius shrinks linearly toward the mouth while the
/// angle sweeps, producing an inward spiral that feeds the core flame.
fn baby_flame_ember_offset(age: u32, ttl: u32, phase: f32) -> [f32; 2] {
    let progress = if ttl == 0 {
        1.0
    } else {
        (age as f32 / ttl as f32).clamp(0.0, 1.0)
    };
    let radius = BABY_FLAME_EMBER_RADIUS_PX * (1.0 - progress);
    let angle = phase + (age as f32 * BABY_FLAME_EMBER_OMEGA);
    [radius * angle.cos(), radius * angle.sin()]
}

/// Alpha of a charge ember: bright at the rim, fading as it merges into the
/// mouth so the swirl dissolves into the core rather than popping out.
fn baby_flame_ember_alpha(age: u32, ttl: u32) -> f32 {
    let progress = if ttl == 0 {
        1.0
    } else {
        (age as f32 / ttl as f32).clamp(0.0, 1.0)
    };
    (0.9 * (1.0 - progress)).max(0.0)
}

/// Offset (world px) of an impact shard relative to the impact origin at `age`
/// ticks. Ease-out so shards burst fast then settle, reading as a dissolve.
fn baby_flame_shard_offset(age: u32, ttl: u32, phase: f32) -> [f32; 2] {
    let progress = if ttl == 0 {
        1.0
    } else {
        (age as f32 / ttl as f32).clamp(0.0, 1.0)
    };
    let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
    let dist = BABY_FLAME_IMPACT_SHARD_SPREAD_PX * eased;
    [dist * phase.cos(), dist * phase.sin()]
}

/// Alpha of an impact shard: linear fade to transparent over its life.
fn baby_flame_shard_alpha(age: u32, ttl: u32) -> f32 {
    let progress = if ttl == 0 {
        1.0
    } else {
        (age as f32 / ttl as f32).clamp(0.0, 1.0)
    };
    (0.9 * (1.0 - progress)).max(0.0)
}

#[allow(clippy::too_many_arguments)]
fn spawn_baby_flame_embers(
    commands: &mut Commands,
    visuals: Option<&VfxVisuals>,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
    source_flip_x: bool,
    source_scale: f32,
) {
    let descriptor = baby_flame_ember_descriptor();
    for i in 0..BABY_FLAME_EMBER_COUNT {
        let phase = (i as f32 / BABY_FLAME_EMBER_COUNT as f32) * std::f32::consts::TAU;
        spawn_vfx_particle(
            commands,
            &descriptor,
            visuals,
            caster_xy,
            target_xy,
            source_unit,
            source_flip_x,
            source_scale,
            phase,
            None,
        );
    }
}

/// Spawn the Baby Flame impact burst: a bright central flash plus a fan of
/// dissolving shards.
///
/// When `impact_effect` is `Some` (the `digimon/agumon/vfx.ron` data path is
/// live), the shard count and lifetime come from the asset's [`spawn_plan`];
/// otherwise the burst falls back to the hardcoded `BABY_FLAME_IMPACT_SHARD_*`
/// constants so the VFX still renders even if the asset is missing/malformed.
fn spawn_baby_flame_impact_burst(
    commands: &mut Commands,
    visuals: Option<&VfxVisuals>,
    origin_xy: [f32; 2],
    source_unit: UnitId,
    impact_effect: Option<&EffectDef>,
) {
    // Bright central flash.
    let core = baby_flame_impact_descriptor();
    spawn_vfx_particle(
        commands, &core, visuals, origin_xy, origin_xy, source_unit, false, 1.0, 0.0, None,
    );
    // Dissolve fan: shards radiate outward from the impact point and fade.
    // Count + lifetime are data-driven (vfx.ron) with a hardcoded fallback.
    let shard = baby_flame_impact_shard_descriptor();
    let (count, ttl_override) = match impact_effect {
        Some(effect) => {
            let plan = spawn_plan(effect);
            (plan.count, Some(plan.ttl_ticks))
        }
        None => (BABY_FLAME_IMPACT_SHARD_COUNT, None),
    };
    for i in 0..count {
        let phase = (i as f32 / count as f32) * std::f32::consts::TAU;
        spawn_vfx_particle(
            commands,
            &shard,
            visuals,
            origin_xy,
            origin_xy,
            source_unit,
            false,
            1.0,
            phase,
            ttl_override,
        );
    }
}

fn vfx_particle_color(descriptor: &VfxSpawnDescriptor) -> Color {
    match vfx_particle_kind(descriptor) {
        VfxParticleKind::BabyFlameCharge => Color::srgba(1.0, 0.75, 0.25, 0.9),
        VfxParticleKind::BabyFlameEmber => Color::srgba(1.0, 0.85, 0.4, 0.85),
        VfxParticleKind::BabyFlameProjectile => Color::srgba(1.0, 0.45, 0.15, 0.95),
        VfxParticleKind::BabyFlameImpact => Color::srgba(1.0, 0.82, 0.45, 0.95),
        VfxParticleKind::BabyFlameImpactShard => Color::srgba(1.0, 0.55, 0.2, 0.9),
        VfxParticleKind::Generic => {
            let name = descriptor.particle.0.as_str();
            if name.contains("burner") {
                Color::srgb(1.0, 0.8, 0.2)
            } else if name.contains("flame") {
                Color::srgb(1.0, 0.45, 0.15)
            } else {
                Color::srgb(1.0, 1.0, 1.0)
            }
        }
    }
}

fn vfx_particle_image(
    kind: VfxParticleKind,
    visuals: Option<&VfxVisuals>,
) -> Option<Handle<Image>> {
    let visuals = visuals?;
    match kind {
        VfxParticleKind::BabyFlameCharge | VfxParticleKind::BabyFlameEmber => {
            Some(visuals.baby_flame_charge.clone())
        }
        VfxParticleKind::BabyFlameProjectile => Some(visuals.baby_flame_projectile.clone()),
        VfxParticleKind::BabyFlameImpact | VfxParticleKind::BabyFlameImpactShard => {
            Some(visuals.baby_flame_impact.clone())
        }
        VfxParticleKind::Generic => None,
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

fn detonate_vfx_descriptor() -> VfxSpawnDescriptor {
    VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("baby_burner_detonate".into()),
        origin: VfxLocus::TargetCenter,
        motion: VfxMotion::Static,
    })
    .expect("SpawnParticle detonate command must distill to a renderable VFX descriptor")
}

fn find_sprite_xy(
    sprites: &Query<(&AgumonSprite, &Transform)>,
    unit_id: UnitId,
) -> Option<[f32; 2]> {
    sprites.iter().find_map(|(sprite, transform)| {
        (sprite.unit_id == unit_id).then_some([transform.translation.x, transform.translation.y])
    })
}

fn spawn_detonate_particles(
    mut commands: Commands,
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    vfx_visuals: Option<Res<VfxVisuals>>,
    sprites: Query<(&AgumonSprite, &Transform)>,
) {
    let Some(trigger) = latest_baby_burner_flash_trigger(events.read()) else {
        return;
    };

    let Some(caster_xy) = find_sprite_xy(&sprites, trigger.source) else {
        debug!(
            target: "windowed.agumon_playback",
            source_unit = ?trigger.source,
            cast_id = ?trigger.cast_id,
            "Baby Burner detonate particle source sprite could not be resolved"
        );
        return;
    };

    let descriptor = detonate_vfx_descriptor();
    for target in trigger.targets {
        let Some(target_xy) = find_sprite_xy(&sprites, target) else {
            debug!(
                target: "windowed.agumon_playback",
                source_unit = ?trigger.source,
                target_unit = ?target,
                cast_id = ?trigger.cast_id,
                particle = %descriptor.particle.0,
                "Baby Burner detonate particle target could not be resolved"
            );
            continue;
        };

        let entity = spawn_vfx_particle(
            &mut commands,
            &descriptor,
            vfx_visuals.as_deref(),
            caster_xy,
            target_xy,
            trigger.source,
            false,
            1.0,
            0.0,
            None,
        );
        let resolved_xy = resolve_vfx_spawn_xy(&descriptor, caster_xy, target_xy);
        trace!(
            target: "windowed.agumon_playback",
            entity = ?entity,
            cast_id = ?trigger.cast_id,
            particle = %descriptor.particle.0,
            source_unit = ?trigger.source,
            target_unit = ?target,
            resolved_xy = ?resolved_xy,
            motion = ?descriptor.motion,
            ttl_ticks = VFX_PARTICLE_TTL_TICKS,
            "spawned Baby Burner detonate particle"
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_vfx_particle(
    commands: &mut Commands,
    descriptor: &VfxSpawnDescriptor,
    visuals: Option<&VfxVisuals>,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
    source_flip_x: bool,
    source_scale: f32,
    phase: f32,
    ttl_override: Option<u32>,
) -> Entity {
    let kind = vfx_particle_kind(descriptor);
    let resolved_xy = match vfx_particle_anchor(kind) {
        Some(VfxAnchor::Mouth) => mouth_anchor_xy(caster_xy, source_flip_x, source_scale),
        None => resolve_vfx_spawn_xy(descriptor, caster_xy, target_xy),
    };
    // Data-driven effects override the kind's default lifetime with the asset's
    // `ttl_ticks`; everything else keeps the hardcoded per-kind default.
    let ttl_ticks = ttl_override.unwrap_or_else(|| vfx_particle_ttl(kind));
    let sprite = if let Some(image) = vfx_particle_image(kind, visuals) {
        Sprite {
            image,
            custom_size: Some(Vec2::splat(vfx_particle_size(kind))),
            color: vfx_particle_color(descriptor),
            ..default()
        }
    } else {
        Sprite::from_color(
            vfx_particle_color(descriptor),
            Vec2::splat(vfx_particle_size(kind)),
        )
    };
    commands
        .spawn((
            sprite,
            Transform::from_xyz(resolved_xy[0], resolved_xy[1], VFX_PARTICLE_Z),
            VfxParticle {
                ttl_ticks,
                age_ticks: 0,
                motion: descriptor.motion.clone(),
                kind,
                anchor: vfx_particle_anchor(kind),
                phase,
            },
            VfxParticleTarget {
                world_xy: target_xy,
            },
            VfxParticleSource {
                unit_id: source_unit,
            },
        ))
        .id()
}

#[allow(clippy::too_many_arguments)]
fn advance_vfx_particles(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    vfx_visuals: Option<Res<VfxVisuals>>,
    agumon_vfx: Option<Res<AgumonVfx>>,
    vfx_assets: Option<Res<Assets<VfxAsset>>>,
    mut warned_missing_impact: Local<bool>,
    source_sprites: Query<(&AgumonSprite, &Sprite, &Transform), Without<VfxParticle>>,
    mut particles: Query<
        (
            Entity,
            &mut VfxParticle,
            &mut Sprite,
            &mut Transform,
            &VfxParticleTarget,
            &VfxParticleSource,
        ),
        Without<AgumonSprite>,
    >,
) {
    // Resolve the Baby Flame impact effect from the owned vfx.ron data path once
    // per frame. `None` (asset not yet loaded, load failed, or effect id absent)
    // means the burst + shard updates below use the hardcoded fallback. A loaded
    // asset that is missing the effect id is warned once (load failure itself is
    // surfaced by `diagnose_agumon_vfx_load`).
    let impact_effect: Option<&EffectDef> = match agumon_vfx
        .as_ref()
        .zip(vfx_assets.as_ref())
        .and_then(|(vfx, assets)| assets.get(&vfx.handle))
    {
        Some(asset) => {
            let effect = resolve_effect(asset, AGUMON_IMPACT_EFFECT_ID);
            if effect.is_none() && !*warned_missing_impact {
                warn!(
                    target: "windowed.agumon_playback",
                    effect_id = AGUMON_IMPACT_EFFECT_ID,
                    reason = "effect id absent from loaded vfx.ron",
                    "falling back to hardcoded Baby Flame impact path"
                );
                *warned_missing_impact = true;
            }
            effect
        }
        None => None,
    };

    for _ in 0..pending_ticks.0 {
        for (entity, mut particle, mut sprite, mut transform, target, source) in &mut particles {
            if matches!(particle.anchor, Some(VfxAnchor::Mouth)) {
                if let Some((_, source_sprite, source_transform)) = source_sprites
                    .iter()
                    .find(|(agumon, _, _)| agumon.unit_id == source.unit_id)
                {
                    let source_xy = [
                        source_transform.translation.x,
                        source_transform.translation.y,
                    ];
                    let anchored_xy =
                        mouth_anchor_xy(source_xy, source_sprite.flip_x, source_transform.scale.x);
                    transform.translation.x = anchored_xy[0];
                    transform.translation.y = anchored_xy[1];
                    if particle.kind == VfxParticleKind::BabyFlameEmber {
                        let off = baby_flame_ember_offset(
                            particle.age_ticks,
                            BABY_FLAME_EMBER_TTL,
                            particle.phase,
                        );
                        transform.translation.x += off[0];
                        transform.translation.y += off[1];
                    }
                }
                match particle.kind {
                    VfxParticleKind::BabyFlameCharge => {
                        let growth = (particle.age_ticks as f32).min(6.0) / 6.0;
                        let pulse_phase = (particle.age_ticks % 4) as f32;
                        let pulse = match pulse_phase as u32 {
                            0 => 0.0,
                            1 => 0.03,
                            2 => 0.06,
                            _ => 0.03,
                        };
                        let scale = 0.42 + (growth * 0.48) + pulse;
                        transform.scale = Vec3::splat(scale);

                        let alpha = 0.35 + (growth * 0.45) + (pulse * 0.6);
                        sprite.color.set_alpha(alpha.min(0.88));
                    }
                    VfxParticleKind::BabyFlameEmber => {
                        transform.scale = Vec3::ONE;
                        sprite.color.set_alpha(baby_flame_ember_alpha(
                            particle.age_ticks,
                            BABY_FLAME_EMBER_TTL,
                        ));
                    }
                    _ => {
                        transform.scale = Vec3::ONE;
                        sprite.color.set_alpha(1.0);
                    }
                }
            }

            match &particle.motion {
                VfxMotion::Static => {}
                VfxMotion::FollowTarget | VfxMotion::ArcToTarget => {
                    let dx = target.world_xy[0] - transform.translation.x;
                    let dy = target.world_xy[1] - transform.translation.y;
                    let factor = match &particle.motion {
                        VfxMotion::FollowTarget => 0.45,
                        // Faster lerp toward the target = the flame visibly rips
                        // across the gap instead of drifting.
                        VfxMotion::ArcToTarget => 0.55,
                        VfxMotion::Static => 0.0,
                    };
                    transform.translation.x += dx * factor;
                    transform.translation.y += dy * factor;
                }
            }

            if particle.kind == VfxParticleKind::BabyFlameImpactShard {
                match impact_effect {
                    // Data path: outward fraction + rgba come from the asset's
                    // curves, the outward distance from its `spread_px` (R004
                    // pure eval). This replaces the hardcoded offset/alpha math
                    // for this effect only.
                    Some(effect) => {
                        let plan = spawn_plan(effect);
                        let progress = if plan.ttl_ticks == 0 {
                            1.0
                        } else {
                            (particle.age_ticks as f32 / plan.ttl_ticks as f32).clamp(0.0, 1.0)
                        };
                        let frac = eval_scale(&effect.appearance.scale, progress);
                        let dist = plan.spread_px * frac;
                        transform.translation.x = target.world_xy[0] + dist * particle.phase.cos();
                        transform.translation.y = target.world_xy[1] + dist * particle.phase.sin();
                        let rgba = eval_color(&effect.appearance.color, progress);
                        sprite.color = Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3]);
                    }
                    // Fallback: original hardcoded ease-out + linear alpha fade.
                    None => {
                        let off = baby_flame_shard_offset(
                            particle.age_ticks,
                            BABY_FLAME_IMPACT_SHARD_TTL,
                            particle.phase,
                        );
                        transform.translation.x = target.world_xy[0] + off[0];
                        transform.translation.y = target.world_xy[1] + off[1];
                        sprite.color.set_alpha(baby_flame_shard_alpha(
                            particle.age_ticks,
                            BABY_FLAME_IMPACT_SHARD_TTL,
                        ));
                    }
                }
            }

            particle.age_ticks += 1;
            particle.ttl_ticks = decrement_vfx_ttl(particle.ttl_ticks);
            if particle.ttl_ticks == 0 {
                if particle.kind == VfxParticleKind::BabyFlameProjectile {
                    spawn_baby_flame_impact_burst(
                        &mut commands,
                        vfx_visuals.as_deref(),
                        [transform.translation.x, transform.translation.y],
                        source.unit_id,
                        impact_effect,
                    );
                }
                trace!(
                    target: "windowed.agumon_playback",
                    entity = ?entity,
                    motion = ?particle.motion,
                    source_unit = ?source.unit_id,
                    kind = ?particle.kind,
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
        assert_eq!(
            resolve_vfx_spawn_xy(&caster_center, caster_xy, target_xy),
            caster_xy
        );

        let target_center = VfxSpawnDescriptor {
            particle: ParticleId("baby_flame".into()),
            locus: VfxLocus::TargetCenter,
            motion: VfxMotion::FollowTarget,
        };
        assert_eq!(
            resolve_vfx_spawn_xy(&target_center, caster_xy, target_xy),
            target_xy
        );

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
    fn mouth_anchor_offsets_follow_sprite_facing() {
        let center = [10.0, 20.0];
        let left = mouth_anchor_xy(center, false, SPRITE_DISPLAY_SCALE);
        let right = mouth_anchor_xy(center, true, SPRITE_DISPLAY_SCALE);
        assert!((left[0] - 46.8).abs() < 0.0001);
        assert!((left[1] - 29.6).abs() < 0.0001);
        assert!((right[0] - -26.8).abs() < 0.0001);
        assert!((right[1] - 29.6).abs() < 0.0001);
    }

    #[test]
    fn baby_flame_particle_specs_split_charge_projectile_and_impact() {
        let charge = VfxSpawnDescriptor {
            particle: bevyrogue::animation::ParticleId("baby_flame_charge".into()),
            locus: VfxLocus::CasterCenter,
            motion: VfxMotion::Static,
        };
        let projectile = baby_flame_projectile_descriptor();
        let impact = baby_flame_impact_descriptor();

        assert_eq!(vfx_particle_kind(&charge), VfxParticleKind::BabyFlameCharge);
        assert_eq!(
            vfx_particle_kind(&projectile),
            VfxParticleKind::BabyFlameProjectile
        );
        assert_eq!(vfx_particle_kind(&impact), VfxParticleKind::BabyFlameImpact);
        assert_eq!(
            vfx_particle_anchor(vfx_particle_kind(&charge)),
            Some(VfxAnchor::Mouth)
        );
        assert_eq!(vfx_particle_ttl(vfx_particle_kind(&projectile)), 4);
        assert_eq!(vfx_particle_ttl(vfx_particle_kind(&impact)), 2);
    }

    #[test]
    fn baby_flame_polish_kinds_map_anchor_and_reuse_existing_sprites() {
        let ember = VfxSpawnDescriptor {
            particle: bevyrogue::animation::ParticleId("baby_flame_ember".into()),
            locus: VfxLocus::CasterCenter,
            motion: VfxMotion::Static,
        };
        let shard = baby_flame_impact_shard_descriptor();

        assert_eq!(vfx_particle_kind(&ember), VfxParticleKind::BabyFlameEmber);
        assert_eq!(
            vfx_particle_kind(&shard),
            VfxParticleKind::BabyFlameImpactShard
        );
        // Embers ride the mouth like the charge core; shards burst free.
        assert_eq!(
            vfx_particle_anchor(VfxParticleKind::BabyFlameEmber),
            Some(VfxAnchor::Mouth)
        );
        assert_eq!(
            vfx_particle_anchor(VfxParticleKind::BabyFlameImpactShard),
            None
        );
        assert_eq!(
            vfx_particle_ttl(VfxParticleKind::BabyFlameEmber),
            BABY_FLAME_EMBER_TTL
        );
        assert_eq!(
            vfx_particle_ttl(VfxParticleKind::BabyFlameImpactShard),
            BABY_FLAME_IMPACT_SHARD_TTL
        );
    }

    #[test]
    fn ember_spirals_inward_and_fades() {
        // At birth the ember sits on the outer rim; by end of life it has
        // collapsed onto the mouth and gone transparent.
        let start = baby_flame_ember_offset(0, BABY_FLAME_EMBER_TTL, 0.0);
        let start_radius = (start[0] * start[0] + start[1] * start[1]).sqrt();
        assert!((start_radius - BABY_FLAME_EMBER_RADIUS_PX).abs() < 0.001);

        let end = baby_flame_ember_offset(
            BABY_FLAME_EMBER_TTL,
            BABY_FLAME_EMBER_TTL,
            0.0,
        );
        let end_radius = (end[0] * end[0] + end[1] * end[1]).sqrt();
        assert!(end_radius < 0.001, "ember should reach the mouth: {end_radius}");

        assert!(baby_flame_ember_alpha(0, BABY_FLAME_EMBER_TTL) > 0.8);
        assert!(baby_flame_ember_alpha(BABY_FLAME_EMBER_TTL, BABY_FLAME_EMBER_TTL) < 0.001);
    }

    #[test]
    fn shard_fans_outward_along_phase_and_fades() {
        // A shard travels along its phase direction, reaching full spread by the
        // end of its life, while its alpha decays to zero.
        let mid = baby_flame_shard_offset(0, BABY_FLAME_IMPACT_SHARD_TTL, 0.0);
        assert!(mid[0].abs() < 0.001 && mid[1].abs() < 0.001, "starts at origin");

        let end = baby_flame_shard_offset(
            BABY_FLAME_IMPACT_SHARD_TTL,
            BABY_FLAME_IMPACT_SHARD_TTL,
            0.0,
        );
        assert!((end[0] - BABY_FLAME_IMPACT_SHARD_SPREAD_PX).abs() < 0.001);
        assert!(end[1].abs() < 0.001, "phase 0 fans along +x");

        // A quarter-turn phase fans along +y instead.
        let quarter = baby_flame_shard_offset(
            BABY_FLAME_IMPACT_SHARD_TTL,
            BABY_FLAME_IMPACT_SHARD_TTL,
            std::f32::consts::FRAC_PI_2,
        );
        assert!(quarter[0].abs() < 0.01 && quarter[1] > BABY_FLAME_IMPACT_SHARD_SPREAD_PX - 0.01);

        assert!(baby_flame_shard_alpha(0, BABY_FLAME_IMPACT_SHARD_TTL) > 0.8);
        assert!(
            baby_flame_shard_alpha(BABY_FLAME_IMPACT_SHARD_TTL, BABY_FLAME_IMPACT_SHARD_TTL) < 0.001
        );
    }

    #[test]
    fn detonate_descriptor_is_renderable_and_serializes_without_numeric_payload() {
        let descriptor = detonate_vfx_descriptor();
        assert!(descriptor.is_renderable());
        assert_eq!(descriptor.locus, VfxLocus::TargetCenter);
        assert_eq!(descriptor.motion, VfxMotion::Static);

        let command = Command::SpawnParticle {
            name: ParticleId("baby_burner_flash".into()),
            origin: descriptor.locus.clone(),
            motion: descriptor.motion.clone(),
        };
        let serialized = serde_json::to_string(&command).expect("SpawnParticle serializes");
        assert!(
            !serialized.chars().any(|ch| ch.is_ascii_digit()),
            "serialized SpawnParticle should not carry numeric gameplay payload: {serialized}"
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
