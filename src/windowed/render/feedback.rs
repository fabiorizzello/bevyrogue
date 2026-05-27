//! Hit-feedback, stance-reaction, and damage-number systems for the windowed
//! presentation layer.
//!
//! Owns camera-shake arming/application ([`observe_camera_shake`],
//! [`apply_camera_shake`]), hurt/death stance reaction drivers
//! ([`drive_hurt_reactions`], [`drive_death_reactions`]), floating damage
//! numbers ([`spawn_canvas_damage_numbers`], [`advance_canvas_damage_numbers`]),
//! and the off-field death fade ([`advance_death_fade`]).

use bevy::prelude::*;

use bevyrogue::animation::{AnimGraph, AnimGraphId, StanceGraphRegistry, StanceReaction, stance_reaction_for};
use bevyrogue::combat::types::UnitId;
use bevyrogue::ui::cues::{CueDef, CueRegistry, shake_offset_parametric};
use bevyrogue::ui::hit_feedback::{
    damage_number_kinematics, hit_damage_amount,
};

use super::{
    CameraRest, CameraShakeState, CanvasDamageNumber, DeathExiting, DigimonSprite,
    DigimonPlaybackMode, FadeOut,
    DAMAGE_NUMBER_FONT_SIZE, DAMAGE_NUMBER_SPAWN_OFFSET_Y_PX, DAMAGE_NUMBER_TICKS,
    DAMAGE_NUMBER_Z,
};
use super::clock::PendingAnimationTicks;
use super::playback::find_sprite_xy;

/// Linear fade alpha for the off-field death exit: `1.0` at full `remaining_ticks`,
/// `0.0` once spent. `total_ticks == 0` saturates to `1.0` (the `.max(1)` guards
/// the divide), so a zero-length fade never divides by zero (Q5).
pub(super) fn fade_alpha(remaining_ticks: u32, total_ticks: u32) -> f32 {
    (remaining_ticks as f32 / total_ticks.max(1) as f32).clamp(0.0, 1.0)
}

/// `true` iff the pure lib mapping classifies this event kind as a death
/// reaction. The death pipeline gates on this; a non-death event (e.g.
/// `OnHitTaken`) must never enter it (Q7 negative test).
pub(super) fn is_death_reaction(kind: &bevyrogue::combat::events::CombatEventKind) -> bool {
    stance_reaction_for(kind) == Some(StanceReaction::Death)
}

/// Arm the camera-shake window on every `OnHitTaken` — the SAME signal that arms
/// `HitShakeState` — sizing it to the `camera_impact` cue's tick count from the
/// `CueRegistry`. Owns its own message cursor (MEM065); same-window multi-hit
/// collapses via the reset in `arm`. Emits a `trace!` on the
/// `windowed.digimon_playback` target (mirroring the `flash+shake armed` seam) so
/// a future agent can confirm the cue fired without running the binary (K001).
pub(super) fn observe_camera_shake(
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
pub(super) fn apply_camera_shake(
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
pub(super) fn drive_hurt_reactions(
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut sprites: Query<&mut DigimonSprite>,
) {
    use std::collections::HashSet;
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
pub(super) fn drive_death_reactions(
    mut commands: Commands,
    mut events: MessageReader<bevyrogue::combat::events::CombatEvent>,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut sprites: Query<(Entity, &mut DigimonSprite)>,
) {
    use std::collections::HashSet;
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
pub(super) fn spawn_canvas_damage_numbers(
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
pub(super) fn advance_canvas_damage_numbers(
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

/// Lerp a [`FadeOut`] sprite's alpha to 0 over its `total_ticks`, then despawn it.
///
/// Runs on the same `PendingAnimationTicks` clock as the presentation chain and is
/// ordered strictly after `advance_digimon_presentation`, so a sprite seeded with
/// `FadeOut` in this frame's death-exit branch begins fading on the next tick.
/// Writes only `Sprite.color` and despawn — strictly downstream of presentation,
/// never feeding the kernel (R004). An entity despawned by another path mid-fade
/// simply stops being yielded by the query (no panic, Q5).
pub(super) fn advance_death_fade(
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevyrogue::combat::events::CombatEventKind;

    #[test]
    fn is_death_reaction_only_matches_unit_died() {
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
}
