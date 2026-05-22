---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T02: Consume on_enter SpawnParticle into short-lived colored Sprite-quad particle entities (windowed)

Why: advance_agumon_presentation reads node `cues` for ReleaseKernel but never reads node `on_enter` commands, so the lone authored SpawnParticle on baby_flame_cast (assets/digimon/agumon/anim_graph.ron:8) is a no-op. This task makes the Baby Flame cast emit a visible particle, establishing the reusable windowed particle entity + ttl/motion despawn infra that T03 also uses.

Do: In src/windowed/render.rs add `#[derive(Component)] struct VfxParticle { ttl_ticks: u32, motion: VfxMotion }` and a `const VFX_PARTICLE_TTL_TICKS: u32` (e.g. 6 anim ticks) and a small quad size const. Add a pure helper `fn entered_node(prev_node: &str, current_node: &str) -> Option<&str>` (returns Some(current) when changed, mirroring the already_released_frame dedup discipline) so on_enter fires exactly once on entry. Add a spawn helper `fn spawn_vfx_particle(commands: &mut Commands, descriptor: &VfxSpawnDescriptor, caster_xy: [f32;2], target_xy: [f32;2])` that calls bevyrogue::animation::resolve_locus, then `commands.spawn((Sprite::from_color(<color>, Vec2::splat(size)), Transform::from_xyz(pos.x, pos.y, 1.0), VfxParticle{ ttl_ticks: VFX_PARTICLE_TTL_TICKS, motion: descriptor.motion.clone() }))` — NO new asset, NO collider/physics (CAP-7c065a44). In advance_agumon_presentation: snapshot the node before advance (render.rs already clones current_node at :411), and after advance, when the node changed AND the sprite is the caster (reuse barrier_targets_sprite via the active_barrier, falling back to spawning only for the casting sprite), iterate the entered node's `on_enter`, build VfxSpawnDescriptor::from_command for each, resolve caster_xy from this sprite's Transform and target_xy from the opposing sprite's Transform, and spawn. (You will need to extend the sprites query to read &Transform, or add a second query for positions; resolve target as the nearest non-caster AgumonSprite Transform.) Register a new `advance_vfx_particles` system in RenderPlugin (render.rs:210) gated on the animation clock or running each Update: decrement ttl_ticks on each animation tick, apply presentation-only motion lerp toward target for FollowTarget/ArcToTarget (Static holds), and despawn when ttl hits 0 — Time/AnimationClock-driven only, no RNG/wall-clock (R004). trace! each spawn (particle id, resolved xy, motion, ttl, source unit) and each despawn. Add #[cfg(test)] unit tests in render.rs for the pure helpers: entered_node returns None when unchanged and Some on change; ttl decrement reaches 0 after VFX_PARTICLE_TTL_TICKS; resolve_locus is exercised for each variant (or a thin wrapper).

Done when: cargo build --features windowed is clean and cargo test --features windowed is green including the new render.rs unit tests; the on_enter scan is caster-gated so the dummy does not double-spawn.

## Inputs

- `src/windowed/render.rs`
- `src/animation/vfx.rs`
- `assets/digimon/agumon/anim_graph.ron`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo test --features windowed --bin bevyrogue

## Observability Impact

trace!(target: "windowed.agumon_playback") on each particle spawn (particle id, resolved locus xy, motion, ttl, source unit) and on despawn; debug! when on_enter target cannot be resolved.
