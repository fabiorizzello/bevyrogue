use std::collections::{HashMap, HashSet};

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    ecs::system::SystemParam,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_enoki::prelude::{OneShot, SpriteParticle2dMaterial};
use bevy_enoki::{EnokiPlugin, ParticleEffectHandle, ParticleSpawner};

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles, AnimationClipLoadState,
    AnimationGraphLookupDiagnostics, AtlasGeometry, Clip, FrameCueCommand, NodeId, PlacementAnchor,
    ResolvedAnimGraph, ResolvedAnimGraphSource, SkillGraphRegistry, StanceGraphRegistry,
    StanceReaction, VfxSpawnDescriptor, stance_reaction_for,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;
use bevyrogue::ui::combat_panel::latest_baby_burner_flash_trigger;
use bevyrogue::ui::cues::{CueDef, CueRegistry, flash_tint_parametric, shake_offset_parametric};
use bevyrogue::ui::hit_feedback::{
    FLASH_TICKS, HitFlashState, HitShakeState, SHAKE_TICKS, damage_number_kinematics,
    hit_damage_amount, observe_hit_feedback,
};

pub(in crate::windowed) mod registries;

/// Marker + FSM state for an on-screen Digimon preview actor. Carries the
/// stance/skill animation-graph ids as DATA (`stance_graph_id` / `skill_graph_id`)
/// rather than reading module-level species-specific consts, so adding a new
/// Digimon (S04/S05) means spawning this component with different ids — no edits here.
#[derive(Component, Debug, Clone)]
pub(super) struct DigimonSprite {
    pub(super) unit_id: UnitId,
    presentation_id: String,
    pub(super) player: AnimGraphPlayer,
    graph: ResolvedAnimGraph,
    mode: DigimonPlaybackMode,
    last_release_frame: Option<ReleaseFrameKey>,
    last_missing_skill_graph_cue: Option<String>,
    /// `AnimGraphId` of this Digimon's stance graph (idle/hurt/death). The data
    /// source for every stance `resolve_snapshot` this sprite drives; preserved
    /// across mode transitions so idle-restore / hurt / death always resolve the
    /// correct graph without consulting a const.
    stance_graph_id: String,
    /// `AnimGraphId` of this Digimon's skill graph. The data source for the skill
    /// `resolve_snapshot` in `sync_digimon_mode`.
    skill_graph_id: String,
}

/// Tags a persistent enoki emitter spawned for the Baby Flame charge orb / ember
/// swirl (M006/S01 T03, D046). Unlike the contact bursts (which carry
/// `OneShot::Despawn` and drain on their own), these emitters must keep emitting at
/// the caster's mouth through the entire charge buildup and then be cleared the
/// instant the flame launches. `advance_digimon_presentation` despawns every marker
/// whose `unit_id` matches the casting sprite at the `CueReleaseResult::Released`
/// boundary, reproducing the old quad-path launch-clear without entering the
/// kernel/FSM timeline (D031/D032). Carries the source `UnitId` so the launch-clear
/// can target only the caster's emitters.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct ChargeEmberEnokiMarker {
    unit_id: UnitId,
}

/// Drives a persistent enoki projectile emitter caster->target then chains the
/// impact burst on arrival (M006/S01 T03, D046). The Baby Flame projectile is a
/// traveling emitter, not a self-despawning one-shot: `advance_enoki_projectiles`
/// lerps the entity's `Transform` from `from_xy` to `to_xy` over `ticks_total`
/// animation ticks and, on arrival (`ticks_elapsed >= ticks_total`), despawns the
/// spawner and spawns `baby_flame.impact` at `to_xy` — reproducing the old
/// quad-path `on_expire` projectile->impact chain. Presentation-only, fire-and-
/// forget; never feeds the kernel/FSM timeline (D031/D032).
#[derive(Component, Debug, Clone, PartialEq)]
struct ProjectileFlight {
    from_xy: [f32; 2],
    to_xy: [f32; 2],
    ticks_total: u32,
    ticks_elapsed: u32,
    /// Effect id spawned at `to_xy` on arrival (data, not a const) — the engine
    /// chains whatever the registry entry named, so a different Digimon's
    /// projectile can chain its own impact without editing this system.
    on_arrival: String,
}

/// Terminal marker: this sprite has been seeded into its `death` stance node and
/// is exiting the field. Kept as a *separate* component rather than a new
/// `DigimonPlaybackMode` variant so the `mode` match arms in `sync_digimon_mode` /
/// `classify_same_skill_sync` stay closed (S02-RESEARCH). Its presence tells
/// `advance_digimon_presentation` to (a) skip mode reconciliation so a still-active
/// barrier cannot re-`start_skill` the dying sprite, and (b) leave the sprite on
/// its final death frame instead of idle-restoring it on node exit. The fade-out
/// driver (T02) consumes this marker to lerp the sprite out and despawn it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct DeathExiting;

/// Off-field exit fader. Seeded by `advance_digimon_presentation` on the frame the
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

/// Binary-local rest position of an `DigimonSprite`, captured at spawn. The hit
/// shake offset (S03/T02) is applied *relative to this* so the sprite always
/// restores to its exact spawn `(x, 0.0)` without accumulating drift — research
/// warns that hardcoding the ±200 layout value goes stale, so capture it once at
/// spawn instead.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct SpriteRest {
    xy: Vec2,
}

/// Binary-local rest translation of the `Camera2d`, captured at spawn. Camera
/// shake (S03/T03) is applied *relative to this* — the apply system hard-sets
/// `translation = rest.translation + offset` every tick and snaps back to
/// `rest.translation` at remaining 0, so the camera never accumulates drift
/// (MEM094 — absolute offset from rest, never additive). Same discipline as
/// [`SpriteRest`].
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct CameraRest {
    translation: Vec3,
}

/// Binary-local camera-shake countdown, armed by `OnHitTaken` to the
/// `camera_impact` cue's tick window. Single `remaining` (mirrors the shape of
/// [`HitShakeState`] but global — there is one camera). Pure presentation
/// overlay; never touches `CombatState` (R010).
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct CameraShakeState {
    remaining: u32,
}

impl CameraShakeState {
    /// Arm (or re-arm) the camera shake to a full `ticks` window. Idempotent:
    /// multiple hits in the same window collapse to a single full countdown.
    fn arm(&mut self, ticks: u32) {
        self.remaining = ticks;
    }

    /// Drain by `n` ticks (saturating, never underflows).
    fn decay_by(&mut self, n: u32) {
        self.remaining = self.remaining.saturating_sub(n);
    }
}

/// Binary-local floating damage number (S03/T03). A short-lived world-space
/// `Text2d` spawned over a struck target on each `OnHitTaken`, driven by the pure
/// [`damage_number_kinematics`] projection: it rises and fades over `total_ticks`
/// then despawns. `base_y` is the spawn Y captured once so each tick can hard-set
/// `translation.y = base_y + rise_px` absolutely (no drift accumulation, mirroring
/// the hit-shake offset discipline). One number is spawned per hit — multi-hit
/// shows multiple, never deduped. Pure overlay; never touches `CombatState` (R010).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct CanvasDamageNumber {
    age_ticks: u32,
    total_ticks: u32,
    base_y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DigimonPlaybackMode {
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

impl DigimonSprite {
    pub(super) fn idle_for(
        unit_id: UnitId,
        presentation_id: String,
        graph: ResolvedAnimGraph,
        stance_graph_id: String,
        skill_graph_id: String,
    ) -> Self {
        let entry = graph.graph().entry.clone();
        Self {
            unit_id,
            presentation_id,
            player: AnimGraphPlayer::new(entry),
            graph,
            mode: DigimonPlaybackMode::Idle,
            last_release_frame: None,
            last_missing_skill_graph_cue: None,
            stance_graph_id,
            skill_graph_id,
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
        self.mode = DigimonPlaybackMode::Skill {
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
        self.mode = DigimonPlaybackMode::Idle;
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

/// Bound atlas handles + geometry for one registered sprite presentation.
/// Stored in [`PresentationAtlasRegistry`] under the presentation's owned id, so
/// the engine can host multiple Digimon atlases without species branches.
#[derive(Debug, Clone)]
pub(super) struct PresentationAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    geometry: AtlasGeometry,
}

/// In-memory atlas bindings keyed by presentation id. Built lazily the first
/// time a registered presentation's clip metadata becomes readable.
#[derive(Resource, Debug, Clone, Default)]
pub(super) struct PresentationAtlasRegistry {
    atlases: HashMap<String, PresentationAtlas>,
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
/// Horizontal rest position of a team's column: allies left, enemies right.
const TEAM_COLUMN_X: f32 = 200.0;
/// Vertical gap between adjacent same-team slots. Sized so two ~205px sprites in
/// the same column read as distinct actors rather than one stacked blob.
const SLOT_VERTICAL_SPACING: f32 = 150.0;
/// Per-slot z increment so same-team sprites never share an exact z (stable draw
/// order, no flicker). Kept well below `VFX_PARTICLE_Z` so sprites stay behind VFX.
const SLOT_Z_STEP: f32 = 0.01;
const VFX_PARTICLE_Z: f32 = 1.0;
/// Z of a floating damage number. Above `VFX_PARTICLE_Z` so the number reads on
/// top of any impact particles spawned at the same target.
const DAMAGE_NUMBER_Z: f32 = 2.0;
/// Pixels above the target's body center a damage number spawns, so it floats
/// over (not through) the struck sprite.
const DAMAGE_NUMBER_SPAWN_OFFSET_Y_PX: f32 = 40.0;
/// Lifetime of a floating damage number in animation ticks (~1s at 12fps).
const DAMAGE_NUMBER_TICKS: u32 = 12;
/// On-canvas font size of a damage number.
const DAMAGE_NUMBER_FONT_SIZE: f32 = 32.0;
const VFX_MOUTH_OFFSET_X_PX: f32 = 92.0;
const VFX_MOUTH_OFFSET_Y_PX: f32 = 24.0;

/// Fixed-rate animation clock for the windowed presentation layer.
///
/// `advance_digimon_presentation` previously advanced the player once per render
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
                    target: "windowed.digimon_playback",
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
        // bevy_enoki's GPU 2D particle backend (M005/S04) is now the sole particle
        // VFX renderer (M006/S01, D043). EnokiPlugin brings its own
        // `Particle2dEffect` asset loader, so the `.particle.ron` needs no
        // RonAssetPlugin registration; the loader is selected by asset type.
        // Windowed-gated like everything in this module so no render-stack dep
        // leaks into the headless build (R005/R016).
        app.add_plugins(EnokiPlugin)
            .insert_resource(AnimationClock::from_env())
            .insert_resource(PendingAnimationTicks::default())
            .init_resource::<HitFlashState>()
            .init_resource::<HitShakeState>()
            .init_resource::<CameraShakeState>()
            // Engine-generic effect registries; the per-Digimon module populates
            // them via its `register` Startup systems (S04).
            .init_resource::<EnokiVfxRegistry>()
            .init_resource::<OnEnterEffectRegistry>()
            .init_resource::<SkillReleaseEffectRegistry>()
            .init_resource::<DetonateEffectRegistry>()
            .init_resource::<SkillStartNodeRegistry>()
            .init_resource::<SpritePresentationRegistry>()
            .init_resource::<PresentationAtlasRegistry>()
            .add_systems(Startup, setup_camera)
            .add_systems(Startup, init_soft_particle_material)
            .add_systems(Update, diagnose_enoki_vfx_load)
            .add_systems(Update, build_digimon_atlases.before(spawn_unit_sprites))
            .add_systems(Update, spawn_unit_sprites)
            .add_systems(
                Update,
                sample_animation_ticks.before(advance_enoki_projectiles),
            )
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
                    .before(advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Arm the transient flash/shake windows off the CombatEvent bus.
                // Mirrors drive_hurt_reactions' ordering so the windows are armed
                // before advance_digimon_presentation decays + applies them.
                observe_hit_feedback
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Arm the camera-shake window off the SAME OnHitTaken signal that
                // arms HitShakeState (camera-shake is just another registered cue).
                // Ordered before advance_digimon_presentation, which owns the single
                // decay site.
                observe_camera_shake
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Spawn a world-space Text2d damage number over each struck
                // target. Ordered like drive_hurt_reactions so it reads the same
                // CombatEvent window before the presentation chain advances.
                spawn_canvas_damage_numbers
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Float/fade/despawn the damage numbers on the same animation
                // clock as advance_vfx_particles (disjoint component set).
                advance_canvas_damage_numbers.after(sample_animation_ticks),
            )
            .add_systems(
                Update,
                // AFTER the hurt driver enforces death-precedence: a target both
                // struck and killed in one window resolves to `death`, not `hurt`.
                drive_death_reactions
                    .after(drive_hurt_reactions)
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                (
                    advance_enoki_projectiles,
                    advance_digimon_presentation,
                    advance_death_fade,
                )
                    .chain()
                    .after(sample_animation_ticks)
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Write the Camera2d transform from the decayed CameraShakeState.
                // Ordered AFTER advance_digimon_presentation (the single decay site)
                // so it reads the freshly-drained remaining and applies an absolute
                // offset from CameraRest — never additive (MEM094).
                apply_camera_shake.after(advance_digimon_presentation),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    let transform = Transform::default();
    commands.spawn((
        Camera2d,
        Hdr,
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        transform,
        // Capture the camera's spawn translation so camera-shake restores to the
        // exact rest without drift (never accumulate — MEM094). Same anti-drift
        // pattern as SpriteRest.
        CameraRest {
            translation: transform.translation,
        },
    ));
}

/// Arm the camera-shake window on every `OnHitTaken` — the SAME signal that arms
/// `HitShakeState` — sizing it to the `camera_impact` cue's tick count from the
/// `CueRegistry`. Owns its own message cursor (MEM065); same-window multi-hit
/// collapses via the reset in `arm`. Emits a `trace!` on the
/// `windowed.digimon_playback` target (mirroring the `flash+shake armed` seam) so
/// a future agent can confirm the cue fired without running the binary (K001).
fn observe_camera_shake(
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    mut camera_shake: ResMut<CameraShakeState>,
    cue_registry: Res<CueRegistry>,
) {
    for event in events.read() {
        if hit_damage_amount(&event.kind).is_some() {
            if let Some(CueDef::CameraShake { ticks, .. }) = cue_registry.get("camera_impact") {
                camera_shake.arm(*ticks);
                trace!(
                    target: "windowed.digimon_playback",
                    target_unit = ?event.target,
                    camera_shake_ticks = *ticks,
                    "camera-shake armed"
                );
            }
        }
    }
}

/// Write the `Camera2d` transform from the decayed [`CameraShakeState`]. While
/// `remaining > 0` the translation is the ABSOLUTE offset from the captured
/// [`CameraRest`] — `rest.translation + shake_offset_parametric(..)` — and at
/// remaining 0 it is hard-set back to `rest.translation`, so the camera never
/// accumulates drift (MEM094). Reads the `camera_impact` `CameraShake` params
/// from the registry; falls through to no offset on a missing/wrong def.
fn apply_camera_shake(
    camera_shake: Res<CameraShakeState>,
    cue_registry: Res<CueRegistry>,
    mut cameras: Query<(&mut Transform, &CameraRest)>,
) {
    for (mut transform, rest) in &mut cameras {
        if camera_shake.remaining > 0 {
            let offset = match cue_registry.get("camera_impact") {
                Some(CueDef::CameraShake {
                    amp,
                    freq_x,
                    freq_y,
                    ticks,
                }) => {
                    shake_offset_parametric(camera_shake.remaining, *ticks, *amp, *freq_x, *freq_y)
                }
                _ => Vec2::ZERO,
            };
            transform.translation = rest.translation + offset.extend(0.0);
        } else {
            transform.translation = rest.translation;
        }
    }
}

// Engine-generic presentation registries and shared types now live in
// `render/registries.rs` (M006/S09). The per-Digimon modules import them
// directly from `crate::windowed::render::registries`; render.rs's own
// presentation systems pull the ones they use into scope here.
use registries::{
    DetonateEffectRegistry, EnokiLifecycle, EnokiVfxRegistry, OnEnterEffectRegistry,
    SkillReleaseEffectRegistry, SkillStartNodeRegistry, SoftParticleMaterial,
    SpritePresentationEntry, SpritePresentationRegistry,
};

fn presentation_entry_for_unit(
    presentation: &SpritePresentationRegistry,
    unit_id: UnitId,
) -> Option<&SpritePresentationEntry> {
    presentation
        .entries
        .iter()
        .find(|entry| entry.matches_unit(unit_id))
}

/// Bundles the spawn-side effect registries plus the skill start-node registry
/// into a single `SystemParam` so `advance_digimon_presentation` stays within
/// Bevy's 16-parameter system limit.
#[derive(SystemParam)]
struct EffectRegistries<'w> {
    enoki: Option<Res<'w, EnokiVfxRegistry>>,
    on_enter: Res<'w, OnEnterEffectRegistry>,
    skill_release: Res<'w, SkillReleaseEffectRegistry>,
    skill_start_node: Res<'w, SkillStartNodeRegistry>,
    soft_material: Option<Res<'w, SoftParticleMaterial>>,
}

/// Build the shared soft-particle [`SpriteParticle2dMaterial`] at Startup: load the
/// radial-gradient PNG, register the material asset, and store its handle in
/// [`SoftParticleMaterial`]. Spawning every effect through this material (instead of
/// enoki's flat-square `ColorParticle2dMaterial` default) is the single
/// highest-leverage VFX fix — see [`SoftParticleMaterial`] and the `bevy-enoki-vfx`
/// skill. The texture is generated deterministically by `scripts/gen_soft_particle.py`.
fn init_soft_particle_material(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut commands: Commands,
) {
    let texture = asset_server.load("vfx/soft_particle.png");
    let handle = materials.add(SpriteParticle2dMaterial::from_texture(texture));
    commands.insert_resource(SoftParticleMaterial(handle));
    info!(
        target: "windowed.digimon_playback",
        texture = "vfx/soft_particle.png",
        "soft-particle sprite material built; enoki effects spawn as soft blobs (not flat squares)"
    );
}

/// Surface a load failure for each registered enoki `.particle.ron` once. A
/// failed/missing effect asset means that effect silently spawns nothing through
/// the enoki backend; this makes a dead burst visible by id+path rather than
/// silently absent (slice failure-visibility). Reads the source path from the
/// registry entry (S04) rather than a const match. Each id is warned at most once
/// via the warned-set.
fn diagnose_enoki_vfx_load(
    enoki_vfx: Option<Res<EnokiVfxRegistry>>,
    asset_server: Res<AssetServer>,
    mut warned: Local<HashSet<String>>,
) {
    let Some(enoki_vfx) = enoki_vfx else {
        return;
    };
    for (effect_id, entry) in &enoki_vfx.handles {
        if warned.contains(effect_id) {
            continue;
        }
        if matches!(
            asset_server.load_state(entry.handle.id()),
            bevy::asset::LoadState::Failed(_)
        ) {
            warn!(
                target: "windowed.digimon_playback",
                path = entry.path.as_str(),
                effect = effect_id.as_str(),
                reason = "enoki .particle.ron failed to load or parse",
                "enoki contact-burst VFX disabled for this effect id; no enoki particles will spawn"
            );
            warned.insert(effect_id.clone());
        }
    }
}

fn sample_animation_ticks(
    time: Res<Time>,
    mut clock: ResMut<AnimationClock>,
    mut pending_ticks: ResMut<PendingAnimationTicks>,
) {
    pending_ticks.0 = clock.tick(time.delta_secs());
}

/// Builds the shared `PresentationAtlas` (image + `TextureAtlasLayout` + geometry)
/// once the agumon `Clip` is readable. Idempotent: returns early once the
/// resource exists, so it runs at most one effective build. Emits a one-time
/// `info!` describing the grid and a one-time `warn!` if the clip never becomes
/// readable or the atlas image fails to load.
#[allow(clippy::too_many_arguments)]
fn build_digimon_atlases(
    mut atlases: ResMut<PresentationAtlasRegistry>,
    clip_load_state: Res<AnimationClipLoadState>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    clips: Res<Assets<Clip>>,
    asset_server: Res<AssetServer>,
    presentation: Res<SpritePresentationRegistry>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut warned: Local<HashSet<String>>,
) {
    let Some(handles) = clip_handles else {
        return;
    };

    for entry in &presentation.entries {
        let image_warn_key = format!("image:{}", entry.presentation_id);
        if let Some(bound) = atlases.atlases.get(&entry.presentation_id) {
            if matches!(
                asset_server.load_state(bound.image.id()),
                bevy::asset::LoadState::Failed(_)
            ) && warned.insert(image_warn_key)
            {
                warn!(
                    target: "windowed.digimon_playback",
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    "digimon atlas image load failed — sprites will render blank"
                );
            }
            continue;
        }

        let clip = handles
            .0
            .get(entry.clip_index)
            .and_then(|handle| clips.get(handle));
        let Some(clip) = clip else {
            let clip_warn_key = format!("clip:{}", entry.presentation_id);
            if clip_load_state.ready && warned.insert(clip_warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    clip_index = entry.clip_index,
                    "presentation clip not readable after load state ready; atlas binding deferred — sprites stay blank"
                );
            }
            continue;
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
        let image = asset_server.load(entry.atlas_image_path.clone());

        if matches!(
            asset_server.load_state(image.id()),
            bevy::asset::LoadState::Failed(_)
        ) && warned.insert(image_warn_key)
        {
            warn!(
                target: "windowed.digimon_playback",
                presentation_id = entry.presentation_id.as_str(),
                path = entry.atlas_image_path.as_str(),
                "digimon atlas image load failed — sprites will render blank"
            );
        }

        info!(
            target: "windowed.digimon_playback",
            presentation_id = entry.presentation_id.as_str(),
            path = entry.atlas_image_path.as_str(),
            frame_w = geometry.frame_size.w,
            frame_h = geometry.frame_size.h,
            columns = geometry.columns,
            rows = geometry.rows,
            total_frames = geometry.total_frames,
            "presentation atlas built (TextureAtlasLayout + image bound)"
        );

        atlases.atlases.insert(
            entry.presentation_id.clone(),
            PresentationAtlas {
                image,
                layout,
                geometry,
            },
        );
    }
}

/// Spawns one `DigimonSprite` entity per unit that does not yet have one.
/// Runs every frame but is idempotent: once a sprite exists for a unit it is skipped.
/// Waits for the stance graph to be loaded before spawning anything.
fn spawn_unit_sprites(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    atlases: Res<PresentationAtlasRegistry>,
    presentation: Res<SpritePresentationRegistry>,
    units: Query<(&Unit, &Team)>,
    sprites: Query<&DigimonSprite>,
    mut warned: Local<HashSet<String>>,
) {
    let spawned: HashSet<UnitId> = sprites.iter().map(|s| s.unit_id).collect();

    // Deterministic per-team slot assignment computed across ALL units (not just
    // the ones spawned this frame), so a multi-actor team fans out to distinct
    // positions instead of every member stacking at one (x, z). Slot is the
    // unit's index within its team after sorting by UnitId — stable regardless of
    // spawn order or which units already exist. Replaces the previous team-only
    // ±200 placement that collapsed two allies onto the same point.
    let slot_of: HashMap<UnitId, (usize, usize)> = {
        let mut roster: Vec<(UnitId, Team)> = units.iter().map(|(u, t)| (u.id, *t)).collect();
        roster.sort_by_key(|(id, _)| id.0);
        let mut map = HashMap::new();
        for assigned in [Team::Ally, Team::Enemy] {
            let members: Vec<UnitId> = roster
                .iter()
                .filter(|(_, t)| *t == assigned)
                .map(|(id, _)| *id)
                .collect();
            let count = members.len();
            for (slot, id) in members.into_iter().enumerate() {
                map.insert(id, (slot, count));
            }
        }
        map
    };

    for (unit, team) in &units {
        if spawned.contains(&unit.id) {
            continue;
        }

        let Some(entry) = presentation_entry_for_unit(&presentation, unit.id) else {
            let warn_key = format!("missing-presentation:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    "no sprite presentation registered for unit; sprite spawn deferred"
                );
            }
            continue;
        };

        let Some(stance_graph) = stance_reg
            .resolve_snapshot(&AnimGraphId(entry.stance_graph_id.clone().into()), &graphs)
        else {
            let warn_key = format!("missing-stance:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    presentation_id = entry.presentation_id.as_str(),
                    graph_id = entry.stance_graph_id.as_str(),
                    "stance graph not yet readable; sprite spawn deferred"
                );
            }
            continue;
        };

        let Some(atlas) = atlases.atlases.get(&entry.presentation_id) else {
            let warn_key = format!("missing-atlas:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    "presentation atlas binding unavailable; sprite spawn deferred"
                );
            }
            continue;
        };

        let flip_x = *team == Team::Enemy;
        let x = if flip_x { TEAM_COLUMN_X } else { -TEAM_COLUMN_X };
        let (slot, count) = slot_of.get(&unit.id).copied().unwrap_or((0, 1));
        let y = slot_offset_y(slot, count);
        let z = slot as f32 * SLOT_Z_STEP;
        commands.spawn((
            DigimonSprite::idle_for(
                unit.id,
                entry.presentation_id.clone(),
                stance_graph.clone(),
                entry.stance_graph_id.clone(),
                entry.skill_graph_id.clone(),
            ),
            Sprite {
                image: atlas.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: 0,
                }),
                flip_x,
                ..default()
            },
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(SPRITE_DISPLAY_SCALE)),
            SpriteRest {
                xy: Vec2::new(x, y),
            },
        ));
    }
}

fn advance_digimon_presentation(
    pending_ticks: Res<PendingAnimationTicks>,
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut lookup_diagnostics: ResMut<AnimationGraphLookupDiagnostics>,
    mut barrier: ResMut<SuspendedTimelineState>,
    atlases: Res<PresentationAtlasRegistry>,
    effects: EffectRegistries,
    charge_ember_markers: Query<(Entity, &ChargeEmberEnokiMarker)>,
    mut hit_flash: ResMut<HitFlashState>,
    mut hit_shake: ResMut<HitShakeState>,
    mut camera_shake: ResMut<CameraShakeState>,
    cue_registry: Res<CueRegistry>,
    // Dedup set for cast-cue spawn misses: a `SpawnParticle` cue that resolves to
    // zero spawned particles is warned at most once per cue id (S08, reusing the
    // S06 `Local<HashSet>` warn-once pattern) instead of silently producing nothing.
    mut cast_cue_spawn_miss_warned: Local<HashSet<String>>,
    mut sprites: ParamSet<(
        Query<(
            Entity,
            &mut DigimonSprite,
            &mut Sprite,
            &mut Transform,
            &SpriteRest,
            Option<&DeathExiting>,
            Option<&FadeOut>,
        )>,
        Query<(&DigimonSprite, &Transform)>,
        // Read-only team lookup (p2 of the ParamSet to stay under Bevy's 16-param
        // system limit). Unit/Team live on combat entities, disjoint from sprite
        // entities; used to aim skill VFX at the opposing team (S08 multi-ally fix).
        Query<(&Unit, &Team)>,
    )>,
) {
    {
        let active_barrier = barrier.active_status().cloned();
        if let Some(status) = active_barrier.as_ref() {
            // Auto-release is now only the fallback for genuinely unbridged skills
            // (no windowed presentation graph). Bridged skills — sharp_claws,
            // baby_flame, agumon_ult — release on their rendered ReleaseKernel cue
            // in the per-tick block below instead of being auto-released here.
            if status.awaiting_release
                && should_auto_release_unbridged(&effects.skill_start_node, &status.skill_id.0)
            {
                debug!(
                    target: "windowed.digimon_playback",
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

    // Decay the transient hit-feedback windows once per frame on the same
    // PendingAnimationTicks clock (single decay source of truth, R010 — pure
    // overlay, never touches CombatState). A unit still sitting at the full
    // window was freshly armed by observe_hit_feedback this frame; trace it once
    // (mirrors drive_hurt_reactions' trace seam) before the decay drains it.
    if pending_ticks.0 > 0 {
        for unit_id in hit_flash.remaining.keys().copied().collect::<Vec<_>>() {
            if hit_flash.remaining(unit_id) == FLASH_TICKS {
                trace!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit_id,
                    flash_ticks = FLASH_TICKS,
                    shake_ticks = SHAKE_TICKS,
                    "flash+shake armed"
                );
            }
        }
        hit_flash.decay_by(pending_ticks.0);
        hit_shake.decay_by(pending_ticks.0);
        // Camera shake decays on the SAME single source-of-truth clock (MEM094 —
        // no second decay site); apply_camera_shake reads the drained remaining.
        camera_shake.decay_by(pending_ticks.0);
    }

    // Advance the player at the fixed animation rate, not once per render frame.
    // Most 60fps frames yield 0 ticks; the kernel-barrier release still observes
    // the rendered impact frame — it just samples it on the animation tick.
    // UnitId -> Team for the live roster, resolved once per frame. Sprites carry
    // no Team component (adding one would make every bare `Query<&Team>` combat
    // system also match sprite entities), so VFX targeting joins back to the
    // combat entities through this map.
    let team_of: HashMap<UnitId, Team> = sprites
        .p2()
        .iter()
        .map(|(unit, team)| (unit.id, *team))
        .collect();

    for _ in 0..pending_ticks.0 {
        let active_barrier = barrier.active_status().cloned();
        let sprite_positions: Vec<(UnitId, Team, [f32; 2])> = sprites
            .p1()
            .iter()
            .filter_map(|(sprite, transform)| {
                team_of.get(&sprite.unit_id).map(|team| {
                    (
                        sprite.unit_id,
                        *team,
                        [transform.translation.x, transform.translation.y],
                    )
                })
            })
            .collect();
        for (entity, mut sprite, mut render_sprite, mut transform, rest, death_exiting, fade_out) in
            &mut sprites.p0()
        {
            let prev_node = sprite.player.current_node.0.clone();

            // A dying sprite is resting on (or playing out) its death node. Skip
            // mode reconciliation entirely: a still-active kernel barrier must not
            // re-`start_skill` the dying caster back into its interrupted skill.
            if death_exiting.is_none() {
                sync_digimon_mode(
                    &mut sprite,
                    active_barrier.as_ref(),
                    &skill_reg,
                    &stance_reg,
                    &graphs,
                    &effects.skill_start_node,
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
            let atlas_index = atlases
                .atlases
                .get(&sprite.presentation_id)
                .and_then(|atlas| atlas.geometry.atlas_index(advance.frame));
            if let (Some(index), Some(texture_atlas)) =
                (atlas_index, render_sprite.texture_atlas.as_mut())
            {
                texture_atlas.index = index as usize;
            }

            // Transient hit feedback (S03/T02): flash tint + positional shake,
            // now sourced from the CueRegistry parametric math instead of the
            // hit_feedback consts (D048 model a — behaviour-preserving). Flash is
            // the SOLE colour writer for DigimonSprite (the parametric tint is
            // WHITE at remaining 0, so steady state stays white) — but skip the
            // write while the death fade owns the colour, so it never fights
            // advance_death_fade's alpha lerp (D031/D032 barrier untouched).
            if death_exiting.is_none() && fade_out.is_none() {
                if let Some(CueDef::Flash { peak, ticks }) = cue_registry.get("hit_flash") {
                    let (r, g, b) =
                        flash_tint_parametric(hit_flash.remaining(sprite.unit_id), *ticks, *peak);
                    render_sprite.color = Color::srgb(r, g, b);
                }
            }
            // Shake is an absolute offset from the captured rest position, never
            // accumulated: at remaining 0 the translation is hard-set back to rest.
            let z = transform.translation.z;
            let shake_remaining = hit_shake.remaining(sprite.unit_id);
            transform.translation = if shake_remaining > 0 {
                let offset = match cue_registry.get("hit_shake") {
                    Some(CueDef::SpriteShake {
                        amp,
                        freq_x,
                        freq_y,
                        ticks,
                    }) => shake_offset_parametric(shake_remaining, *ticks, *amp, *freq_x, *freq_y),
                    _ => Vec2::ZERO,
                };
                (rest.xy + offset).extend(z)
            } else {
                rest.xy.extend(z)
            };

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
                target: "windowed.digimon_playback",
                presentation_id = sprite.presentation_id.as_str(),
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
                "digimon windowed playback tick"
            );

            let pending_release = if let DigimonPlaybackMode::Skill {
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
                        let target_xy = nearest_opposing_target_xy(
                            &sprite_positions,
                            sprite.unit_id,
                            team_of.get(&sprite.unit_id).copied(),
                            caster_xy,
                        );

                        for command in &node.on_enter {
                            let Some(descriptor) = VfxSpawnDescriptor::from_command(command) else {
                                continue;
                            };

                            let Some(target_xy) = target_xy else {
                                debug!(
                                    target: "windowed.digimon_playback",
                                    source_unit = ?sprite.unit_id,
                                    node = node_id,
                                    particle = %descriptor.particle.0,
                                    "SpawnParticle on_enter target could not be resolved"
                                );
                                continue;
                            };

                            // Map the authored particle name to the owned effect
                            // id(s) it spawns via the engine-generic registry (S04,
                            // the charge command also seeds the inward ember swirl).
                            // This name->effect map at the spawn boundary replaces
                            // VfxParticleKind dispatch; each id is then rendered
                            // through enoki (D043).
                            let effect_ids = effects
                                .on_enter
                                .map
                                .get(descriptor.particle.0.as_str())
                                .map(Vec::as_slice)
                                .unwrap_or(&[]);
                            let mut cue_spawned: u32 = 0;
                            for effect_id in effect_ids {
                                let spawned = spawn_effect_by_id(
                                    &mut commands,
                                    effect_id,
                                    caster_xy,
                                    target_xy,
                                    sprite.unit_id,
                                    render_sprite.flip_x,
                                    transform.scale.x,
                                    effects.enoki.as_deref(),
                                    effects.soft_material.as_ref().map(|m| &m.0),
                                );
                                cue_spawned += spawned;
                                trace!(
                                    target: "windowed.digimon_playback",
                                    effect_id = effect_id.as_str(),
                                    spawned,
                                    caster_xy = ?caster_xy,
                                    source_unit = ?sprite.unit_id,
                                    "spawned windowed vfx effect on node enter"
                                );
                            }

                            // Spawn-miss diagnostic: a cast cue that resolved to no
                            // particle (unmapped in OnEnterEffectRegistry, or its
                            // effect ids absent from EnokiVfxRegistry) would
                            // otherwise be a silent no-op. Warn once per cue id so
                            // an unregistered cue is visible by name rather than
                            // invisible (slice failure-visibility; S06 warn-once
                            // pattern). Registered cues that spawn stay silent.
                            if cue_spawned == 0
                                && cast_cue_spawn_miss_warned.insert(descriptor.particle.0.clone())
                            {
                                warn!(
                                    target: "windowed.digimon_playback",
                                    cue = descriptor.particle.0.as_str(),
                                    node = node_id,
                                    source_unit = ?sprite.unit_id,
                                    "cast cue spawned no particle — cue id unregistered in OnEnterEffectRegistry or its effect ids missing from EnokiVfxRegistry; warned once per cue id"
                                );
                            }
                        }
                    }
                }
            }

            if let Some((cue_id, lf)) = pending_release {
                let result = barrier.request_release(&cue_id);
                trace!(
                    target: "windowed.digimon_playback",
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
                    if let Some(release_effect_id) =
                        mode_skill_id.and_then(|skill_id| effects.skill_release.map.get(skill_id))
                    {
                        // Despawn the charge orb + ember swirl enoki emitters the
                        // instant the flame launches, so the mouth clears for the
                        // projectile. Membership is by ChargeEmberEnokiMarker (enoki-
                        // native) — cleared generically for every persistent emitter
                        // owned by this caster, regardless of which skill released.
                        for (marker_entity, marker) in &charge_ember_markers {
                            if marker.unit_id == sprite.unit_id {
                                commands.entity(marker_entity).despawn();
                                trace!(
                                    target: "windowed.digimon_playback",
                                    source_unit = ?sprite.unit_id,
                                    entity = ?marker_entity,
                                    "despawned charge/ember enoki emitter on flame launch"
                                );
                            }
                        }

                        if let Some(target_xy) = nearest_opposing_target_xy(
                            &sprite_positions,
                            sprite.unit_id,
                            team_of.get(&sprite.unit_id).copied(),
                            [transform.translation.x, transform.translation.y],
                        ) {
                            let spawned = spawn_effect_by_id(
                                &mut commands,
                                release_effect_id,
                                [transform.translation.x, transform.translation.y],
                                target_xy,
                                sprite.unit_id,
                                render_sprite.flip_x,
                                transform.scale.x,
                                effects.enoki.as_deref(),
                                effects.soft_material.as_ref().map(|m| &m.0),
                            );
                            trace!(
                                target: "windowed.digimon_playback",
                                effect_id = release_effect_id.as_str(),
                                spawned,
                                source_unit = ?sprite.unit_id,
                                target_xy = ?target_xy,
                                "spawned skill-release projectile effect"
                            );
                        } else {
                            debug!(
                                target: "windowed.digimon_playback",
                                source_unit = ?sprite.unit_id,
                                "skill-release projectile target could not be resolved on release"
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
                        target: "windowed.digimon_playback",
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
                        target: "windowed.digimon_playback",
                        unit_id = ?sprite.unit_id,
                        node = sprite.player.current_node.0.as_str(),
                        fade_ticks = DEATH_FADE_TICKS,
                        "death node exited; seeding fade-out off field (idle restore suppressed)"
                    );
                } else if let Some(stance_graph) = stance_reg
                    .resolve_snapshot(&AnimGraphId(sprite.stance_graph_id.clone().into()), &graphs)
                {
                    let preserve_missing = active_barrier.as_ref().and_then(|status| {
                        (sprite.graph.source == ResolvedAnimGraphSource::InstantFallback
                            && sprite.last_missing_skill_graph_cue.as_deref()
                                == Some(status.cue_id))
                        .then(|| status.cue_id.to_string())
                    });
                    sprite.return_to_idle(stance_graph, preserve_missing);
                    trace!(
                        target: "windowed.digimon_playback",
                        "digimon playback returned to idle"
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
    mut sprites: Query<&mut DigimonSprite>,
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

    let hurt_node = StanceReaction::Hurt.stance_node();

    for mut sprite in &mut sprites {
        if !struck.contains(&sprite.unit_id) {
            continue;
        }
        // Resolve this sprite's own stance graph by its carried id (data, not a
        // const) — adding a Digimon with a different stance graph needs no edit here.
        let Some(stance_graph) = stance_reg
            .resolve_snapshot(&AnimGraphId(sprite.stance_graph_id.clone().into()), &graphs)
        else {
            continue;
        };
        // Do not interrupt an in-flight skill cast on the struck unit (S01).
        if !matches!(sprite.mode, DigimonPlaybackMode::Idle) {
            trace!(
                target: "windowed.digimon_playback",
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
            target: "windowed.digimon_playback",
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
/// dying sprite is also tagged [`DeathExiting`] so `advance_digimon_presentation`
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
    mut sprites: Query<(Entity, &mut DigimonSprite)>,
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

    let death_node = StanceReaction::Death.stance_node();

    for (entity, mut sprite) in &mut sprites {
        if !dying.contains(&sprite.unit_id) {
            continue;
        }
        // Resolve this sprite's own stance graph by its carried id (data, not a const).
        let Some(stance_graph) = stance_reg
            .resolve_snapshot(&AnimGraphId(sprite.stance_graph_id.clone().into()), &graphs)
        else {
            continue;
        };
        let prior_mode = sprite.mode.clone();
        // Death interrupts any in-flight skill: drive unconditionally (no
        // `matches!(mode, Idle)` guard, unlike the hurt path).
        sprite.drive_stance_reaction(death_node.clone(), stance_graph.clone());
        commands.entity(entity).insert(DeathExiting);
        trace!(
            target: "windowed.digimon_playback",
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

/// Linear fade alpha for the off-field death exit: `1.0` at full `remaining_ticks`,
/// `0.0` once spent. `total_ticks == 0` saturates to `1.0` (the `.max(1)` guards
/// the divide), so a zero-length fade never divides by zero (Q5).
fn fade_alpha(remaining_ticks: u32, total_ticks: u32) -> f32 {
    (remaining_ticks as f32 / total_ticks.max(1) as f32).clamp(0.0, 1.0)
}

/// Lerp a [`FadeOut`] sprite's alpha to 0 over its `total_ticks`, then despawn it.
///
/// Runs on the same `PendingAnimationTicks` clock as the presentation chain and is
/// ordered strictly after `advance_digimon_presentation`, so a sprite seeded with
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
                    target: "windowed.digimon_playback",
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

/// Spawn effect `effect_id` through bevy_enoki's GPU 2D backend — the sole
/// particle VFX renderer (M006/S01, D043). Looks the id up in the enoki handle
/// map, computes the placement `base` from the anchor carried in the map entry
/// (migrated out of `VfxAsset` in T02), and spawns the enoki `ParticleSpawner`
/// with its T03 lifecycle tag. Returns 1 on a spawn, 0 if the resource is absent
/// or the id is unmapped (the caller logs; a load failure is surfaced once by
/// `diagnose_enoki_vfx_load`). No kernel/FSM cue or barrier control flow is
/// touched (D031/D032).
#[allow(clippy::too_many_arguments)]
fn spawn_effect_by_id(
    commands: &mut Commands,
    effect_id: &str,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
    source_flip_x: bool,
    source_scale: f32,
    enoki: Option<&EnokiVfxRegistry>,
    soft_material: Option<&Handle<SpriteParticle2dMaterial>>,
) -> u32 {
    let Some(enoki) = enoki else {
        return 0;
    };
    let Some(entry) = enoki.handles.get(effect_id) else {
        return 0;
    };
    // The soft-particle material is built at Startup; if it is not yet present
    // (asset/resource ordering) we skip rather than fall back to enoki's
    // flat-square `ColorParticle2dMaterial` default — a missed spawn is surfaced
    // by the caller's spawn-miss diagnostic, a square is a silent regression.
    let Some(soft_material) = soft_material else {
        return 0;
    };
    let base = anchor_base_xy(
        entry.anchor,
        caster_xy,
        target_xy,
        source_flip_x,
        source_scale,
    );
    let mut spawned = commands.spawn((
        ParticleSpawner(soft_material.clone()),
        ParticleEffectHandle(entry.handle.clone()),
        Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z),
    ));
    // The lifecycle is data carried per registry entry (S04), not a closed
    // effect-id match. PersistentEmitter effects (the Baby Flame charge orb +
    // ember swirl) keep emitting at the mouth until `advance_digimon_presentation`
    // clears them by marker at the launch boundary; a Projectile travels
    // caster->target under `advance_enoki_projectiles`, which chains `on_arrival`
    // on arrival. OneShot effects (the impact/detonate/slash contact bursts) are
    // fire-and-forget and carry `OneShot::Despawn` so the spawner is removed once
    // it drains.
    match &entry.lifecycle {
        EnokiLifecycle::PersistentEmitter => {
            spawned.insert(ChargeEmberEnokiMarker {
                unit_id: source_unit,
            });
        }
        EnokiLifecycle::Projectile {
            flight_ticks,
            on_arrival,
        } => {
            spawned.insert(ProjectileFlight {
                from_xy: base,
                to_xy: target_xy,
                ticks_total: *flight_ticks,
                ticks_elapsed: 0,
                on_arrival: on_arrival.clone(),
            });
        }
        EnokiLifecycle::OneShot => {
            spawned.insert(OneShot::Despawn);
        }
    }
    1
}

fn should_spawn_node_vfx(
    mode: &DigimonPlaybackMode,
    active_barrier: Option<&CueBarrierStatus>,
    unit_id: UnitId,
) -> bool {
    matches!(mode, DigimonPlaybackMode::Skill { .. })
        && active_barrier
            .map(|status| barrier_targets_sprite(status, unit_id))
            .unwrap_or(true)
}

/// Vertical rest offset for `slot` (0-based) of a `count`-member team column,
/// centered on y=0 so a team fans out symmetrically (e.g. 2 members → ±75).
fn slot_offset_y(slot: usize, count: usize) -> f32 {
    let centered = slot as f32 - (count.max(1) as f32 - 1.0) / 2.0;
    centered * SLOT_VERTICAL_SPACING
}

/// Resolve the VFX impact point for a skill: the nearest sprite on the *opposing*
/// team, falling back to the nearest non-caster sprite if the caster's team is
/// unknown or no opponent has a live sprite.
///
/// The team filter is the S08 fix for multi-ally compositions. Targeting purely
/// by proximity ("nearest non-caster") aimed VFX at whichever sprite was closest
/// — fine with a single enemy, but with two allies sharing a column the closest
/// sprite was the *other ally* at ~0 distance, so projectiles spawned at and
/// "flew" to the caster's own teammate and read as invisible.
fn nearest_opposing_target_xy(
    sprite_positions: &[(UnitId, Team, [f32; 2])],
    caster: UnitId,
    caster_team: Option<Team>,
    caster_xy: [f32; 2],
) -> Option<[f32; 2]> {
    let nearest = |accept: &dyn Fn(Team) -> bool| {
        sprite_positions
            .iter()
            .filter(|(unit_id, team, _)| *unit_id != caster && accept(*team))
            .map(|(_, _, xy)| {
                let dx = xy[0] - caster_xy[0];
                let dy = xy[1] - caster_xy[1];
                (*xy, dx * dx + dy * dy)
            })
            .min_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
            .map(|(xy, _)| xy)
    };

    caster_team
        .and_then(|ct| nearest(&move |team| team != ct))
        .or_else(|| nearest(&|_| true))
}

fn find_sprite_xy(
    sprites: &Query<(&DigimonSprite, &Transform)>,
    unit_id: UnitId,
) -> Option<[f32; 2]> {
    sprites.iter().find_map(|(sprite, transform)| {
        (sprite.unit_id == unit_id).then_some([transform.translation.x, transform.translation.y])
    })
}

/// Bridge the combat event bus to a world-space `Text2d` damage number on the
/// pixel canvas over each struck target (S03/T03, the slice headline).
///
/// For every `CombatEvent` whose pure lib mapping ([`hit_damage_amount`]) yields
/// `Some(amount)` (i.e. `OnHitTaken`), resolve the target's live sprite XY via
/// [`find_sprite_xy`] and, if resolved, spawn one `Text2d` showing the integer
/// amount at `(x, y + offset)` with [`DAMAGE_NUMBER_Z`] above the VFX layer.
///
/// One number is spawned PER hit (never deduped): a multi-hit window shows
/// multiple numbers. A hit for a target with no live sprite resolves to `None`
/// and is skipped — no orphan number (Q5). Owns its own message cursor (MEM065);
/// reads events and spawns presentation entities only, never mutating combat or
/// kernel state (R010). The white number is sourced from `OnHitTaken.amount`;
/// kind-coloring is explicitly out of scope (S03-RESEARCH).
fn spawn_canvas_damage_numbers(
    mut commands: Commands,
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    sprites: Query<(&DigimonSprite, &Transform)>,
) {
    for event in events.read() {
        let Some(amount) = hit_damage_amount(&event.kind) else {
            continue;
        };
        let Some(xy) = find_sprite_xy(&sprites, event.target) else {
            debug!(
                target: "windowed.digimon_playback",
                unit_id = ?event.target,
                amount,
                "canvas damage number target sprite could not be resolved; skipped"
            );
            continue;
        };
        let base_y = xy[1] + DAMAGE_NUMBER_SPAWN_OFFSET_Y_PX;
        commands.spawn((
            Text2d::new(amount.to_string()),
            TextFont {
                font_size: DAMAGE_NUMBER_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(xy[0], base_y, DAMAGE_NUMBER_Z),
            CanvasDamageNumber {
                age_ticks: 0,
                total_ticks: DAMAGE_NUMBER_TICKS,
                base_y,
            },
        ));
        trace!(
            target: "windowed.digimon_playback",
            unit_id = ?event.target,
            amount,
            "spawned canvas damage number"
        );
    }
}

/// Advance every floating damage number on the shared `PendingAnimationTicks`
/// clock: per tick, age the number, apply [`damage_number_kinematics`] to rise
/// its Y absolutely from the captured `base_y` and fade its text alpha, then
/// despawn it once its lifetime is spent so numbers cannot accumulate unbounded
/// (Q6). Writes only `Transform`/`TextColor` and despawn — strictly downstream
/// presentation, never feeding the kernel (R004). An entity despawned by another
/// path mid-life simply stops being yielded by the query (no panic, Q5).
fn advance_canvas_damage_numbers(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    mut numbers: Query<(
        Entity,
        &mut CanvasDamageNumber,
        &mut Transform,
        &mut TextColor,
    )>,
) {
    for _ in 0..pending_ticks.0 {
        for (entity, mut number, mut transform, mut color) in &mut numbers {
            number.age_ticks += 1;
            let (rise_px, alpha) = damage_number_kinematics(number.age_ticks, number.total_ticks);
            transform.translation.y = number.base_y + rise_px;
            color.0 = color.0.with_alpha(alpha);
            if number.age_ticks >= number.total_ticks {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_detonate_particles(
    mut commands: Commands,
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    agumon_enoki_vfx: Option<Res<EnokiVfxRegistry>>,
    detonate_reg: Res<DetonateEffectRegistry>,
    soft_material: Option<Res<SoftParticleMaterial>>,
    sprites: Query<(&DigimonSprite, &Transform)>,
) {
    let Some(trigger) = latest_baby_burner_flash_trigger(events.read()) else {
        return;
    };

    // The detonate burst effect id is data (S04); no registered detonate effect
    // means nothing to spawn.
    let Some(detonate_effect_id) = detonate_reg.effect_id.as_deref() else {
        return;
    };

    let Some(caster_xy) = find_sprite_xy(&sprites, trigger.source) else {
        debug!(
            target: "windowed.digimon_playback",
            source_unit = ?trigger.source,
            cast_id = ?trigger.cast_id,
            "Baby Burner detonate particle source sprite could not be resolved"
        );
        return;
    };

    for target in trigger.targets {
        let Some(target_xy) = find_sprite_xy(&sprites, target) else {
            debug!(
                target: "windowed.digimon_playback",
                source_unit = ?trigger.source,
                target_unit = ?target,
                cast_id = ?trigger.cast_id,
                "Baby Burner detonate particle target could not be resolved"
            );
            continue;
        };

        let spawned = spawn_effect_by_id(
            &mut commands,
            detonate_effect_id,
            caster_xy,
            target_xy,
            trigger.source,
            false,
            1.0,
            agumon_enoki_vfx.as_deref(),
            soft_material.as_ref().map(|m| &m.0),
        );
        trace!(
            target: "windowed.digimon_playback",
            cast_id = ?trigger.cast_id,
            effect_id = detonate_effect_id,
            spawned,
            source_unit = ?trigger.source,
            target_unit = ?target,
            "spawned Baby Burner detonate effect"
        );
    }
}

/// Advance every in-flight Baby Flame projectile emitter on the shared
/// `PendingAnimationTicks` clock (M006/S01 T03, D046). Each tick the entity's
/// `Transform` is lerped linearly from `ProjectileFlight::from_xy` to `to_xy` over
/// `ticks_total` ticks; on arrival (`ticks_elapsed >= ticks_total`) the projectile
/// spawner is despawned and `baby_flame.impact` is spawned at `to_xy`, reproducing
/// the deleted quad path's `on_expire` projectile->impact chain through the enoki
/// backend. Runs in the presentation chain slot the quad `advance_vfx_particles`
/// occupies, strictly before `advance_digimon_presentation`. Presentation-only and
/// fire-and-forget: it never reads or mutates combat/kernel state (R010, D031/D032).
fn advance_enoki_projectiles(
    mut commands: Commands,
    pending_ticks: Res<PendingAnimationTicks>,
    agumon_enoki_vfx: Option<Res<EnokiVfxRegistry>>,
    soft_material: Option<Res<SoftParticleMaterial>>,
    mut projectiles: Query<(Entity, &mut Transform, &mut ProjectileFlight)>,
) {
    for _ in 0..pending_ticks.0 {
        for (entity, mut transform, mut flight) in &mut projectiles {
            flight.ticks_elapsed += 1;
            let t = if flight.ticks_total == 0 {
                1.0
            } else {
                (flight.ticks_elapsed as f32 / flight.ticks_total as f32).clamp(0.0, 1.0)
            };
            transform.translation.x = flight.from_xy[0] + (flight.to_xy[0] - flight.from_xy[0]) * t;
            transform.translation.y = flight.from_xy[1] + (flight.to_xy[1] - flight.from_xy[1]) * t;
            transform.translation.z = VFX_PARTICLE_Z;

            if flight.ticks_elapsed >= flight.ticks_total {
                // Arrival: clear the traveling emitter and chain the `on_arrival`
                // burst at the target (data carried on the flight, S04). The
                // placeholder `UnitId` is unused on the enoki path (the arrival burst
                // is a fire-and-forget `OneShot::Despawn`). Mirrors the old `on_expire`
                // chain's pos,pos.
                let on_arrival = flight.on_arrival.clone();
                let to_xy = flight.to_xy;
                commands.entity(entity).despawn();
                spawn_effect_by_id(
                    &mut commands,
                    &on_arrival,
                    to_xy,
                    to_xy,
                    UnitId(0),
                    false,
                    1.0,
                    agumon_enoki_vfx.as_deref(),
                    soft_material.as_ref().map(|m| &m.0),
                );
                trace!(
                    target: "windowed.digimon_playback",
                    entity = ?entity,
                    to_xy = ?to_xy,
                    effect_id = on_arrival.as_str(),
                    "enoki projectile arrived; chained arrival burst"
                );
            }
        }
    }
}

fn sync_digimon_mode(
    sprite: &mut DigimonSprite,
    active_barrier: Option<&CueBarrierStatus>,
    skill_reg: &SkillGraphRegistry,
    stance_reg: &StanceGraphRegistry,
    graphs: &Assets<AnimGraph>,
    start_node_reg: &SkillStartNodeRegistry,
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
    // are handled by the auto-release fallback in `advance_digimon_presentation`.
    // The bridged-skill -> FSM entry node map is engine-generic registry data
    // (S04) the per-Digimon module populates; absence means unbridged.
    let Some(start_node) = start_node_reg
        .map
        .get(status.skill_id.0.as_str())
        .map(String::as_str)
    else {
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
            if let DigimonPlaybackMode::Skill {
                awaiting_cue_id, ..
            } = &mut sprite.mode
            {
                *awaiting_cue_id = status.cue_id.to_string();
            }
            sprite.last_release_frame = None;
            trace!(
                target: "windowed.digimon_playback",
                skill_id = %status.skill_id.0,
                awaiting_cue_id = status.cue_id,
                hop_index = ?status.hop_index,
                node = %sprite.player.current_node.0,
                "digimon multi-barrier cue advanced (player not reset)"
            );
            sprite.last_missing_skill_graph_cue = None;
            return;
        }
        SameSkillSync::DifferentSkill => {}
    }

    if sprite.last_missing_skill_graph_cue.as_deref() == Some(status.cue_id) {
        return;
    }

    // Resolve the skill graph by this sprite's carried id (data, not a const).
    let skill_graph_id = sprite.skill_graph_id.clone();
    let resolved_graph = skill_reg.resolve_snapshot_or_instant_fallback(
        &AnimGraphId(skill_graph_id.clone().into()),
        graphs,
        lookup_diagnostics,
    );

    if resolved_graph.source == ResolvedAnimGraphSource::InstantFallback {
        warn!(
            target: "windowed.digimon_playback",
            cue_id = status.cue_id,
            skill_id = %status.skill_id.0,
            graph_id = %skill_graph_id,
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
        target: "windowed.digimon_playback",
        cue_id = status.cue_id,
        skill_id = %status.skill_id.0,
        start_node = %sprite.player.current_node.0,
        graph_source = ?sprite.graph.source,
        "skill playback entered start node"
    );

    if sprite.graph.source == ResolvedAnimGraphSource::InstantFallback {
        let stance_graph_id = sprite.stance_graph_id.clone();
        if let Some(stance_graph) =
            stance_reg.resolve_snapshot(&AnimGraphId(stance_graph_id.clone().into()), graphs)
        {
            trace!(
                target: "windowed.digimon_playback",
                graph_id = %stance_graph_id,
                stance_entry = %stance_graph.graph().entry.0,
                "stance snapshot remains available for post-fallback idle restore"
            );
        }
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
    mode: &DigimonPlaybackMode,
    skill_id: &str,
    cue_id: &str,
) -> SameSkillSync {
    match mode {
        DigimonPlaybackMode::Skill {
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
fn mode_trace_fields(mode: &DigimonPlaybackMode) -> (Option<&str>, Option<&str>) {
    match mode {
        DigimonPlaybackMode::Idle => (None, None),
        DigimonPlaybackMode::Skill {
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
/// unbridged fallback. Bridged skills (those with a windowed FSM entry node in
/// [`SkillStartNodeRegistry`]) release on their rendered `ReleaseKernel` cue
/// instead, so they are NOT auto-released here.
pub(in crate::windowed) fn should_auto_release_unbridged(
    reg: &SkillStartNodeRegistry,
    skill_id: &str,
) -> bool {
    !reg.map.contains_key(skill_id)
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
        assert!(!is_death_reaction(&CombatEventKind::OnHitTaken {
            amount: 5
        }));
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
    fn same_skill_cue_hop_advances_without_resetting_player() {
        // Generic FSM reconciliation: uses neutral skill-id / cue / node strings so
        // the engine test carries no per-Digimon coupling (the Agumon start-node
        // mapping is proven in the agumon module's own tests, S04).
        let mode = DigimonPlaybackMode::Skill {
            skill_id: "skill_ult".into(),
            awaiting_cue_id: "skill_ult/windup".into(),
            start_node: "skill_ult_charge".into(),
        };

        // Same skill, same cue: nothing to do (player keeps advancing).
        assert_eq!(
            classify_same_skill_sync(&mode, "skill_ult", "skill_ult/windup"),
            SameSkillSync::Unchanged
        );
        // Same skill, the awaiting cue hopped to the next barrier within the cast:
        // refresh the cue + dedup guard, but the player is NOT restarted.
        assert_eq!(
            classify_same_skill_sync(&mode, "skill_ult", "skill_ult/impact"),
            SameSkillSync::CueChanged
        );
        // A different skill (or Idle) forces a fresh start.
        assert_eq!(
            classify_same_skill_sync(&mode, "skill_other", "skill_other/impact"),
            SameSkillSync::DifferentSkill
        );
        assert_eq!(
            classify_same_skill_sync(&DigimonPlaybackMode::Idle, "skill_ult", "x"),
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
