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
