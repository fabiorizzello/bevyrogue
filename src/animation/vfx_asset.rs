//! Owned, editor-ready per-Digimon VFX schema (D033/D034).
//!
//! This is the typed counterpart to the per-Digimon `vfx.ron` asset. Unlike a
//! stringly-typed parameter bag, every verb parameter here is a named, typed,
//! introspectable field deriving [`Reflect`], so the future anim_graph+vfx GUI
//! editor can generate forms by reflection without forcing a schema refactor.
//!
//! The schema mirrors the closed-vocabulary discipline of
//! [`crate::animation::anim_graph`]: structs derive [`Serialize`] +
//! [`Deserialize`] with `#[serde(deny_unknown_fields)]` so malformed RON
//! (the only untrusted-shaped surface) fails at load with a precise,
//! field-naming parse error rather than silently dropping data.
//!
//! Scope (M004/S01): this lays the schema + data foundation only. Placement
//! verbs reference a namespaced Registry id *by string* — Registry resolution
//! itself is deferred to S02. No windowed dependency is introduced (R016).

use std::collections::BTreeMap;

use bevy::prelude::{Asset, Reflect};
use serde::{Deserialize, Serialize};

/// Opaque, namespaced effect identifier used as a `VfxAsset` map key and as the
/// `on_expire` chain reference. Stays a transparent string newtype so RON keeps
/// authoring effects by name.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
#[serde(transparent)]
pub struct EffectId(pub String);

/// Top-level owned VFX asset: a map of effect-id -> [`EffectDef`].
///
/// Deriving [`Asset`] lets `bevy_common_assets` load it from RON; deriving
/// [`Reflect`] (which also supplies the required `TypePath`) proves
/// editor-readiness (D034).
// `TypePath` is intentionally omitted from the derive list: the `Reflect` derive
// already generates a `TypePath` impl, and `Asset` only requires that the type
// implement `TypePath` (which Reflect satisfies). Listing both conflicts (E0119).
#[derive(Asset, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct VfxAsset {
    /// Each entry maps a namespaced effect id to its definition.
    pub effects: BTreeMap<EffectId, EffectDef>,
    /// Optional variant-selection table (D033 graft 5): `skill_id -> variant_key
    /// -> target [`EffectId`]`. Mirrors how anim_graph state picks which tree to
    /// instantiate, but stays closed data (D035) — not a new ExtRegistries axis.
    /// Nested `BTreeMap`s (not a tuple key) keep RON authoring readable and give a
    /// deterministic iteration order (R004). Defaulted + empty, so existing assets
    /// without a `variants` block keep loading under `deny_unknown_fields`.
    #[serde(default)]
    pub variants: BTreeMap<String, BTreeMap<String, EffectId>>,
}

/// One named effect: where it is placed, how it appears, and what (optionally)
/// chains from it when it expires.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct EffectDef {
    /// Placement reference into the (S02) Registry, expressed as data here.
    pub placement: Placement,
    /// Typed, introspectable appearance curves.
    pub appearance: Appearance,
    /// Optional chain: the effect id spawned when this effect expires.
    #[serde(default)]
    pub on_expire: Option<EffectId>,
}

/// Typed placement reference. `verb` names a namespaced `PlacementExt` Registry
/// id by string (resolved at the windowed layer in S02); `params` is the typed,
/// editor-ready payload that verb consumes; `anchor` is the world-space origin
/// the resolved offset is applied relative to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    /// Namespaced Registry verb id (e.g. `"agumon/baby_flame/fan_out"`).
    pub verb: String,
    /// Typed, closed-vocabulary parameter payload for the verb (D035).
    pub params: PlacementParams,
    /// World-space anchor the resolved offset is applied relative to.
    pub anchor: PlacementAnchor,
}

/// Typed, introspectable appearance description — the editor-readiness surface
/// the D034 reflection test asserts against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    /// Number of particles spawned by the effect.
    pub count: u32,
    /// Spatial spread, in pixels, applied at spawn.
    pub spread_px: f32,
    /// Lifetime of each particle, in deterministic ticks (R004).
    pub ttl_ticks: u32,
    /// Scale-over-lifetime curve.
    pub scale: ScaleCurve,
    /// Color-over-lifetime curve.
    pub color: ColorCurve,
    /// Per-particle quad size in pixels (replaces the per-kind
    /// `vfx_particle_size` match the dispatcher removes in T03).
    pub size_px: f32,
    /// Windowed-resolved image key (replaces the per-kind texture match the
    /// dispatcher removes in T03). Headless code never loads it.
    pub texture: String,
    /// Per-particle quad rotation policy (M004/S06). Defaulted so existing assets
    /// that omit it keep loading under `deny_unknown_fields`; [`RotationParams::Static`]
    /// reproduces the prior axis-aligned billboard behavior exactly.
    #[serde(default)]
    pub rotation: RotationParams,
}

/// Per-particle quad rotation policy (radians, CCW). Editor-ready closed
/// vocabulary (D034/D035) mirroring [`PlacementParams`]: each variant is a
/// closed-form, deterministic function of the per-tick [`PlacementCtx`],
/// evaluated by [`eval_rotation`].
///
/// Windowed billboards are otherwise axis-aligned, so this is the lever that
/// lets a single asymmetric sprite (e.g. one flame tongue) fan into a fire by
/// spawning many at decorrelated angles. Defaults to [`Static`](Self::Static) so
/// assets predating this field load unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub enum RotationParams {
    /// Axis-aligned: no rotation (default, == prior billboard behavior).
    #[default]
    Static,
    /// Orient by the per-particle `phase` (radial layouts: charge/impact
    /// tongues), plus a constant `offset_rad` and a per-tick spin `omega`.
    Radial {
        /// Constant angle added to `phase`, radians.
        offset_rad: f32,
        /// Spin rate, radians per tick.
        omega: f32,
    },
    /// Orient toward the target along the caster->target heading (e.g. a launched
    /// streak), plus a constant `offset_rad` and a per-tick spin `omega`.
    TowardTarget {
        /// Constant angle added to the caster->target heading, radians.
        offset_rad: f32,
        /// Spin rate, radians per tick.
        omega: f32,
    },
    /// Fixed orientation `angle_rad` plus a per-tick spin `omega`.
    Fixed {
        /// Base orientation, radians.
        angle_rad: f32,
        /// Spin rate, radians per tick.
        omega: f32,
    },
}

/// Scale-over-lifetime curve as an ordered list of keyframes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct ScaleCurve(pub Vec<ScaleKeyframe>);

/// Color-over-lifetime curve as an ordered list of keyframes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct ColorCurve(pub Vec<ColorKeyframe>);

/// A single scale keyframe: normalized time `t` in `[0, 1]` -> scalar `value`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct ScaleKeyframe {
    /// Normalized lifetime position, `0.0..=1.0`.
    pub t: f32,
    /// Scale multiplier at `t`.
    pub value: f32,
}

/// A single color keyframe: normalized time `t` in `[0, 1]` -> linear RGBA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct ColorKeyframe {
    /// Normalized lifetime position, `0.0..=1.0`.
    pub t: f32,
    /// Linear RGBA in `0.0..=1.0` per channel.
    pub rgba: [f32; 4],
}

// ─── Placement axis (M004/S02): runtime context + typed verb params ──────────
//
// The placement *verb* is named by string in [`Placement`] and resolved against
// the `PlacementExt` Registry (S02). A resolved verb is a pure fn
// `fn(&PlacementCtx, &PlacementParams) -> [f32; 2]` (see [`crate::animation::placement`]).
// `PlacementCtx` is the render-free per-tick input; `PlacementParams` is the
// editor-ready, closed-vocabulary, typed parameter payload. Neither references a
// Bevy World or render type (R004 / R016).

/// Pure, render-free per-tick context handed to a placement verb.
///
/// Carries only deterministic scalars (R004): no wall-clock, no RNG, no Bevy
/// World/render handles. `progress`/`phase` are precomputed by the caller so the
/// verb stays a closed-form pure function of its inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementCtx {
    /// Ticks elapsed since the particle spawned.
    pub age_ticks: u32,
    /// Total particle lifetime in ticks.
    pub ttl_ticks: u32,
    /// Normalized lifetime position, `age_ticks / ttl_ticks` clamped to `[0, 1]`.
    pub progress: f32,
    /// Per-particle angular phase (radians), e.g. swirl/fan seed.
    pub phase: f32,
    /// Caster anchor in world px.
    pub caster_xy: [f32; 2],
    /// Target anchor in world px.
    pub target_xy: [f32; 2],
}

/// Anchor an effect's placement resolves relative to. Editor-ready (Reflect),
/// closed vocabulary (D034/D035).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum PlacementAnchor {
    /// The caster's mouth muzzle anchor (charge/launch origin).
    Mouth,
    /// The caster's body center.
    CasterCenter,
    /// The target's body center (impact origin).
    TargetCenter,
}

/// Closed, editor-ready, typed parameter payload for a placement verb (D035).
///
/// Each variant maps 1:1 to a pure verb in [`crate::animation::placement`].
/// Struct-shaped variants derive `deny_unknown_fields` semantics so malformed
/// RON fails at load with a field-naming parse error (mirrors the rest of this
/// schema). `Reflect` proves editor-readiness — the GUI can build a form per
/// variant by reflection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub enum PlacementParams {
    /// Inward-spiraling swirl: radius shrinks linearly to the anchor while the
    /// angle sweeps at `omega` rad/tick.
    ConvergeInward {
        /// Starting radius in world px (at `progress == 0`).
        radius_px: f32,
        /// Angular sweep rate, radians per tick.
        omega: f32,
    },
    /// Radial burst along the per-particle `phase` direction, scaled by spread.
    FanOut {
        /// Maximum outward distance in world px (at `progress == 1`).
        spread_px: f32,
    },
    /// Closed-form lerp from caster toward target over the particle's life.
    ArcLaunch {},
    /// Deterministic sum-of-sines wander (no RNG, R004) for follow-through
    /// embers/smoke: organic drift without a noise field. `amp_px` is the wander
    /// amplitude, `freq` the base octave rate (radians/tick), `rise_px` a net
    /// vertical drift applied over life (positive = upward; negative falls).
    Turbulence {
        /// Wander amplitude in world px.
        amp_px: f32,
        /// Base wander frequency, radians per tick.
        freq: f32,
        /// Net vertical drift over full life, world px (progress-scaled).
        rise_px: f32,
    },
    /// Fixed at the anchor; the verb contributes no offset.
    Static,
}

/// Default scale returned by [`eval_scale`] for an empty curve.
const DEFAULT_SCALE: f32 = 1.0;
/// Default color (opaque white) returned by [`eval_color`] for an empty curve.
const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Concrete, render-free spawn parameters the windowed layer reads to build the
/// impact fan-out without re-deriving anything from the asset (R004: pure).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImpactSpawnPlan {
    /// Number of particles to spawn.
    pub count: u32,
    /// Spatial spread, in pixels, applied at spawn.
    pub spread_px: f32,
    /// Lifetime of each particle, in deterministic ticks.
    pub ttl_ticks: u32,
}

/// Look up an [`EffectDef`] by its namespaced id. Returns `None` when absent so
/// the windowed layer can log + fall back rather than panic (slice verification).
pub fn resolve_effect<'a>(asset: &'a VfxAsset, effect_id: &str) -> Option<&'a EffectDef> {
    asset.effects.get(&EffectId(effect_id.to_owned()))
}

/// Render-free, gameplay-numeric-free selection input for the variant seam
/// (D033 graft 5). A synthetic stand-in for a future skill-tree unlock context:
/// `skill_id` names the unlock bucket, `variant_key` the chosen variation within
/// it. Carries no numbers and no render/world handles (R012/MEM044, R004) — it
/// only keys into [`VfxAsset::variants`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VfxContext {
    /// Unlock bucket id, e.g. `"baby_burner"`.
    pub skill_id: String,
    /// Chosen variation within the bucket, e.g. `"empowered"`.
    pub variant_key: String,
}

/// Deterministically select an effect-tree variant for a synthetic
/// [`VfxContext`] (D033 graft 5). Looks up `variants[skill_id][variant_key]` and
/// delegates to [`resolve_effect`], returning `None` for any unmapped key or a
/// mapped-but-absent target so the caller falls back to the base effect — the
/// same None-not-panic discipline as [`resolve_effect`]. Pure and deterministic
/// (R004): no RNG, no clock; identical input yields an identical result.
pub fn select_variant<'a>(asset: &'a VfxAsset, ctx: &VfxContext) -> Option<&'a EffectDef> {
    let target = asset.variants.get(&ctx.skill_id)?.get(&ctx.variant_key)?;
    resolve_effect(asset, &target.0)
}

/// Why a [`VfxAsset`] failed headless load-time validation. Carries the offending
/// effect id and a precise reason so the windowed layer can warn once with the
/// Digimon/effect/verb that broke, then skip that effect (slice verification:
/// surfaced as data, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfxValidationError {
    /// `effect.placement.verb` is not in the set of registered verb ids.
    UnknownVerb {
        /// The effect whose placement names an unregistered verb.
        effect_id: String,
        /// The unresolvable verb id.
        verb: String,
    },
    /// `effect.on_expire` points at an effect id not present in the asset.
    DanglingOnExpire {
        /// The effect carrying the dangling chain reference.
        effect_id: String,
        /// The missing target effect id.
        missing: String,
    },
    /// A `variants[skill_id][variant_key]` target effect id is not present in the
    /// asset (D033 graft 5). Same warn-once + skip pattern as the other variants
    /// (MEM076): surfaced as data, never a panic.
    DanglingVariant {
        /// The unlock bucket carrying the dangling variant reference.
        skill_id: String,
        /// The variant key whose target is missing.
        variant_key: String,
        /// The missing target effect id.
        missing: String,
    },
}

impl std::fmt::Display for VfxValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownVerb { effect_id, verb } => {
                write!(f, "effect `{effect_id}` references unregistered placement verb `{verb}`")
            }
            Self::DanglingOnExpire { effect_id, missing } => {
                write!(f, "effect `{effect_id}` on_expire references absent effect `{missing}`")
            }
            Self::DanglingVariant { skill_id, variant_key, missing } => write!(
                f,
                "variant `{skill_id}`/`{variant_key}` references absent effect `{missing}`"
            ),
        }
    }
}

impl std::error::Error for VfxValidationError {}

/// Headless load-time validation (CONTEXT-mandated): every effect's placement
/// verb must be in `known_verbs`, and every `on_expire` id must resolve to a
/// present effect. Returns the first offending pair (deterministic `BTreeMap`
/// order, R004) as data so the caller warns + skips rather than panics. Pure: no
/// I/O, no Bevy world/render types.
pub fn validate_effects(
    asset: &VfxAsset,
    known_verbs: &[&str],
) -> Result<(), VfxValidationError> {
    for (id, effect) in &asset.effects {
        if !known_verbs.contains(&effect.placement.verb.as_str()) {
            return Err(VfxValidationError::UnknownVerb {
                effect_id: id.0.clone(),
                verb: effect.placement.verb.clone(),
            });
        }
        if let Some(target) = &effect.on_expire {
            if !asset.effects.contains_key(target) {
                return Err(VfxValidationError::DanglingOnExpire {
                    effect_id: id.0.clone(),
                    missing: target.0.clone(),
                });
            }
        }
    }
    // Variant targets are checked after the effect graph, in deterministic
    // BTreeMap order (skill_id then variant_key), returning the first dangling
    // target so the windowed layer warns + skips (D033 graft 5, MEM076).
    for (skill_id, by_key) in &asset.variants {
        for (variant_key, target) in by_key {
            if !asset.effects.contains_key(target) {
                return Err(VfxValidationError::DanglingVariant {
                    skill_id: skill_id.clone(),
                    variant_key: variant_key.clone(),
                    missing: target.0.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Build the concrete [`ImpactSpawnPlan`] for an effect from its [`Appearance`].
/// Pure: no I/O, no Bevy world/render types.
pub fn spawn_plan(effect: &EffectDef) -> ImpactSpawnPlan {
    ImpactSpawnPlan {
        count: effect.appearance.count,
        spread_px: effect.appearance.spread_px,
        ttl_ticks: effect.appearance.ttl_ticks,
    }
}

/// Evaluate a [`ScaleCurve`] at normalized lifetime `progress`.
///
/// Piecewise-linear between keyframes sorted by `t`. `progress` is clamped to
/// `[0, 1]`; before the first keyframe returns the first value, after the last
/// returns the last, and an empty curve returns [`DEFAULT_SCALE`]. Pure and
/// deterministic (R004): identical input yields identical output.
pub fn eval_scale(curve: &ScaleCurve, progress: f32) -> f32 {
    eval_curve(
        &curve.0,
        progress,
        DEFAULT_SCALE,
        |kf| kf.t,
        |kf| kf.value,
        |a, b, frac| a + (b - a) * frac,
    )
}

/// Evaluate a [`ColorCurve`] at normalized lifetime `progress`.
///
/// Same piecewise-linear semantics as [`eval_scale`] (per-channel linear
/// interpolation); an empty curve returns [`DEFAULT_COLOR`].
pub fn eval_color(curve: &ColorCurve, progress: f32) -> [f32; 4] {
    eval_curve(
        &curve.0,
        progress,
        DEFAULT_COLOR,
        |kf| kf.t,
        |kf| kf.rgba,
        |a, b, frac| {
            [
                a[0] + (b[0] - a[0]) * frac,
                a[1] + (b[1] - a[1]) * frac,
                a[2] + (b[2] - a[2]) * frac,
                a[3] + (b[3] - a[3]) * frac,
            ]
        },
    )
}

/// Evaluate a [`RotationParams`] to a quad rotation angle (radians, CCW) for the
/// current tick. Pure and deterministic (R004): identical [`PlacementCtx`] yields
/// identical output; no RNG, no clock, no Bevy world/render types. `TowardTarget`
/// reads the closed-form caster->target heading from the context.
pub fn eval_rotation(rotation: &RotationParams, ctx: &PlacementCtx) -> f32 {
    let spin = ctx.age_ticks as f32;
    match *rotation {
        RotationParams::Static => 0.0,
        RotationParams::Radial { offset_rad, omega } => ctx.phase + offset_rad + spin * omega,
        RotationParams::TowardTarget { offset_rad, omega } => {
            let dx = ctx.target_xy[0] - ctx.caster_xy[0];
            let dy = ctx.target_xy[1] - ctx.caster_xy[1];
            dy.atan2(dx) + offset_rad + spin * omega
        }
        RotationParams::Fixed { angle_rad, omega } => angle_rad + spin * omega,
    }
}

/// Shared piecewise-linear evaluator over an unsorted keyframe slice.
///
/// Sorts keyframe *indices* by `t` (no keyframe mutation, so callers keep their
/// authored order) using [`f32::total_cmp`] for a deterministic total order, then
/// clamps `progress`, handles the empty/before-first/after-last edges, and
/// linearly interpolates within the bracketing segment via `lerp`.
fn eval_curve<K, V: Copy>(
    keys: &[K],
    progress: f32,
    default: V,
    t_of: impl Fn(&K) -> f32,
    v_of: impl Fn(&K) -> V,
    lerp: impl Fn(V, V, f32) -> V,
) -> V {
    if keys.is_empty() {
        return default;
    }

    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by(|&a, &b| t_of(&keys[a]).total_cmp(&t_of(&keys[b])));

    let p = progress.clamp(0.0, 1.0);
    let first = order[0];
    if p <= t_of(&keys[first]) {
        return v_of(&keys[first]);
    }
    let last = order[order.len() - 1];
    if p >= t_of(&keys[last]) {
        return v_of(&keys[last]);
    }

    for pair in order.windows(2) {
        let (i0, i1) = (pair[0], pair[1]);
        let (t0, t1) = (t_of(&keys[i0]), t_of(&keys[i1]));
        if p >= t0 && p <= t1 {
            // `t1 > t0` here unless duplicate `t`s collapse the span; guard /0.
            let span = t1 - t0;
            let frac = if span > 0.0 { (p - t0) / span } else { 0.0 };
            return lerp(v_of(&keys[i0]), v_of(&keys[i1]), frac);
        }
    }

    // Unreachable: p is strictly between first and last t after the edge checks.
    v_of(&keys[last])
}

#[cfg(test)]
mod rotation_tests {
    use super::*;

    fn ctx(age: u32, phase: f32, caster: [f32; 2], target: [f32; 2]) -> PlacementCtx {
        PlacementCtx {
            age_ticks: age,
            ttl_ticks: 10,
            progress: age as f32 / 10.0,
            phase,
            caster_xy: caster,
            target_xy: target,
        }
    }

    #[test]
    fn static_rotation_is_always_zero() {
        let c = ctx(5, 1.23, [0.0, 0.0], [100.0, 0.0]);
        assert_eq!(eval_rotation(&RotationParams::Static, &c), 0.0);
    }

    #[test]
    fn radial_orients_by_phase_then_spins_with_age() {
        let rot = RotationParams::Radial { offset_rad: 0.5, omega: 0.1 };
        // age 0: phase + offset only.
        let a0 = eval_rotation(&rot, &ctx(0, 1.0, [0.0, 0.0], [0.0, 0.0]));
        assert!((a0 - 1.5).abs() < 1e-6);
        // age 3: + 3*omega.
        let a3 = eval_rotation(&rot, &ctx(3, 1.0, [0.0, 0.0], [0.0, 0.0]));
        assert!((a3 - (1.5 + 0.3)).abs() < 1e-6);
    }

    #[test]
    fn toward_target_points_along_caster_to_target_heading() {
        let rot = RotationParams::TowardTarget { offset_rad: 0.0, omega: 0.0 };
        // Target straight right -> heading 0.
        let right = eval_rotation(&rot, &ctx(0, 9.9, [10.0, 10.0], [110.0, 10.0]));
        assert!(right.abs() < 1e-6, "heading right is 0 rad, got {right}");
        // Target straight up -> heading +pi/2.
        let up = eval_rotation(&rot, &ctx(0, 9.9, [10.0, 10.0], [10.0, 110.0]));
        assert!((up - std::f32::consts::FRAC_PI_2).abs() < 1e-6, "heading up is pi/2, got {up}");
    }

    #[test]
    fn eval_rotation_is_deterministic() {
        let rot = RotationParams::Fixed { angle_rad: 0.2, omega: 0.05 };
        let c = ctx(7, 2.0, [1.0, 2.0], [3.0, 4.0]);
        assert_eq!(eval_rotation(&rot, &c), eval_rotation(&rot, &c));
    }

    #[test]
    fn rotation_defaults_to_static() {
        assert_eq!(RotationParams::default(), RotationParams::Static);
    }
}
