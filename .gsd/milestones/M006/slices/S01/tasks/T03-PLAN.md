---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Add enoki lifecycle layer: charge/ember launch-despawn marker + projectile flight→impact chain

Why: D040 deferred charge/ember/projectile from enoki because they need stateful per-tick emitter behavior — charge/ember must emit at the mouth then clear the instant the flame launches, and the projectile must travel caster→target then chain the impact burst. With the quad path being deleted (T04), those behaviors must exist enoki-native. This is the highest-risk task (per S01 research First Proof) and implements D046. Do: in src/windowed/render.rs, (1) add a ChargeEmberEnokiMarker component; in spawn_effect_by_id, for baby_flame.charge / baby_flame.ember spawn the enoki spawner as a persistent emitter (NOT OneShot::Despawn) tagged with ChargeEmberEnokiMarker + the source UnitId; (2) add a ProjectileFlight component { from_xy, to_xy, ticks_total, ticks_elapsed } and for baby_flame.projectile spawn the enoki emitter tagged with ProjectileFlight (NOT OneShot::Despawn); (3) keep the three contact bursts (impact/detonate/slash) on OneShot::Despawn; (4) replace the old VfxParticle effect-id despawn at ~1083-1092 (inside advance_agumon_presentation, fired at CueReleaseResult::Released for BABY_FLAME) with a query that despawns ChargeEmberEnokiMarker entities for the casting unit; (5) add an advance_enoki_projectiles system on the PendingAnimationTicks clock (registered in RenderPlugin::build in the slot the advance_vfx_particles chain occupies, before advance_agumon_presentation) that each tick lerps the ProjectileFlight entity's Transform from_xy→to_xy and, on arrival (ticks_elapsed>=ticks_total), despawns the projectile spawner and calls spawn_effect_by_id for baby_flame.impact at to_xy — reproducing the old on_expire projectile→impact chain. Emit trace! on target windowed.agumon_playback at charge/ember despawn and at projectile arrival. These systems are presentation-only, fire-and-forget, and never feed the kernel/FSM timeline (D031/D032). Done-when: cargo build --features windowed green and a source-contract test (added/extended in T05) pins ChargeEmberEnokiMarker, ProjectileFlight, and advance_enoki_projectiles. Note: the projectile launch site at ~1100 already calls spawn_effect_by_id(AGUMON_PROJECTILE_EFFECT_ID); after T02 that routes through enoki, so this task only needs to attach the flight component inside spawn_effect_by_id and add the advancing system.

## Inputs

- `src/windowed/render.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
