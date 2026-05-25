//! Pure placement verbs (M004/S02): closed-form `[f32; 2]` offsets driven by a
//! render-free [`PlacementCtx`] + typed [`PlacementParams`].
//!
//! These are the functions registered on the `PlacementExt` Registry axis. Each
//! is pure and deterministic (R004): no wall-clock, no RNG, no Bevy World/render
//! types (R016) — identical input yields bit-identical output. A verb returns
//! `[0.0, 0.0]` (no offset) when handed a `PlacementParams` variant it does not
//! own, rather than panicking, so a mis-paired (verb, params) at load degrades to
//! a static particle instead of a crash (slice verification).

use crate::animation::vfx_asset::{PlacementCtx, PlacementParams};

/// Inward-spiraling swirl. Ports `baby_flame_ember_offset` (render.rs:948-957):
/// `radius = radius_px * (1 - progress)`, `angle = phase + age_ticks * omega`.
/// Radius collapses to the anchor as the ember merges into the core flame.
pub fn converge_inward(ctx: &PlacementCtx, params: &PlacementParams) -> [f32; 2] {
    let PlacementParams::ConvergeInward { radius_px, omega } = params else {
        return [0.0, 0.0];
    };
    let radius = radius_px * (1.0 - ctx.progress);
    let angle = ctx.phase + (ctx.age_ticks as f32 * omega);
    [radius * angle.cos(), radius * angle.sin()]
}

/// Radial burst along `phase`. Ports `baby_flame_shard_offset` (render.rs:972-981)
/// as a pure direction*distance fn: `eased = 1 - (1 - progress)^2`,
/// `dist = spread_px * eased`, returning `[dist*cos(phase), dist*sin(phase)]`.
///
/// NOTE: the windowed dispatcher will instead drive the shard's outward distance
/// via `eval_scale(scale_curve)` (as S01 does); `fan_out` itself stays a pure
/// direction*spread function and is the canonical placement verb for `FanOut`.
pub fn fan_out(ctx: &PlacementCtx, params: &PlacementParams) -> [f32; 2] {
    let PlacementParams::FanOut { spread_px } = params else {
        return [0.0, 0.0];
    };
    let eased = 1.0 - (1.0 - ctx.progress) * (1.0 - ctx.progress);
    let dist = spread_px * eased;
    [dist * ctx.phase.cos(), dist * ctx.phase.sin()]
}

/// Closed-form launch lerp toward the target:
/// `[(target - caster) * progress]` per axis.
///
/// This replaces the stateful per-tick 0.55 lerp at render.rs:1371-1385 with a
/// deterministic closed form (K001 visual deviation: the original eased toward
/// the target geometrically each tick — a decaying step — whereas this is a
/// straight linear interpolation over normalized lifetime; the endpoints match
/// but the in-between trajectory is linear instead of ease-out).
pub fn arc_launch(ctx: &PlacementCtx, params: &PlacementParams) -> [f32; 2] {
    let PlacementParams::ArcLaunch {} = params else {
        return [0.0, 0.0];
    };
    [
        (ctx.target_xy[0] - ctx.caster_xy[0]) * ctx.progress,
        (ctx.target_xy[1] - ctx.caster_xy[1]) * ctx.progress,
    ]
}

/// Fixed at the anchor: contributes no offset.
pub fn static_placement(_ctx: &PlacementCtx, params: &PlacementParams) -> [f32; 2] {
    let PlacementParams::Static = params else {
        return [0.0, 0.0];
    };
    [0.0, 0.0]
}
