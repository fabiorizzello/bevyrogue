//! Core playback system and FSM helpers for the windowed presentation layer.
//!
//! Owns [`advance_digimon_presentation`] (the main per-tick driver), its FSM
//! reconciliation helpers ([`sync_digimon_mode`], [`classify_same_skill_sync`]),
//! and the pure utility functions for release-frame detection, barrier targeting,
//! and VFX target resolution.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimationGraphLookupDiagnostics,
    FrameCueCommand, NodeId, ResolvedAnimGraphSource, SkillGraphRegistry,
    StanceGraphRegistry,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;
use bevyrogue::ui::cues::{CueDef, CueRegistry, flash_tint_parametric, shake_offset_parametric};
use bevyrogue::ui::hit_feedback::{FLASH_TICKS, HitFlashState, HitShakeState, SHAKE_TICKS};

use super::{
    ChargeEmberEnokiMarker, DeathExiting, DigimonPlaybackMode, DigimonSprite, EffectRegistries,
    FadeOut, PresentationAtlasRegistry, ReleaseFrameKey, SpriteRest,
    DEATH_FADE_TICKS,
};
use super::clock::PendingAnimationTicks;
use super::effects::{spawn_effect_by_id, should_spawn_node_vfx};

/// How an active barrier reconciles against the current playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SameSkillSync {
    /// Mode is a different skill (or Idle); the caller must (re)start the skill,
    /// seeding the player at the skill's FSM entry node.
    DifferentSkill,
    /// Same skill, same awaiting cue; the player keeps advancing untouched.
    Unchanged,
    /// Same skill, awaiting cue hopped within the cast; refresh `awaiting_cue_id`
    /// and clear the dedup guard, but do NOT reset the player node.
    CueChanged,
}

pub(super) fn advance_digimon_presentation(
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
    mut camera_shake: ResMut<super::CameraShakeState>,
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
    // --- Phase 1: auto-release unbridged skills ---------------------------------
    // Unbridged skills have no windowed FSM entry node and must be released
    // immediately so the combat timeline never stalls. Bridged skills release on
    // their rendered ReleaseKernel cue in the per-tick loop below.
    if tick_clock_auto_release_unbridged(&mut barrier, &effects) {
        return;
    }

    // --- Phase 2: decay hit-feedback windows ------------------------------------
    // Decay flash/shake/camera-shake once per frame on the animation-tick clock
    // (single decay source of truth, R010). Must run even when pending_ticks == 0
    // so fully-decayed windows are handled correctly; the inner guard skips work.
    tick_clock_decay_feedback(
        pending_ticks.0,
        &mut hit_flash,
        &mut hit_shake,
        &mut camera_shake,
    );

    // --- Phase 3: build team map (once per frame) --------------------------------
    // UnitId -> Team for the live roster; resolved once so VFX targeting can join
    // back to combat entities without touching sprite entities (which carry no Team).
    let team_of: HashMap<UnitId, Team> = sprites
        .p2()
        .iter()
        .map(|(unit, team)| (unit.id, *team))
        .collect();

    // --- Phase 4: per-animation-tick sprite loop --------------------------------
    // Most 60fps frames yield 0 ticks; each tick advances every sprite exactly once.
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

            // --- Step A: reconcile playback mode with barrier state --------------
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

            // --- Step B: advance player and drive atlas tile ---------------------
            let graph = sprite.graph.graph().clone();
            let advance = sprite.player.advance_result(&graph);
            let current_node = sprite.player.current_node.0.clone();
            let entered = entered_node(&prev_node, &current_node);
            let local_frame = local_frame_for(&graph, &sprite.player.current_node, advance.frame);
            advance_playback_atlas(
                &sprite.presentation_id,
                advance.frame,
                &atlases,
                &mut render_sprite,
            );

            // --- Step C: apply transient hit feedback (flash tint + shake) -------
            apply_hit_feedback(
                &sprite.unit_id,
                death_exiting,
                fade_out,
                rest,
                &cue_registry,
                &hit_flash,
                &hit_shake,
                &mut render_sprite,
                &mut transform,
            );

            // --- Step D: annotate barrier with current node/frame ----------------
            // Only the caster's sprite annotates the barrier with node/frame, so
            // an idle non-caster actor can't clobber the caster's impact state.
            if active_barrier
                .as_ref()
                .is_some_and(|status| barrier_targets_sprite(status, sprite.unit_id))
            {
                barrier.annotate_active_animation(&current_node, advance.frame as usize);
            }

            // --- Step E: trace per-tick playback state ---------------------------
            trace_playback_tick(
                &sprite,
                &current_node,
                advance.frame,
                local_frame,
                atlas_index_for(&sprite.presentation_id, advance.frame, &atlases),
                active_barrier.as_ref(),
            );

            // --- Step F: check whether this frame hits a release cue -------------
            let pending_release = compute_pending_release(
                &sprite,
                &graph,
                &current_node,
                local_frame,
            );

            // --- Step G: spawn on-enter VFX when the player enters a new node ----
            if should_spawn_node_vfx(&sprite.mode, active_barrier.as_ref(), sprite.unit_id) {
                if let Some(node_id) = entered {
                    try_spawn_node_vfx(
                        node_id,
                        &graph,
                        &sprite,
                        &render_sprite,
                        &transform,
                        &sprite_positions,
                        &team_of,
                        &effects,
                        &mut commands,
                        &mut cast_cue_spawn_miss_warned,
                    );
                }
            }

            // --- Step H: execute barrier release if this is the release frame ----
            if let Some((cue_id, lf)) = pending_release {
                // Capture mode_skill_id as an owned String to avoid a simultaneous
                // immutable borrow of sprite.mode alongside the &mut sprite below.
                let mode_skill_id: Option<String> =
                    mode_trace_fields(&sprite.mode).0.map(str::to_owned);
                execute_barrier_release(
                    cue_id,
                    lf,
                    &current_node,
                    advance.frame,
                    mode_skill_id.as_deref(),
                    &sprite_positions,
                    &team_of,
                    &effects,
                    &charge_ember_markers,
                    &transform,
                    &render_sprite,
                    &mut barrier,
                    &mut commands,
                    &mut sprite,
                );
            }

            // --- Step I: handle node exit (death fade or idle restore) -----------
            if advance.exited {
                handle_node_exit(
                    entity,
                    &mut sprite,
                    death_exiting,
                    fade_out,
                    active_barrier.as_ref(),
                    &stance_reg,
                    &graphs,
                    &mut commands,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Phase helpers (called by advance_digimon_presentation only)
// ---------------------------------------------------------------------------

/// Phase 1 — If there is an active barrier for an unbridged skill, auto-release
/// it immediately and return `true` (so the caller can early-return). Bridged
/// skills are NOT auto-released here; they release on their rendered ReleaseKernel
/// cue in the per-tick loop.
fn tick_clock_auto_release_unbridged(
    barrier: &mut SuspendedTimelineState,
    effects: &EffectRegistries,
) -> bool {
    let active_barrier = barrier.active_status().cloned();
    if let Some(status) = active_barrier.as_ref() {
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
            return true;
        }
    }
    false
}

/// Phase 2 — Decay the transient hit-feedback windows once per frame on the
/// animation-tick clock (single decay source of truth, R010). A unit still at
/// the full window was freshly armed this frame; trace it before the decay.
fn tick_clock_decay_feedback(
    pending_ticks: u32,
    hit_flash: &mut HitFlashState,
    hit_shake: &mut HitShakeState,
    camera_shake: &mut super::CameraShakeState,
) {
    if pending_ticks > 0 {
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
        hit_flash.decay_by(pending_ticks);
        hit_shake.decay_by(pending_ticks);
        // Camera shake decays on the SAME single source-of-truth clock (MEM094 —
        // no second decay site); apply_camera_shake reads the drained remaining.
        camera_shake.decay_by(pending_ticks);
    }
}

/// Step B helper — resolve atlas index for the current player frame.
fn atlas_index_for(
    presentation_id: &str,
    clip_frame: u32,
    atlases: &PresentationAtlasRegistry,
) -> Option<u32> {
    atlases
        .atlases
        .get(presentation_id)
        .and_then(|atlas| atlas.geometry.atlas_index(clip_frame))
}

/// Step B — Drive the rendered tile from the player frame via the identity
/// frame→atlas-index map. Leaves the index unchanged on an out-of-range frame.
fn advance_playback_atlas(
    presentation_id: &str,
    clip_frame: u32,
    atlases: &PresentationAtlasRegistry,
    render_sprite: &mut Sprite,
) {
    let atlas_index = atlas_index_for(presentation_id, clip_frame, atlases);
    if let (Some(index), Some(texture_atlas)) = (atlas_index, render_sprite.texture_atlas.as_mut())
    {
        texture_atlas.index = index as usize;
    }
}

/// Step C — Apply transient hit feedback: flash tint + positional shake.
///
/// Flash is sourced from the CueRegistry parametric math (D048 model a —
/// behaviour-preserving). Flash is the SOLE colour writer for DigimonSprite (the
/// parametric tint is WHITE at remaining 0, so steady state stays white) — but
/// skipped while the death fade owns the colour (D031/D032 barrier untouched).
/// Shake is an absolute offset from the captured rest position, never accumulated:
/// at remaining 0 the translation is hard-set back to rest.
#[allow(clippy::too_many_arguments)]
fn apply_hit_feedback(
    unit_id: &UnitId,
    death_exiting: Option<&DeathExiting>,
    fade_out: Option<&FadeOut>,
    rest: &SpriteRest,
    cue_registry: &CueRegistry,
    hit_flash: &HitFlashState,
    hit_shake: &HitShakeState,
    render_sprite: &mut Sprite,
    transform: &mut Transform,
) {
    if death_exiting.is_none() && fade_out.is_none() {
        if let Some(CueDef::Flash { peak, ticks }) = cue_registry.get("hit_flash") {
            let (r, g, b) = flash_tint_parametric(hit_flash.remaining(*unit_id), *ticks, *peak);
            render_sprite.color = Color::srgb(r, g, b);
        }
    }
    let z = transform.translation.z;
    let shake_remaining = hit_shake.remaining(*unit_id);
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
}

/// Step E — Emit the per-tick playback trace at `TRACE` level.
fn trace_playback_tick(
    sprite: &DigimonSprite,
    current_node: &str,
    clip_frame: u32,
    local_frame: Option<u32>,
    atlas_index: Option<u32>,
    active_barrier: Option<&CueBarrierStatus>,
) {
    let awaiting = active_barrier.is_some_and(|s| s.awaiting_release);
    let released = active_barrier.is_some_and(|s| s.released);
    let (mode_skill_id, mode_awaiting_cue_id) = mode_trace_fields(&sprite.mode);
    trace!(
        target: "windowed.digimon_playback",
        presentation_id = sprite.presentation_id.as_str(),
        mode = ?sprite.mode,
        skill_id = mode_skill_id,
        awaiting_cue_id = mode_awaiting_cue_id,
        graph_source = ?sprite.graph.source,
        node = current_node,
        clip_frame,
        local_frame,
        atlas_index,
        awaiting,
        released,
        barrier = ?active_barrier.map(barrier_trace_tuple),
        "digimon windowed playback tick"
    );
}

/// Step F — Compute whether the current frame is the barrier release frame.
///
/// Returns `Some((awaiting_cue_id, local_frame))` if a `ReleaseKernel` cue
/// fires on this frame and the frame has not already been released (dedup guard).
fn compute_pending_release(
    sprite: &DigimonSprite,
    graph: &AnimGraph,
    current_node: &str,
    local_frame: Option<u32>,
) -> Option<(String, u32)> {
    let DigimonPlaybackMode::Skill { awaiting_cue_id, .. } = &sprite.mode else {
        return None;
    };
    let lf = local_frame?;
    let node = graph.nodes.get(&sprite.player.current_node)?;
    if should_release_kernel(node, lf)
        && !already_released_frame(
            sprite.last_release_frame.as_ref(),
            awaiting_cue_id,
            current_node,
            lf,
        )
    {
        Some((awaiting_cue_id.clone(), lf))
    } else {
        None
    }
}

/// Step G — Spawn on-enter VFX for a newly-entered animation node.
///
/// Maps the authored particle name to effect id(s) via the engine-generic
/// registry (S04) and delegates to `spawn_effect_by_id`. Emits a warn-once
/// diagnostic (S06/S08 pattern) for any cue that resolves to zero spawns.
#[allow(clippy::too_many_arguments)]
fn try_spawn_node_vfx(
    node_id: &str,
    graph: &AnimGraph,
    sprite: &DigimonSprite,
    render_sprite: &Sprite,
    transform: &Transform,
    sprite_positions: &[(UnitId, Team, [f32; 2])],
    team_of: &HashMap<UnitId, Team>,
    effects: &EffectRegistries,
    commands: &mut Commands,
    cast_cue_spawn_miss_warned: &mut HashSet<String>,
) {
    let Some(node) = graph.nodes.get(&NodeId(node_id.to_string())) else {
        return;
    };
    let caster_xy = [transform.translation.x, transform.translation.y];
    let target_xy = nearest_opposing_target_xy(
        sprite_positions,
        sprite.unit_id,
        team_of.get(&sprite.unit_id).copied(),
        caster_xy,
    );

    for command in &node.on_enter {
        let Some(descriptor) = bevyrogue::animation::VfxSpawnDescriptor::from_command(command)
        else {
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

        // Map the authored particle name to the owned effect id(s) via the
        // engine-generic registry (S04, the charge command also seeds the
        // inward ember swirl). This name->effect map at the spawn boundary
        // replaces VfxParticleKind dispatch; each id is rendered via enoki (D043).
        let effect_ids = effects
            .on_enter
            .map
            .get(descriptor.particle.0.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let mut cue_spawned: u32 = 0;
        for effect_id in effect_ids {
            let spawned = spawn_effect_by_id(
                commands,
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

        // Spawn-miss diagnostic: a cast cue that resolved to no particle
        // (unmapped in OnEnterEffectRegistry, or effect ids absent from
        // EnokiVfxRegistry) would be a silent no-op. Warn once per cue id
        // (S06 warn-once pattern) so an unregistered cue is visible by name.
        if cue_spawned == 0 && cast_cue_spawn_miss_warned.insert(descriptor.particle.0.clone()) {
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

/// Step H — Execute the barrier release: request the release, despawn charge/ember
/// emitters, spawn the release projectile, arm the FSM kernel cue, and record the
/// dedup key. No-ops on a cue_id that has already been released this frame.
#[allow(clippy::too_many_arguments)]
fn execute_barrier_release(
    cue_id: String,
    lf: u32,
    current_node: &str,
    clip_frame: u32,
    mode_skill_id: Option<&str>,
    sprite_positions: &[(UnitId, Team, [f32; 2])],
    team_of: &HashMap<UnitId, Team>,
    effects: &EffectRegistries,
    charge_ember_markers: &Query<(Entity, &ChargeEmberEnokiMarker)>,
    transform: &Transform,
    render_sprite: &Sprite,
    barrier: &mut SuspendedTimelineState,
    commands: &mut Commands,
    sprite: &mut DigimonSprite,
) {
    let result = barrier.request_release(&cue_id);
    trace!(
        target: "windowed.digimon_playback",
        cue_id = cue_id.as_str(),
        node = current_node,
        clip_frame,
        local_frame = lf,
        ?result,
        "skill release frame observed"
    );
    if !matches!(
        result,
        CueReleaseResult::Released | CueReleaseResult::DuplicateRelease
    ) {
        return;
    }

    if let Some(release_effect_id) =
        mode_skill_id.and_then(|skill_id| effects.skill_release.map.get(skill_id))
    {
        // Despawn the charge orb + ember swirl enoki emitters the instant the
        // flame launches, so the mouth clears for the projectile. Membership is
        // by ChargeEmberEnokiMarker (enoki-native) — cleared generically for
        // every persistent emitter owned by this caster, regardless of which
        // skill released.
        for (marker_entity, marker) in charge_ember_markers {
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
            sprite_positions,
            sprite.unit_id,
            team_of.get(&sprite.unit_id).copied(),
            [transform.translation.x, transform.translation.y],
        ) {
            let spawned = spawn_effect_by_id(
                commands,
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

    // Arm the KernelCue-gated FSM transition. The node actually changes on the
    // next tick's advance_result; this only arms the pending cue. Skills with a
    // forward KernelCue edge advance (Baby Burner charge->launch->recovery, Sharp
    // Claws strike->recover); Baby Flame's impact node has no KernelCue edge (the
    // bounce is pure VFX, not an animation hop), so this is a no-op there and
    // impact->recover proceeds via TimeInNode.
    sprite.player.fire_kernel_cue();
    trace!(
        target: "windowed.digimon_playback",
        cue_id = cue_id.as_str(),
        node = current_node,
        "multi-barrier FSM advance fired (kernel cue armed)"
    );
    sprite.last_release_frame = Some(ReleaseFrameKey {
        cue_id,
        node: current_node.to_string(),
        local_frame: lf,
    });
}

/// Step I — Handle a node exit: seed the death fade-out for dying sprites, or
/// restore idle via the stance graph for living sprites. The AnimationClock
/// catch-up cap is enforced upstream by PendingAnimationTicks; this step only
/// acts on what `advance_result` reported as `exited`.
fn handle_node_exit(
    entity: Entity,
    sprite: &mut DigimonSprite,
    death_exiting: Option<&DeathExiting>,
    fade_out: Option<&FadeOut>,
    active_barrier: Option<&CueBarrierStatus>,
    stance_reg: &StanceGraphRegistry,
    graphs: &Assets<AnimGraph>,
    commands: &mut Commands,
) {
    if death_exiting.is_some() {
        // The death node has played out. Never idle-restore a dying sprite —
        // instead seed the fade-out so it lerps off the field and despawns
        // (advance_death_fade). Insert FadeOut once: the death node exits a
        // single time, but guard defensively against re-entry while the marker
        // is still present.
        if fade_out.is_none() {
            commands.entity(entity).insert(super::FadeOut {
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
        .resolve_snapshot(&AnimGraphId(sprite.stance_graph_id.clone().into()), graphs)
    {
        let preserve_missing = active_barrier.and_then(|status| {
            (sprite.graph.source == ResolvedAnimGraphSource::InstantFallback
                && sprite.last_missing_skill_graph_cue.as_deref() == Some(status.cue_id))
            .then(|| status.cue_id.to_string())
        });
        sprite.return_to_idle(stance_graph, preserve_missing);
        trace!(
            target: "windowed.digimon_playback",
            "digimon playback returned to idle"
        );
    }
}

pub(super) fn sync_digimon_mode(
    sprite: &mut DigimonSprite,
    active_barrier: Option<&CueBarrierStatus>,
    skill_reg: &SkillGraphRegistry,
    stance_reg: &StanceGraphRegistry,
    graphs: &Assets<AnimGraph>,
    start_node_reg: &super::registries::SkillStartNodeRegistry,
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

/// Classify an active barrier `(skill_id, cue_id)` against the current mode.
/// This is the load-bearing seam that lets multi-barrier skills advance their
/// FSM in place instead of restarting the player on every barrier hop.
pub(super) fn classify_same_skill_sync(
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
pub(super) fn barrier_targets_sprite(status: &CueBarrierStatus, unit_id: UnitId) -> bool {
    status.source == unit_id
}

/// `(skill_id, awaiting_cue_id)` for the active mode, used to enrich the
/// per-tick playback trace. `Idle` carries neither.
pub(super) fn mode_trace_fields(mode: &DigimonPlaybackMode) -> (Option<&str>, Option<&str>) {
    match mode {
        DigimonPlaybackMode::Idle => (None, None),
        DigimonPlaybackMode::Skill {
            skill_id,
            awaiting_cue_id,
            ..
        } => (Some(skill_id), Some(awaiting_cue_id)),
    }
}

pub(super) fn local_frame_for(
    graph: &AnimGraph,
    node_id: &NodeId,
    clip_frame: u32,
) -> Option<u32> {
    let node = graph.nodes.get(node_id)?;
    Some(if node.reverse {
        node.frames.end().saturating_sub(clip_frame)
    } else {
        clip_frame.saturating_sub(node.frames.start())
    })
}

pub(super) fn should_release_kernel(
    node: &bevyrogue::animation::AnimNode,
    local_frame: u32,
) -> bool {
    node.cues.iter().any(|cue| {
        cue.at == local_frame && matches!(cue.command, FrameCueCommand::ReleaseKernel(_))
    })
}

pub(super) fn already_released_frame(
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
    reg: &super::registries::SkillStartNodeRegistry,
    skill_id: &str,
) -> bool {
    !reg.map.contains_key(skill_id)
}

pub(super) fn barrier_trace_tuple(
    status: &CueBarrierStatus,
) -> (&str, &str, &str, bool, bool) {
    (
        status.skill_id.0.as_str(),
        status.beat_id,
        status.cue_id,
        status.awaiting_release,
        status.released,
    )
}

pub(super) fn entered_node<'a>(prev_node: &'a str, current_node: &'a str) -> Option<&'a str> {
    (prev_node != current_node).then_some(current_node)
}

/// Vertical rest offset for `slot` (0-based) of a `count`-member team column,
/// centered on y=0 so a team fans out symmetrically (e.g. 2 members → ±75).
pub(super) fn slot_offset_y(slot: usize, count: usize) -> f32 {
    use super::SLOT_VERTICAL_SPACING;
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
pub(super) fn nearest_opposing_target_xy(
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

pub(super) fn find_sprite_xy(
    sprites: &Query<(&DigimonSprite, &Transform)>,
    unit_id: UnitId,
) -> Option<[f32; 2]> {
    sprites.iter().find_map(|(sprite, transform)| {
        (sprite.unit_id == unit_id).then_some([transform.translation.x, transform.translation.y])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevyrogue::animation::{AnimNode, FrameCue, FrameRange, ReleaseKernelCue};
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
