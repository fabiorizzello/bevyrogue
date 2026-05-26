//! Windowed-only hit-feedback projection: transient flash/shake state plus the
//! pure kinematics used to render a floating damage number over the struck
//! target. Mirrors `TargetHurtState`/`observe_target_hurt` in
//! `crate::ui::combat_panel`: a deterministic, headless-testable projection of
//! the `CombatEvent` bus that NEVER mutates `CombatState` (R010).
//!
//! All testable logic lives here (LIB crate) so it has bare-`App` headless
//! tests; T02/T03 only apply these projections to `Sprite.color` / `Transform`
//! / `Text2d` in the binary `render.rs` (K001 — manual `cargo winx`).
//!
//! Decay is driven by the caller (render.rs passes the `PendingAnimationTicks`
//! count to `decay_by`) so there is a single decay source of truth; this module
//! deliberately ships NO frame-driven tick system.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::types::UnitId;

/// Ticks a struck unit stays in the colour-flash window. Sized for the ~12fps
/// animation clock that drives `PendingAnimationTicks`.
pub const FLASH_TICKS: u32 = 8;

/// Ticks a struck unit stays in the positional-shake window.
pub const SHAKE_TICKS: u32 = 8;

/// Peak shake displacement in pixels at the start of the shake window.
pub const SHAKE_MAX_PX: f32 = 4.0;

/// Peak rise (in pixels) of a floating damage number over its lifetime.
pub const DAMAGE_NUMBER_RISE_PX: f32 = 24.0;

/// Per-unit colour-flash countdown armed by `OnHitTaken`. Windowed-only
/// projection resource; never touches `CombatState`.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct HitFlashState {
    pub remaining: HashMap<UnitId, u32>,
}

impl HitFlashState {
    /// Arm (or re-arm) the flash for `target` to the full window. Idempotent:
    /// multiple hits in the same window collapse to a single full countdown.
    pub fn arm(&mut self, target: UnitId) {
        self.remaining.insert(target, FLASH_TICKS);
    }

    /// Drain every entry by `n` ticks (saturating, never underflows) and drop
    /// entries that reach zero.
    pub fn decay_by(&mut self, n: u32) {
        self.remaining.retain(|_, r| {
            *r = r.saturating_sub(n);
            *r > 0
        });
    }

    /// Remaining flash ticks for `id` (0 when not flashing).
    pub fn remaining(&self, id: UnitId) -> u32 {
        self.remaining.get(&id).copied().unwrap_or(0)
    }
}

/// Per-unit positional-shake countdown armed by `OnHitTaken`. Windowed-only
/// projection resource; never touches `CombatState`.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct HitShakeState {
    pub remaining: HashMap<UnitId, u32>,
}

impl HitShakeState {
    /// Arm (or re-arm) the shake for `target` to the full window. Idempotent.
    pub fn arm(&mut self, target: UnitId) {
        self.remaining.insert(target, SHAKE_TICKS);
    }

    /// Drain every entry by `n` ticks (saturating, never underflows) and drop
    /// entries that reach zero.
    pub fn decay_by(&mut self, n: u32) {
        self.remaining.retain(|_, r| {
            *r = r.saturating_sub(n);
            *r > 0
        });
    }

    /// Remaining shake ticks for `id` (0 when not shaking).
    pub fn remaining(&self, id: UnitId) -> u32 {
        self.remaining.get(&id).copied().unwrap_or(0)
    }
}

/// Extract the damage amount carried by a hit. Returns `Some(amount)` only for
/// `OnHitTaken`; every other event kind yields `None` so the feedback arms on
/// exactly the defender-facing hit signal.
pub fn hit_damage_amount(kind: &CombatEventKind) -> Option<i32> {
    match kind {
        CombatEventKind::OnHitTaken { amount } => Some(*amount),
        _ => None,
    }
}

/// Arm both the flash and shake windows for `event.target` on every
/// `OnHitTaken`. Owns its own message cursor (MEM065); same-window multi-hit
/// dedups automatically via the map reset in `arm`.
pub fn observe_hit_feedback(
    mut events: MessageReader<CombatEvent>,
    mut flash: ResMut<HitFlashState>,
    mut shake: ResMut<HitShakeState>,
) {
    for event in events.read() {
        if hit_damage_amount(&event.kind).is_some() {
            flash.arm(event.target);
            shake.arm(event.target);
        }
    }
}

/// Colour applied to the struck sprite during the flash window. Lerps from
/// `Color::WHITE` (no tint) at `remaining == 0` toward a bright red-white tint
/// at `remaining == total`. Returns exactly `Color::WHITE` when not flashing.
pub fn flash_tint(remaining: u32, total: u32) -> Color {
    if remaining == 0 || total == 0 {
        return Color::WHITE;
    }
    let t = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    // Bright hot tint the sprite biases toward at peak flash.
    const TINT: (f32, f32, f32) = (1.0, 0.45, 0.45);
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Color::srgb(lerp(1.0, TINT.0), lerp(1.0, TINT.1), lerp(1.0, TINT.2))
}

/// Deterministic decaying jitter applied to the struck sprite's transform. No
/// RNG (R004): a fixed sinusoid scaled by `remaining / total`, so amplitude
/// decays to zero as the window drains. Returns `Vec2::ZERO` when not shaking.
pub fn shake_offset(remaining: u32, total: u32) -> Vec2 {
    if remaining == 0 || total == 0 {
        return Vec2::ZERO;
    }
    let decay = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    let amplitude = SHAKE_MAX_PX * decay;
    let phase = remaining as f32;
    Vec2::new(amplitude * (phase * 1.7).sin(), amplitude * (phase * 2.3).cos())
}

/// Kinematics for a floating damage number: returns `(rise_px, alpha)` for a
/// number `age_ticks` into a `total_ticks` lifetime. `rise` grows monotonically
/// from 0 and `alpha` fades monotonically from 1.0 toward 0.0 as `age → total`.
pub fn damage_number_kinematics(age_ticks: u32, total_ticks: u32) -> (f32, f32) {
    if total_ticks == 0 {
        return (DAMAGE_NUMBER_RISE_PX, 0.0);
    }
    let t = (age_ticks as f32 / total_ticks as f32).clamp(0.0, 1.0);
    let rise = DAMAGE_NUMBER_RISE_PX * t;
    let alpha = 1.0 - t;
    (rise, alpha)
}
