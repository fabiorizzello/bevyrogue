//! VFX / particle effect systems for the windowed presentation layer.
//!
//! Owns enoki load diagnostics ([`diagnose_enoki_vfx_load`]), the spawn seam
//! ([`spawn_effect_by_id`]), detonate-burst spawning ([`spawn_detonate_particles`]),
//! and in-flight projectile advancement ([`advance_enoki_projectiles`]).

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_enoki::prelude::{OneShot, SpriteParticle2dMaterial};
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner};

use bevyrogue::animation::PlacementAnchor;
use bevyrogue::combat::types::UnitId;
use bevyrogue::ui::combat_panel::latest_baby_burner_flash_trigger;

use super::{
    ChargeEmberEnokiMarker, ProjectileFlight, VFX_PARTICLE_Z, VFX_MOUTH_OFFSET_X_PX,
    VFX_MOUTH_OFFSET_Y_PX,
};
use super::registries::{
    DetonateEffectRegistry, EnokiLifecycle, EnokiVfxRegistry, SoftParticleMaterial,
};
use super::playback::find_sprite_xy;
use super::DigimonSprite;

/// Surface a load failure for each registered enoki `.particle.ron` once. A
/// failed/missing effect asset means that effect silently spawns nothing through
/// the enoki backend; this makes a dead burst visible by id+path rather than
/// silently absent (slice failure-visibility). Reads the source path from the
/// registry entry (S04) rather than a const match. Each id is warned at most once
/// via the warned-set.
pub(super) fn diagnose_enoki_vfx_load(
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

pub(super) fn spawn_detonate_particles(
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
pub(super) fn advance_enoki_projectiles(
    mut commands: Commands,
    pending_ticks: Res<super::clock::PendingAnimationTicks>,
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

pub(super) fn mouth_anchor_xy(caster_xy: [f32; 2], flip_x: bool, sprite_scale: f32) -> [f32; 2] {
    let dir = if flip_x { -1.0 } else { 1.0 };
    [
        caster_xy[0] + ((VFX_MOUTH_OFFSET_X_PX * sprite_scale) * dir),
        caster_xy[1] + (VFX_MOUTH_OFFSET_Y_PX * sprite_scale),
    ]
}

/// World-space base point a resolved placement offset is applied relative to.
/// `caster_xy` is the caster's live body center; the mouth anchor derives the
/// muzzle from it using the sprite facing + scale.
pub(super) fn anchor_base_xy(
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
pub(super) fn spawn_effect_by_id(
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
    // Per-effect material: the entry's own override (e.g. a flipbook flame sheet for
    // a "defined flame" layer) takes precedence; otherwise the shared soft-blob
    // material. Both are built at Startup; if NEITHER is present (asset/resource
    // ordering) we skip rather than fall back to enoki's flat-square
    // `ColorParticle2dMaterial` default — a missed spawn is surfaced by the caller's
    // spawn-miss diagnostic, a square is a silent regression.
    let Some(material) = entry.material_override.as_ref().or(soft_material) else {
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
        ParticleSpawner(material.clone()),
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

pub(super) fn should_spawn_node_vfx(
    mode: &super::DigimonPlaybackMode,
    active_barrier: Option<&bevyrogue::combat::runtime::CueBarrierStatus>,
    unit_id: UnitId,
) -> bool {
    use super::DigimonPlaybackMode;
    matches!(mode, DigimonPlaybackMode::Skill { .. })
        && active_barrier
            .map(|status| super::playback::barrier_targets_sprite(status, unit_id))
            .unwrap_or(true)
}
