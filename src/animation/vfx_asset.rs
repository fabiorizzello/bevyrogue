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

/// Typed placement reference. `verb` names a namespaced Registry id by string;
/// resolution is deferred to S02 (here it is data only). Kept as a struct so
/// future typed placement parameters extend it without a serde shape change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    /// Namespaced Registry verb id (e.g. `"impact.fan_out"`).
    pub verb: String,
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
