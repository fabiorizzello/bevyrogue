use std::collections::HashMap;

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use bevy_enoki::EnokiPlugin;

use bevyrogue::animation::{
    AnimGraphPlayer, AtlasGeometry, ResolvedAnimGraph,
};
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::ui::hit_feedback::{HitFlashState, HitShakeState, observe_hit_feedback};

pub(in crate::windowed) mod registries;

mod clock;
mod effects;
mod feedback;
mod playback;
mod spawn;

// Re-export the one item that external sibling modules (agumon tests) need directly.
// Only used in #[cfg(test)] code in agumon, so suppress the unused-imports lint.
#[allow(unused_imports)]
pub(in crate::windowed) use playback::should_auto_release_unbridged;

// Pull clock types into this module's namespace so submodules resolve correctly.
use clock::{AnimationClock, PendingAnimationTicks};

use registries::{
    DetonateEffectRegistry, EnokiVfxRegistry, OnEnterEffectRegistry,
    SkillReleaseEffectRegistry, SkillStartNodeRegistry, SoftParticleMaterial,
    SpritePresentationEntry, SpritePresentationRegistry,
};

// ─── Shared component types ──────────────────────────────────────────────────

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
        use bevyrogue::animation::NodeId;
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
    fn drive_stance_reaction(
        &mut self,
        node: bevyrogue::animation::NodeId,
        stance_graph: ResolvedAnimGraph,
    ) {
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

// ─── Layout / display constants ──────────────────────────────────────────────

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

// ─── Shared system params ─────────────────────────────────────────────────────

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

// ─── Plugin ───────────────────────────────────────────────────────────────────

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
            .add_systems(Startup, spawn::setup_camera)
            .add_systems(Startup, spawn::init_soft_particle_material)
            .add_systems(Update, effects::diagnose_enoki_vfx_load)
            .add_systems(Update, spawn::build_digimon_atlases.before(spawn::spawn_unit_sprites))
            .add_systems(Update, spawn::spawn_unit_sprites)
            .add_systems(
                Update,
                clock::sample_animation_ticks.before(effects::advance_enoki_projectiles),
            )
            .add_systems(
                Update,
                effects::spawn_detonate_particles
                    .after(resolve_action_system)
                    .after(spawn::spawn_unit_sprites)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                feedback::drive_hurt_reactions
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(playback::advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Arm the transient flash/shake windows off the CombatEvent bus.
                // Mirrors drive_hurt_reactions' ordering so the windows are armed
                // before advance_digimon_presentation decays + applies them.
                observe_hit_feedback
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(playback::advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Arm the camera-shake window off the SAME OnHitTaken signal that
                // arms HitShakeState (camera-shake is just another registered cue).
                // Ordered before advance_digimon_presentation, which owns the single
                // decay site.
                feedback::observe_camera_shake
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(playback::advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Spawn a world-space Text2d damage number over each struck
                // target. Ordered like drive_hurt_reactions so it reads the same
                // CombatEvent window before the presentation chain advances.
                feedback::spawn_canvas_damage_numbers
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(playback::advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Float/fade/despawn the damage numbers on the same animation
                // clock as advance_vfx_particles (disjoint component set).
                feedback::advance_canvas_damage_numbers.after(clock::sample_animation_ticks),
            )
            .add_systems(
                Update,
                // AFTER the hurt driver enforces death-precedence: a target both
                // struck and killed in one window resolves to `death`, not `hurt`.
                feedback::drive_death_reactions
                    .after(feedback::drive_hurt_reactions)
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(playback::advance_digimon_presentation)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                (
                    effects::advance_enoki_projectiles,
                    playback::advance_digimon_presentation,
                    feedback::advance_death_fade,
                )
                    .chain()
                    .after(clock::sample_animation_ticks)
                    .after(spawn::spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            )
            .add_systems(
                Update,
                // Write the Camera2d transform from the decayed CameraShakeState.
                // Ordered AFTER advance_digimon_presentation (the single decay site)
                // so it reads the freshly-drained remaining and applies an absolute
                // offset from CameraRest — never additive (MEM094).
                feedback::apply_camera_shake.after(playback::advance_digimon_presentation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::clock::{MAX_CATCHUP_TICKS};
    use super::effects::{anchor_base_xy, mouth_anchor_xy};
    use super::feedback::fade_alpha;
    use super::playback::{
        already_released_frame, barrier_targets_sprite, classify_same_skill_sync,
        entered_node, should_release_kernel, SameSkillSync,
    };

    use bevyrogue::animation::{AnimNode, FrameCue, FrameRange, FrameCueCommand, ReleaseKernelCue, PlacementAnchor};
    use bevyrogue::combat::runtime::intent::CastId;
    use bevyrogue::combat::runtime::CueBarrierStatus;
    use bevyrogue::combat::types::{SkillId, UnitId};

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
        use super::feedback::is_death_reaction;
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
        use super::clock::AnimationClock;
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
        use super::clock::AnimationClock;
        let mut clock = AnimationClock::new(12.0);
        // A one-second hitch is 12 periods' worth, but catch-up is bounded.
        assert_eq!(clock.tick(1.0), MAX_CATCHUP_TICKS);
        // Backlog beyond the cap is dropped, so the next normal frame is quiet.
        assert_eq!(clock.tick(1.0 / 60.0), 0);
    }

    #[test]
    fn anim_clock_with_nonpositive_fps_never_ticks() {
        use super::clock::AnimationClock;
        let mut clock = AnimationClock::new(0.0);
        assert_eq!(clock.tick(10.0), 0);
    }

    #[test]
    fn parse_anim_fps_defaults_and_validates() {
        use super::clock::{parse_anim_fps, DEFAULT_ANIM_FPS};
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
