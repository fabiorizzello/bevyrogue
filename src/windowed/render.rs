use std::collections::HashSet;

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_common_assets::ron::RonAssetPlugin;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles, AnimationClipLoadState,
    AnimationGraphLookupDiagnostics, AtlasGeometry, Clip, EffectId, FrameCueCommand,
    NodeId, PlacementAnchor, PlacementCtx, PlacementParams, ResolvedAnimGraph, ResolvedAnimGraphSource,
    SkillGraphRegistry, StanceGraphRegistry, StanceReaction, VfxAsset, VfxMotion, VfxSpawnDescriptor,
    eval_color, eval_rotation, eval_scale, resolve_effect, stance_reaction_for,
};
use bevyrogue::combat::runtime::{
    CueBarrierStatus, CueReleaseResult, ExtRegistries, SuspendedTimelineState,
};
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
    /// Retained per the T03 contract; position is now driven by the resolved
    /// placement verb, so every data-driven spawn carries `Static` here.
    motion: VfxMotion,
    /// Owned effect id (resolved at spawn). The per-tick dispatcher resolves it
    /// against the loaded `VfxAsset` to drive placement + appearance.
    effect_id: EffectId,
    /// World-space anchor the resolved placement offset is applied relative to,
    /// copied from the resolved effect's `placement.anchor` at spawn.
    anchor: PlacementAnchor,
    /// Per-particle seed angle (radians). Swirls use it as the starting orbit
    /// angle; fan-out bursts use it as the radial direction. 0.0 for single-quad
    /// effects, which derive everything from `age_ticks`.
    phase: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct VfxParticleTarget {
    world_xy: [f32; 2],
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct VfxParticleSource {
    unit_id: UnitId,
}

/// Terminal marker: this sprite has been seeded into its `death` stance node and
/// is exiting the field. Kept as a *separate* component rather than a new
/// `AgumonPlaybackMode` variant so the `mode` match arms in `sync_agumon_mode` /
/// `classify_same_skill_sync` stay closed (S02-RESEARCH). Its presence tells
/// `advance_agumon_presentation` to (a) skip mode reconciliation so a still-active
/// barrier cannot re-`start_skill` the dying sprite, and (b) leave the sprite on
/// its final death frame instead of idle-restoring it on node exit. The fade-out
/// driver (T02) consumes this marker to lerp the sprite out and despawn it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct DeathExiting;

/// Off-field exit fader. Seeded by `advance_agumon_presentation` on the frame the
/// `death` node finishes (a [`DeathExiting`] sprite exits its node). Driven by
/// [`advance_death_fade`], which lerps `Sprite.color` alpha from 1.0 to 0.0 over
/// `total_ticks` animation ticks and despawns the entity when it reaches 0 — the
/// KO'd unit fades off the field only AFTER its authored death frames play out,
/// preserving the M002 post-KO overshoot observability rather than instant-despawning.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct FadeOut {
    remaining_ticks: u32,
    total_ticks: u32,
}

/// Animation ticks the off-field fade takes once the death node completes. A few
/// ticks at the 12fps animation clock — long enough to read as a fade, short
/// enough not to clutter the field.
const DEATH_FADE_TICKS: u32 = 8;

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

    /// Seed the player at a stance-reaction node (S01: `hurt`) within the stance
    /// graph. Mirrors the `start_skill` / `return_to_idle` seeding pattern, but
    /// keeps `mode` at `Idle`: a stance reaction is a transient detour inside the
    /// stance graph, not a skill cast. The authored `hurt -> idle` TimeInNode
    /// transition returns the player to idle once the hurt frames complete, so a
    /// dropped/duplicated event degrades to "stays idle" rather than a stuck frame.
    fn drive_stance_reaction(&mut self, node: NodeId, stance_graph: ResolvedAnimGraph) {
        self.player = AnimGraphPlayer::new(node);
        self.graph = stance_graph;
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
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
    // Composable "atom" set (S06): single-element glow primitives the particle
    // engine layers + rotates into the full effect. `baby_flame_impact` and
    // `sharp_claws_slash` are retained for the baby_burner.* + sharp_claws paths.
    flame_core: Handle<Image>,
    flame_spark: Handle<Image>,
    flame_streak: Handle<Image>,
    flame_orb: Handle<Image>,
    flame_tall: Handle<Image>,
    flame_comet: Handle<Image>,
    flame_burst: Handle<Image>,
    flame_ring: Handle<Image>,
    baby_flame_impact: Handle<Image>,
    sharp_claws_slash: Handle<Image>,
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
const VFX_PARTICLE_Z: f32 = 1.0;
const VFX_MOUTH_OFFSET_X_PX: f32 = 92.0;
const VFX_MOUTH_OFFSET_Y_PX: f32 = 24.0;
// All per-effect appearance values (count, ttl, size, spread, scale/color curves,
// texture) now live in `assets/digimon/agumon/vfx.ron` and are read through the
// `VfxAsset` schema; the hardcoded Baby Flame polish constants the per-kind
// dispatcher used were deleted in T03.

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
                drive_hurt_reactions
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_agumon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // AFTER the hurt driver enforces death-precedence: a target both
                // struck and killed in one window resolves to `death`, not `hurt`.
                drive_death_reactions
                    .after(drive_hurt_reactions)
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_agumon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                (
                    advance_vfx_particles,
                    advance_agumon_presentation,
                    advance_death_fade,
                )
                    .chain()
                    .after(sample_animation_ticks)
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
    ));
}

fn load_vfx_visuals(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(VfxVisuals {
        flame_core: asset_server.load("vfx/flame_core.png"),
        flame_spark: asset_server.load("vfx/flame_spark.png"),
        flame_streak: asset_server.load("vfx/flame_streak.png"),
        flame_orb: asset_server.load("vfx/flame_orb.png"),
        flame_tall: asset_server.load("vfx/flame_tall.png"),
        flame_comet: asset_server.load("vfx/flame_comet.png"),
        flame_burst: asset_server.load("vfx/flame_burst.png"),
        flame_ring: asset_server.load("vfx/flame_ring.png"),
        baby_flame_impact: asset_server.load("vfx/baby_flame_impact.png"),
        sharp_claws_slash: asset_server.load("vfx/sharp_claws_slash.png"),
    });
}

/// Path (relative to `assets/`) of Agumon's owned VFX asset.
const AGUMON_VFX_PATH: &str = "digimon/agumon/vfx.ron";
/// Namespaced effect ids of the five Baby Flame effects within the asset. The
/// dispatcher resolves each through `resolve_effect` to drive placement +
/// appearance; nothing is keyed off a hardcoded VFX kind any more (T03).
const AGUMON_CHARGE_EFFECT_ID: &str = "baby_flame.charge";
const AGUMON_EMBER_EFFECT_ID: &str = "baby_flame.ember";
const AGUMON_PROJECTILE_EFFECT_ID: &str = "baby_flame.projectile";
const AGUMON_IMPACT_EFFECT_ID: &str = "baby_flame.impact";
#[allow(dead_code)] // reachable only via projectile `on_expire`; named for clarity.
const AGUMON_IMPACT_FLASH_EFFECT_ID: &str = "baby_flame.impact_flash";
/// Baby Burner detonate flash. Out of scope for S02's data-port (the full Baby
/// Burner port is S03), but routed through the unified effect path with a minimal
/// owned effect so it keeps rendering after VfxParticleKind was deleted.
const AGUMON_DETONATE_EFFECT_ID: &str = "baby_burner.detonate";
/// Sharp Claws slash. Owned, data-driven effect (M004/S05): a single
/// target-anchored streak spawned on the `sharp_claws_strike` node enter,
/// resolved through the same `resolve_effect` path as every other effect.
const AGUMON_SHARP_CLAWS_EFFECT_ID: &str = "sharp_claws.slash";

/// Handle to Agumon's owned `VfxAsset` (M004/S01). Held in a resource so every
/// Baby Flame effect can source its placement verb + appearance curves from data.
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
/// so no effect ever resolves and every Baby Flame particle is skipped; there is
/// no hardcoded fallback path any more (T03). This makes the dead VFX visible
/// rather than mysterious (slice failure-visibility).
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
            path = AGUMON_VFX_PATH,
            reason = "vfx.ron failed to load or parse",
            "Baby Flame VFX disabled; no owned effects could resolve"
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
    agumon_vfx: Option<Res<AgumonVfx>>,
    vfx_assets: Option<Res<Assets<VfxAsset>>>,
    vfx_particles: Query<(Entity, &VfxParticle, &VfxParticleSource)>,
    mut sprites: ParamSet<(
        Query<(
            Entity,
            &mut AgumonSprite,
            &mut Sprite,
            &Transform,
            Option<&DeathExiting>,
            Option<&FadeOut>,
        )>,
        Query<(&AgumonSprite, &Transform)>,
    )>,
) {
    let stance_graph =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs);

    // The loaded owned asset: every Baby Flame spawn site reads its effects from
    // here. `None` (not yet loaded / load failed) means no effect resolves, so
    // nothing spawns — surfaced once by `diagnose_agumon_vfx_load`.
    let vfx_asset = agumon_vfx
        .as_ref()
        .zip(vfx_assets.as_ref())
        .and_then(|(vfx, assets)| assets.get(&vfx.handle));

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
        for (entity, mut sprite, mut render_sprite, transform, death_exiting, fade_out) in
            &mut sprites.p0()
        {
            let prev_node = sprite.player.current_node.0.clone();

            // A dying sprite is resting on (or playing out) its death node. Skip
            // mode reconciliation entirely: a still-active kernel barrier must not
            // re-`start_skill` the dying caster back into its interrupted skill.
            if death_exiting.is_none() {
                sync_agumon_mode(
                    &mut sprite,
                    active_barrier.as_ref(),
                    &skill_reg,
                    &stance_reg,
                    &graphs,
                    &mut lookup_diagnostics,
                );
            }

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

                            // Map the authored particle name to the owned effect
                            // id(s) it spawns (the charge command also seeds the
                            // inward ember swirl). This name->effect map at the
                            // spawn boundary replaces VfxParticleKind dispatch.
                            let Some(asset) = vfx_asset else {
                                debug!(
                                    target: "windowed.agumon_playback",
                                    source_unit = ?sprite.unit_id,
                                    particle = %descriptor.particle.0,
                                    "vfx.ron not loaded; on_enter VFX skipped"
                                );
                                continue;
                            };
                            for effect_id in on_enter_effect_ids(descriptor.particle.0.as_str()) {
                                let spawned = spawn_effect_by_id(
                                    &mut commands,
                                    asset,
                                    effect_id,
                                    vfx_visuals.as_deref(),
                                    caster_xy,
                                    target_xy,
                                    sprite.unit_id,
                                    render_sprite.flip_x,
                                    transform.scale.x,
                                );
                                trace!(
                                    target: "windowed.agumon_playback",
                                    effect_id,
                                    spawned,
                                    caster_xy = ?caster_xy,
                                    source_unit = ?sprite.unit_id,
                                    "spawned windowed vfx effect on node enter"
                                );
                            }
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
                        // Despawn the charge orb + ember swirl by effect-id
                        // membership (not VfxParticleKind) the instant the flame
                        // launches, so the mouth clears for the projectile.
                        for (entity, particle, source) in &vfx_particles {
                            if source.unit_id == sprite.unit_id
                                && matches!(
                                    particle.effect_id.0.as_str(),
                                    AGUMON_CHARGE_EFFECT_ID | AGUMON_EMBER_EFFECT_ID
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
                            if let Some(asset) = vfx_asset {
                                let spawned = spawn_effect_by_id(
                                    &mut commands,
                                    asset,
                                    AGUMON_PROJECTILE_EFFECT_ID,
                                    vfx_visuals.as_deref(),
                                    [transform.translation.x, transform.translation.y],
                                    target_xy,
                                    sprite.unit_id,
                                    render_sprite.flip_x,
                                    transform.scale.x,
                                );
                                trace!(
                                    target: "windowed.agumon_playback",
                                    effect_id = AGUMON_PROJECTILE_EFFECT_ID,
                                    spawned,
                                    source_unit = ?sprite.unit_id,
                                    target_xy = ?target_xy,
                                    "spawned Baby Flame projectile on release"
                                );
                            } else {
                                debug!(
                                    target: "windowed.agumon_playback",
                                    source_unit = ?sprite.unit_id,
                                    "vfx.ron not loaded; Baby Flame projectile skipped on release"
                                );
                            }
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
                if death_exiting.is_some() {
                    // The death node has played out. Never idle-restore a dying
                    // sprite — instead seed the fade-out so it lerps off the field
                    // and despawns (advance_death_fade). Insert FadeOut once: the
                    // death node exits a single time, but guard defensively against
                    // re-entry while the marker is still present.
                    if fade_out.is_none() {
                        commands.entity(entity).insert(FadeOut {
                            remaining_ticks: DEATH_FADE_TICKS,
                            total_ticks: DEATH_FADE_TICKS,
                        });
                    }
                    trace!(
                        target: "windowed.agumon_playback",
                        unit_id = ?sprite.unit_id,
                        node = sprite.player.current_node.0.as_str(),
                        fade_ticks = DEATH_FADE_TICKS,
                        "death node exited; seeding fade-out off field (idle restore suppressed)"
                    );
                } else if let Some(stance_graph) = stance_graph.clone() {
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

/// Bridge the combat event bus to the struck sprite's `hurt` stance reaction.
///
/// For each `CombatEvent` that the pure lib mapping ([`stance_reaction_for`])
/// classifies as [`StanceReaction::Hurt`], drive the *target* unit's sprite into
/// its `hurt` stance node. This is the visible half of S01: in `cargo winx`,
/// hitting either combatant makes that sprite flinch (frames 46–52) then return
/// to idle via the authored `hurt -> idle` transition.
///
/// S01 scope guards:
/// - Only `Hurt` is handled here. `Death` (also classified by the lib mapping)
///   is deliberately left for S02 — it is never driven from this system.
/// - Only an idle sprite reacts. An in-flight skill cast on the struck unit is
///   never interrupted (S01 assumption: mid-cast hurt is out of scope).
///
/// Reads events and writes presentation components only; it never mutates
/// combat or kernel state (R010). A dropped or duplicated event degrades to
/// "stays idle" via the existing `hurt -> idle` transition rather than a stuck
/// frame.
fn drive_hurt_reactions(
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut sprites: Query<&mut AgumonSprite>,
) {
    // Dedup by target: a unit struck twice within the same window still plays
    // `hurt` once. `Death` and every non-reaction event resolve to `None` here
    // and are filtered out — only `Hurt` survives.
    let struck: HashSet<UnitId> = events
        .read()
        .filter(|event| stance_reaction_for(&event.kind) == Some(StanceReaction::Hurt))
        .map(|event| event.target)
        .collect();
    if struck.is_empty() {
        return;
    }

    let Some(stance_graph) =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs)
    else {
        return;
    };
    let hurt_node = StanceReaction::Hurt.stance_node();

    for mut sprite in &mut sprites {
        if !struck.contains(&sprite.unit_id) {
            continue;
        }
        // Do not interrupt an in-flight skill cast on the struck unit (S01).
        if !matches!(sprite.mode, AgumonPlaybackMode::Idle) {
            trace!(
                target: "windowed.agumon_playback",
                unit_id = ?sprite.unit_id,
                reaction = ?StanceReaction::Hurt,
                node = hurt_node.0.as_str(),
                mode = ?sprite.mode,
                "hurt reaction skipped; struck sprite mid-cast (in-flight skill left uninterrupted)"
            );
            continue;
        }
        sprite.drive_stance_reaction(hurt_node.clone(), stance_graph.clone());
        trace!(
            target: "windowed.agumon_playback",
            unit_id = ?sprite.unit_id,
            reaction = ?StanceReaction::Hurt,
            node = hurt_node.0.as_str(),
            "drove struck sprite into hurt stance node"
        );
    }
}

/// `true` iff the pure lib mapping classifies this event kind as a death
/// reaction. The death pipeline gates on this; a non-death event (e.g.
/// `OnHitTaken`) must never enter it (Q7 negative test).
fn is_death_reaction(kind: &bevyrogue::combat::events::CombatEventKind) -> bool {
    stance_reaction_for(kind) == Some(StanceReaction::Death)
}

/// Bridge the combat event bus to the struck sprite's `death` stance reaction.
///
/// For each `CombatEvent` the pure lib mapping ([`stance_reaction_for`])
/// classifies as [`StanceReaction::Death`], drive the *target* unit's sprite
/// into its `death` node — the visible half of S02. Unlike [`drive_hurt_reactions`]
/// this is *un-gated by mode*: death interrupts an in-flight skill cast. The
/// dying sprite is also tagged [`DeathExiting`] so `advance_agumon_presentation`
/// skips mode reconciliation (a still-active barrier cannot re-`start_skill` it)
/// and leaves it resting on its final death frame instead of idle-restoring.
///
/// Registered AFTER `drive_hurt_reactions`, enforcing death-precedence: a target
/// both struck and killed in one window resolves to `death`, never `hurt`.
///
/// Reads events and writes presentation components/entities only; it never
/// mutates combat or kernel state (R010). A dropped/duplicated `UnitDied`
/// degrades to "stays on the death frame" (idempotent marker insert, no stuck
/// frame); a death event for a unit with no live sprite is a no-op.
fn drive_death_reactions(
    mut commands: Commands,
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut sprites: Query<(Entity, &mut AgumonSprite)>,
) {
    // Dedup by target: a unit reported dead more than once in the same window
    // still plays `death` once. Only `Death` survives the filter.
    let dying: HashSet<UnitId> = events
        .read()
        .filter(|event| is_death_reaction(&event.kind))
        .map(|event| event.target)
        .collect();
    if dying.is_empty() {
        return;
    }

    let Some(stance_graph) =
        stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs)
    else {
        return;
    };
    let death_node = StanceReaction::Death.stance_node();

    for (entity, mut sprite) in &mut sprites {
        if !dying.contains(&sprite.unit_id) {
            continue;
        }
        let prior_mode = sprite.mode.clone();
        // Death interrupts any in-flight skill: drive unconditionally (no
        // `matches!(mode, Idle)` guard, unlike the hurt path).
        sprite.drive_stance_reaction(death_node.clone(), stance_graph.clone());
        commands.entity(entity).insert(DeathExiting);
        trace!(
            target: "windowed.agumon_playback",
            unit_id = ?sprite.unit_id,
            reaction = ?StanceReaction::Death,
            node = death_node.0.as_str(),
            prior_mode = ?prior_mode,
            "drove struck sprite into death stance node (skill interrupt; marked DeathExiting)"
        );
    }
}

fn entered_node<'a>(prev_node: &'a str, current_node: &'a str) -> Option<&'a str> {
    (prev_node != current_node).then_some(current_node)
}

fn decrement_vfx_ttl(ttl_ticks: u32) -> u32 {
    ttl_ticks.saturating_sub(1)
}

/// Linear fade alpha for the off-field death exit: `1.0` at full `remaining_ticks`,
/// `0.0` once spent. `total_ticks == 0` saturates to `1.0` (the `.max(1)` guards
/// the divide), so a zero-length fade never divides by zero (Q5).
fn fade_alpha(remaining_ticks: u32, total_ticks: u32) -> f32 {
    (remaining_ticks as f32 / total_ticks.max(1) as f32).clamp(0.0, 1.0)
}

/// Lerp a [`FadeOut`] sprite's alpha to 0 over its `total_ticks`, then despawn it.
///
/// Runs on the same `PendingAnimationTicks` clock as the presentation chain and is
/// ordered strictly after `advance_agumon_presentation`, so a sprite seeded with
/// `FadeOut` in this frame's death-exit branch begins fading on the next tick.
/// Writes only `Sprite.color` and despawn — strictly downstream of presentation,
/// never feeding the kernel (R004). An entity despawned by another path mid-fade
/// simply stops being yielded by the query (no panic, Q5).
fn advance_death_fade(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    mut faders: Query<(Entity, &mut FadeOut, &mut Sprite)>,
) {
    for _ in 0..pending_ticks.0 {
        for (entity, mut fade, mut sprite) in &mut faders {
            fade.remaining_ticks = fade.remaining_ticks.saturating_sub(1);
            let alpha = fade_alpha(fade.remaining_ticks, fade.total_ticks);
            let rgba = sprite.color.to_linear();
            sprite.color = Color::linear_rgba(rgba.red, rgba.green, rgba.blue, alpha);
            if fade.remaining_ticks == 0 {
                trace!(
                    target: "windowed.agumon_playback",
                    ?entity,
                    total_ticks = fade.total_ticks,
                    "death fade complete; despawning sprite off field"
                );
                commands.entity(entity).despawn();
            }
        }
    }
}

fn mouth_anchor_xy(caster_xy: [f32; 2], flip_x: bool, sprite_scale: f32) -> [f32; 2] {
    let dir = if flip_x { -1.0 } else { 1.0 };
    [
        caster_xy[0] + ((VFX_MOUTH_OFFSET_X_PX * sprite_scale) * dir),
        caster_xy[1] + (VFX_MOUTH_OFFSET_Y_PX * sprite_scale),
    ]
}

/// Map an authored `SpawnParticle` name (from the anim graph `on_enter` command)
/// to the owned effect id(s) it spawns. The charge command also seeds the inward
/// ember swirl. This name->effect map at the spawn boundary is what replaced
/// VfxParticleKind dispatch (T03): it is NOT kind resolution — every spawned
/// particle is thereafter driven entirely by its resolved `EffectDef`.
fn on_enter_effect_ids(particle_name: &str) -> &'static [&'static str] {
    match particle_name {
        "baby_flame_charge" => &[AGUMON_CHARGE_EFFECT_ID, AGUMON_EMBER_EFFECT_ID],
        "baby_flame_projectile" => &[AGUMON_PROJECTILE_EFFECT_ID],
        "baby_flame_impact" => &[AGUMON_IMPACT_EFFECT_ID],
        "sharp_claws_slash" => &[AGUMON_SHARP_CLAWS_EFFECT_ID],
        _ => &[],
    }
}

/// Resolve an `Appearance.texture` key to a windowed image handle. A small
/// string->handle map (NOT VfxParticleKind dispatch): an unknown/empty key
/// resolves to `None` so the particle renders as a flat-color quad. Headless
/// code never calls this (R016).
fn vfx_texture_handle(key: &str, visuals: Option<&VfxVisuals>) -> Option<Handle<Image>> {
    let visuals = visuals?;
    match key {
        "flame_core" => Some(visuals.flame_core.clone()),
        "flame_spark" => Some(visuals.flame_spark.clone()),
        "flame_streak" => Some(visuals.flame_streak.clone()),
        "flame_orb" => Some(visuals.flame_orb.clone()),
        "flame_tall" => Some(visuals.flame_tall.clone()),
        "flame_comet" => Some(visuals.flame_comet.clone()),
        "flame_burst" => Some(visuals.flame_burst.clone()),
        "flame_ring" => Some(visuals.flame_ring.clone()),
        "baby_flame_impact" => Some(visuals.baby_flame_impact.clone()),
        "sharp_claws_slash" => Some(visuals.sharp_claws_slash.clone()),
        _ => None,
    }
}

/// World-space base point a resolved placement offset is applied relative to.
/// `caster_xy` is the caster's live body center; the mouth anchor derives the
/// muzzle from it using the sprite facing + scale.
fn anchor_base_xy(
    anchor: PlacementAnchor,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    flip_x: bool,
    scale: f32,
) -> [f32; 2] {
    match anchor {
        PlacementAnchor::Mouth => mouth_anchor_xy(caster_xy, flip_x, scale),
        PlacementAnchor::CasterCenter => caster_xy,
        PlacementAnchor::TargetCenter => target_xy,
    }
}

/// Spawn every particle of effect `effect_id` from the owned `VfxAsset`. Reads
/// `count`, `ttl_ticks`, `size_px`, `texture`, and the spawn-time color from the
/// resolved `EffectDef`; per-tick position/scale/color are then driven by
/// [`advance_vfx_particles`] through the Registry-resolved placement verb.
/// Returns the number of particles spawned (0 if the effect id is absent — the
/// caller logs; a load failure is surfaced once by `diagnose_agumon_vfx_load`).
#[allow(clippy::too_many_arguments)]
fn spawn_effect_by_id(
    commands: &mut Commands,
    asset: &VfxAsset,
    effect_id: &str,
    visuals: Option<&VfxVisuals>,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
    source_flip_x: bool,
    source_scale: f32,
) -> u32 {
    let Some(effect) = resolve_effect(asset, effect_id) else {
        return 0;
    };
    let count = effect.appearance.count.max(1);
    let base = anchor_base_xy(
        effect.placement.anchor,
        caster_xy,
        target_xy,
        source_flip_x,
        source_scale,
    );
    let rgba = eval_color(&effect.appearance.color, 0.0);
    let color = Color::linear_rgba(rgba[0], rgba[1], rgba[2], rgba[3]);
    let size = Vec2::splat(effect.appearance.size_px);
    for i in 0..count {
        let phase = (i as f32 / count as f32) * std::f32::consts::TAU;
        let sprite = match vfx_texture_handle(&effect.appearance.texture, visuals) {
            Some(image) => Sprite {
                image,
                custom_size: Some(size),
                color,
                ..default()
            },
            None => Sprite::from_color(color, size),
        };
        // Seed the spawn-frame rotation (age 0) so rotated effects don't pop from
        // axis-aligned on their first tick; per-tick rotation is then driven by
        // `advance_vfx_particles`. Static rotation yields 0.0 (no change).
        let spawn_ctx = PlacementCtx {
            age_ticks: 0,
            ttl_ticks: effect.appearance.ttl_ticks,
            progress: 0.0,
            phase,
            caster_xy,
            target_xy,
        };
        let spawn_angle = eval_rotation(&effect.appearance.rotation, &spawn_ctx);
        commands.spawn((
            sprite,
            Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z)
                .with_rotation(Quat::from_rotation_z(spawn_angle)),
            VfxParticle {
                ttl_ticks: effect.appearance.ttl_ticks,
                age_ticks: 0,
                motion: VfxMotion::Static,
                effect_id: EffectId(effect_id.to_owned()),
                anchor: effect.placement.anchor,
                phase,
            },
            VfxParticleTarget {
                world_xy: target_xy,
            },
            VfxParticleSource {
                unit_id: source_unit,
            },
        ));
    }
    count
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
    agumon_vfx: Option<Res<AgumonVfx>>,
    vfx_assets: Option<Res<Assets<VfxAsset>>>,
    sprites: Query<(&AgumonSprite, &Transform)>,
) {
    let Some(trigger) = latest_baby_burner_flash_trigger(events.read()) else {
        return;
    };

    let Some(asset) = agumon_vfx
        .as_ref()
        .zip(vfx_assets.as_ref())
        .and_then(|(vfx, assets)| assets.get(&vfx.handle))
    else {
        debug!(
            target: "windowed.agumon_playback",
            cast_id = ?trigger.cast_id,
            "vfx.ron not loaded; Baby Burner detonate effect skipped"
        );
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

    for target in trigger.targets {
        let Some(target_xy) = find_sprite_xy(&sprites, target) else {
            debug!(
                target: "windowed.agumon_playback",
                source_unit = ?trigger.source,
                target_unit = ?target,
                cast_id = ?trigger.cast_id,
                "Baby Burner detonate particle target could not be resolved"
            );
            continue;
        };

        let spawned = spawn_effect_by_id(
            &mut commands,
            asset,
            AGUMON_DETONATE_EFFECT_ID,
            vfx_visuals.as_deref(),
            caster_xy,
            target_xy,
            trigger.source,
            false,
            1.0,
        );
        trace!(
            target: "windowed.agumon_playback",
            cast_id = ?trigger.cast_id,
            effect_id = AGUMON_DETONATE_EFFECT_ID,
            spawned,
            source_unit = ?trigger.source,
            target_unit = ?target,
            "spawned Baby Burner detonate effect"
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn advance_vfx_particles(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    vfx_visuals: Option<Res<VfxVisuals>>,
    agumon_vfx: Option<Res<AgumonVfx>>,
    vfx_assets: Option<Res<Assets<VfxAsset>>>,
    regs: Option<Res<ExtRegistries>>,
    mut warned_effects: Local<HashSet<String>>,
    mut warned_verbs: Local<HashSet<String>>,
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
    let asset = agumon_vfx
        .as_ref()
        .zip(vfx_assets.as_ref())
        .and_then(|(vfx, assets)| assets.get(&vfx.handle));

    // Without the owned data path (asset not yet loaded / load failed) or the
    // placement Registry, no particle can be driven — there is no hardcoded
    // fallback path any more (T03). Age + despawn so none linger; the load
    // failure itself is surfaced once by `diagnose_agumon_vfx_load`.
    let (Some(asset), Some(regs)) = (asset, regs.as_deref()) else {
        for _ in 0..pending_ticks.0 {
            for (entity, mut particle, _, _, _, _) in &mut particles {
                particle.age_ticks += 1;
                particle.ttl_ticks = decrement_vfx_ttl(particle.ttl_ticks);
                if particle.ttl_ticks == 0 {
                    commands.entity(entity).despawn();
                }
            }
        }
        return;
    };

    for _ in 0..pending_ticks.0 {
        for (entity, mut particle, mut sprite, mut transform, target, source) in &mut particles {
            // Resolve the owned effect. A particle carrying an id absent from the
            // asset is warned once (naming the id) then despawned — no panic (Q7).
            let Some(effect) = resolve_effect(asset, &particle.effect_id.0) else {
                if warned_effects.insert(particle.effect_id.0.clone()) {
                    warn!(
                        target: "windowed.agumon_playback",
                        effect_id = %particle.effect_id.0,
                        reason = "effect id absent from loaded vfx.ron",
                        "skipping VFX particle; owned effect unresolved"
                    );
                }
                commands.entity(entity).despawn();
                continue;
            };

            // Resolve the placement verb from the Registry. An unregistered verb
            // id is warned once (naming effect + verb) then the particle is
            // despawned — no panic.
            let Some(verb) = regs.placements.get(&effect.placement.verb) else {
                if warned_verbs.insert(effect.placement.verb.clone()) {
                    warn!(
                        target: "windowed.agumon_playback",
                        effect_id = %particle.effect_id.0,
                        verb = %effect.placement.verb,
                        reason = "placement verb id not registered",
                        "skipping VFX particle; placement verb unresolved"
                    );
                }
                commands.entity(entity).despawn();
                continue;
            };

            let full_ttl = effect.appearance.ttl_ticks;
            let progress = if full_ttl == 0 {
                1.0
            } else {
                (particle.age_ticks as f32 / full_ttl as f32).clamp(0.0, 1.0)
            };

            // Anchor base resolves against the caster's *live* transform so
            // mouth/caster-anchored effects track the sprite as it moves.
            let live_source = source_sprites
                .iter()
                .find(|(agumon, _, _)| agumon.unit_id == source.unit_id)
                .map(|(_, sp, tf)| ([tf.translation.x, tf.translation.y], sp.flip_x, tf.scale.x));
            let (caster_xy, flip_x, scale) = live_source.unwrap_or((
                [transform.translation.x, transform.translation.y],
                false,
                1.0,
            ));
            let base = anchor_base_xy(particle.anchor, caster_xy, target.world_xy, flip_x, scale);

            let ctx = PlacementCtx {
                age_ticks: particle.age_ticks,
                ttl_ticks: full_ttl,
                progress,
                phase: particle.phase,
                caster_xy,
                target_xy: target.world_xy,
            };

            // FanOut shards drive their outward distance from the asset's scale
            // curve (S01 behavior) rather than the verb's own easing; the verb is
            // still resolved above so an unregistered id is caught. Every other
            // verb contributes its closed-form offset directly.
            let offset = if let PlacementParams::FanOut { spread_px } = effect.placement.params {
                let dist = spread_px * eval_scale(&effect.appearance.scale, progress);
                [dist * particle.phase.cos(), dist * particle.phase.sin()]
            } else {
                verb(&ctx, &effect.placement.params)
            };
            transform.translation.x = base[0] + offset[0];
            transform.translation.y = base[1] + offset[1];
            transform.translation.z = VFX_PARTICLE_Z;

            // FanOut keeps unit scale (sprite size from size_px, travel from the
            // curve); every other effect drives sprite scale from the curve.
            transform.scale = if matches!(effect.placement.params, PlacementParams::FanOut { .. }) {
                Vec3::ONE
            } else {
                Vec3::splat(eval_scale(&effect.appearance.scale, progress))
            };
            // Per-particle quad rotation (S06): a single asymmetric sprite (flame
            // tongue) fans into fire via decorrelated `phase`-driven angles. Static
            // rotation evaluates to 0.0, preserving the prior axis-aligned billboard.
            transform.rotation =
                Quat::from_rotation_z(eval_rotation(&effect.appearance.rotation, &ctx));
            let rgba = eval_color(&effect.appearance.color, progress);
            sprite.color = Color::linear_rgba(rgba[0], rgba[1], rgba[2], rgba[3]);

            let on_expire = effect.on_expire.clone();

            particle.age_ticks += 1;
            particle.ttl_ticks = decrement_vfx_ttl(particle.ttl_ticks);
            if particle.ttl_ticks == 0 {
                // Data-driven chain: spawn the on_expire effect at the current
                // position (replaces the hardcoded projectile->impact burst).
                if let Some(next) = on_expire {
                    let pos = [transform.translation.x, transform.translation.y];
                    spawn_effect_by_id(
                        &mut commands,
                        asset,
                        &next.0,
                        vfx_visuals.as_deref(),
                        pos,
                        pos,
                        source.unit_id,
                        flip_x,
                        scale,
                    );
                }
                trace!(
                    target: "windowed.agumon_playback",
                    entity = ?entity,
                    motion = ?particle.motion,
                    effect_id = %particle.effect_id.0,
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
    fn is_death_reaction_only_matches_unit_died() {
        use bevyrogue::combat::events::CombatEventKind;
        // A KO event enters the death pipeline...
        assert!(is_death_reaction(&CombatEventKind::UnitDied {
            status_remaining: vec![],
            heated_remaining: 0,
        }));
        // ...while a non-lethal hit (the hurt path) never does (Q7 negative test).
        assert!(!is_death_reaction(&CombatEventKind::OnHitTaken { amount: 5 }));
    }

    #[test]
    fn decrement_vfx_ttl_saturates_at_zero() {
        let mut ttl = 6u32;
        for _ in 0..6 {
            ttl = decrement_vfx_ttl(ttl);
        }
        assert_eq!(ttl, 0);
        assert_eq!(decrement_vfx_ttl(ttl), 0);
    }

    #[test]
    fn fade_alpha_lerps_full_to_zero() {
        // Full remaining ticks = fully opaque.
        assert_eq!(fade_alpha(8, 8), 1.0);
        // Half spent = ~half alpha.
        assert!((fade_alpha(4, 8) - 0.5).abs() < f32::EPSILON);
        // Spent = fully transparent.
        assert_eq!(fade_alpha(0, 8), 0.0);
        // total_ticks == 0 saturates to 1.0 without dividing by zero (Q5).
        assert_eq!(fade_alpha(0, 0), 0.0);
        assert_eq!(fade_alpha(5, 0), 1.0);
    }

    #[test]
    fn on_enter_charge_seeds_both_the_orb_and_the_ember_swirl() {
        // The single authored `baby_flame_charge` SpawnParticle fans out to the
        // owned charge + ember effect ids; the projectile maps to its own id.
        assert_eq!(
            on_enter_effect_ids("baby_flame_charge"),
            &[AGUMON_CHARGE_EFFECT_ID, AGUMON_EMBER_EFFECT_ID]
        );
        assert_eq!(
            on_enter_effect_ids("baby_flame_projectile"),
            &[AGUMON_PROJECTILE_EFFECT_ID]
        );
        // An unknown particle name maps to no effects (spawns nothing, no panic).
        assert!(on_enter_effect_ids("unknown_particle").is_empty());
    }

    #[test]
    fn on_enter_sharp_claws_maps_only_to_the_slash_effect() {
        // The `sharp_claws_slash` SpawnParticle maps to exactly the owned slash
        // effect id — proving the data-driven bridge, not a VFX-kind branch.
        assert_eq!(
            on_enter_effect_ids("sharp_claws_slash"),
            &[AGUMON_SHARP_CLAWS_EFFECT_ID]
        );
        assert_eq!(AGUMON_SHARP_CLAWS_EFFECT_ID, "sharp_claws.slash");

        // Unrelated / near-miss names must NOT resolve to the Sharp Claws effect:
        // the bridge is an exact name map, not a substring/string-kind match.
        for name in ["sharp_claws", "slash", "baby_flame_charge", "sharp_claws_strike", ""] {
            assert!(
                !on_enter_effect_ids(name).contains(&AGUMON_SHARP_CLAWS_EFFECT_ID),
                "`{name}` must not map to the Sharp Claws effect id"
            );
        }
    }

    #[test]
    fn anchor_base_resolves_each_anchor_against_the_right_origin() {
        let caster = [10.0, 20.0];
        let target = [80.0, -4.0];
        assert_eq!(
            anchor_base_xy(PlacementAnchor::CasterCenter, caster, target, false, 1.0),
            caster
        );
        assert_eq!(
            anchor_base_xy(PlacementAnchor::TargetCenter, caster, target, false, 1.0),
            target
        );
        // Mouth derives the muzzle from the caster center + facing/scale.
        assert_eq!(
            anchor_base_xy(PlacementAnchor::Mouth, caster, target, false, 1.0),
            mouth_anchor_xy(caster, false, 1.0)
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
