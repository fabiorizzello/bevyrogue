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

/// Deterministic pseudo-turbulence wander for follow-through embers/smoke. Two
/// decorrelated sine octaves seeded by the per-particle `phase`, plus a net
/// vertical drift `rise_px` scaled by life progress. Pure (R004): no RNG, no
/// curl-noise texture — a closed form that reads as organic churn at the 14–34px
/// particle scale without a noise field, satisfying the "fire needs turbulence"
/// rule the VFX validations flag.
pub fn turbulence(ctx: &PlacementCtx, params: &PlacementParams) -> [f32; 2] {
    let PlacementParams::Turbulence { amp_px, freq, rise_px } = params else {
        return [0.0, 0.0];
    };
    let t = ctx.age_ticks as f32 * freq;
    let seed = ctx.phase;
    let wander_x = (t + seed).sin() + 0.5 * (2.3 * t + 1.7 * seed).sin();
    let wander_y = (1.3 * t + 2.1 * seed).cos() + 0.5 * (1.9 * t + seed).cos();
    [amp_px * wander_x, amp_px * wander_y + rise_px * ctx.progress]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(age: u32, phase: f32, progress: f32) -> PlacementCtx {
        PlacementCtx {
            age_ticks: age,
            ttl_ticks: 10,
            progress,
            phase,
            caster_xy: [0.0, 0.0],
            target_xy: [0.0, 0.0],
        }
    }

    #[test]
    fn turbulence_returns_zero_for_mismatched_params() {
        // A verb handed the wrong params variant degrades to no offset (no panic).
        assert_eq!(turbulence(&ctx(3, 1.0, 0.3), &PlacementParams::Static), [0.0, 0.0]);
    }

    #[test]
    fn turbulence_is_deterministic() {
        let p = PlacementParams::Turbulence { amp_px: 6.0, freq: 0.4, rise_px: 18.0 };
        let c = ctx(5, 2.1, 0.5);
        assert_eq!(turbulence(&c, &p), turbulence(&c, &p));
    }

    #[test]
    fn turbulence_amplitude_bounds_wander_and_rise_adds_drift() {
        let amp = 6.0;
        let rise = 18.0;
        let p = PlacementParams::Turbulence { amp_px: amp, freq: 0.4, rise_px: rise };
        // Wander is two octaves of unit sines scaled by amp: |component| <= 1.5*amp.
        // The rise term adds rise_px*progress to y on top of that bound.
        let c = ctx(7, 1.3, 0.5);
        let [x, y] = turbulence(&c, &p);
        assert!(x.abs() <= 1.5 * amp + 1e-4, "wander x within amplitude bound");
        assert!(y.abs() <= 1.5 * amp + rise * 0.5 + 1e-4, "y within wander+rise bound");
    }

    #[test]
    fn turbulence_rise_scales_with_progress() {
        // With zero frequency the wander term is constant across ticks, so the
        // y delta between two progress values is exactly the rise contribution.
        let rise = 20.0;
        let p = PlacementParams::Turbulence { amp_px: 0.0, freq: 0.0, rise_px: rise };
        let y_quarter = turbulence(&ctx(0, 0.7, 0.25), &p)[1];
        let y_full = turbulence(&ctx(0, 0.7, 1.0), &p)[1];
        assert!((y_full - y_quarter - rise * 0.75).abs() < 1e-5);
    }
}
