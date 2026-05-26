---
id: T03
parent: S01
milestone: M006
key_files:
  - src/windowed/render.rs
key_decisions:
  - Dispatched the enoki branch of spawn_effect_by_id by effect_id into three lifecycle classes (persistent charge/ember marker, traveling projectile flight, OneShot contact burst) rather than a flag, keeping the lifecycle policy co-located with the spawn site.
  - Made spawn_effect_by_id's asset param Option<&VfxAsset> so the enoki impact chain (and any future enoki-only caller) does not depend on the vfx.ron loader T04 deletes; the quad fallback returns 0 when asset is None.
  - Stored from_xy/to_xy/ticks on ProjectileFlight (no unit_id, per the planned struct shape) and passed a placeholder UnitId(0) to the impact spawn — source_unit is unused on the enoki OneShot path (impact carries no VfxParticleSource).
  - Set AGUMON_PROJECTILE_FLIGHT_TICKS = 5 to match the deleted quad projectile's ttl_ticks so the flight timing is identical.
duration: 
verification_result: passed
completed_at: 2026-05-26T10:46:40.070Z
blocker_discovered: false
---

# T03: Added enoki-native lifecycle layer: persistent charge/ember emitters cleared on launch, and a traveling projectile that chains the impact burst on arrival

**Added enoki-native lifecycle layer: persistent charge/ember emitters cleared on launch, and a traveling projectile that chains the impact burst on arrival**

## What Happened

Implemented D046 in src/windowed/render.rs, giving the Baby Flame buildup effects stateful per-tick behavior enoki-native (so T04 can delete the quad path).

(1) Added two components: ChargeEmberEnokiMarker { unit_id } and ProjectileFlight { from_xy, to_xy, ticks_total, ticks_elapsed }. Added const AGUMON_PROJECTILE_FLIGHT_TICKS = 5 (mirrors the deleted quad projectile's ttl_ticks in vfx.ron).

(2) Reworked the enoki branch of spawn_effect_by_id to dispatch on effect_id instead of unconditionally attaching OneShot::Despawn: charge/ember spawn as persistent emitters tagged ChargeEmberEnokiMarker + the source UnitId; the projectile spawns as a persistent emitter tagged ProjectileFlight (from_xy = computed anchor base, to_xy = target_xy); impact/detonate/slash keep OneShot::Despawn.

(3) Replaced the old VfxParticle effect-id launch-clear inside advance_agumon_presentation (the CueReleaseResult::Released branch for BABY_FLAME) with a query over ChargeEmberEnokiMarker that despawns every marker for the casting unit, emitting a trace! on windowed.agumon_playback. Swapped the system's vfx_particles query param for charge_ember_markers (the old param had no remaining users once charge/ember route through enoki).

(4) Added advance_enoki_projectiles on the PendingAnimationTicks clock, registered in the presentation chain in the slot advance_vfx_particles occupies, strictly before advance_agumon_presentation. Each tick it lerps the ProjectileFlight entity's Transform from_xy→to_xy; on arrival (ticks_elapsed >= ticks_total) it despawns the spawner and calls spawn_effect_by_id for baby_flame.impact at to_xy, reproducing the old on_expire projectile→impact chain, with a trace! on arrival.

To let the enoki impact chain run without the vfx.ron loader (which T04 deletes), changed spawn_effect_by_id's asset param to Option<&VfxAsset>: the enoki branch returns before it is consulted, and the quad fallback returns 0 when asset is None. Updated the four existing callers to pass Some(asset). These systems are presentation-only and never feed the kernel/FSM timeline (D031/D032, R010).

## Verification

Ran `cargo build --features windowed` — green (Finished dev profile, exit 0). Re-ran the build filtering for warnings — none introduced. Build is the task's declared verification command. Full cargo test and the dep-gating test are slice-level checks deferred to the final task (T05, which also adds the source-contract test pinning ChargeEmberEnokiMarker, ProjectileFlight, and advance_enoki_projectiles). VFX visual quality is K001 manual (deferred).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 3250ms |
| 2 | `cargo build --features windowed 2>&1 | grep -i warning` | 0 | pass (no warnings) | 2000ms |

## Deviations

Changed spawn_effect_by_id's asset parameter from &VfxAsset to Option<&VfxAsset> (and updated its 4 existing call sites to pass Some(asset)). Not in the literal plan text, but required so the enoki impact chain in advance_enoki_projectiles can spawn without a VfxAsset — forward-compatible with T04's loader deletion. The signature change is confined to the windowed module.

## Known Issues

VFX visual quality (does the sequence actually look right on screen) is K001 manual and unverified by auto-mode — never run the windowed binary from auto-mode. The source-contract test pinning the three new symbols is authored in T05, not this task.

## Files Created/Modified

- `src/windowed/render.rs`
