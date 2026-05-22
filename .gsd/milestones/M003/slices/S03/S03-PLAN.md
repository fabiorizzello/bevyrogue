# S03: VFX flash renders as visible particles

**Goal:** Make the already-defined-but-inert VFX seam render as visible particles. Build a Bevy-free VfxSpawnDescriptor + resolve_locus helper in the lib (mirroring S01's AtlasGeometry), prove headless that a Command::SpawnParticle yields a renderable spawn descriptor with VfxLocus/VfxMotion honored and no numeric gameplay payload (vfx_handle_seam parity preserved), then wire two windowed spawn sources — the Baby Flame cast's on_enter SpawnParticle and the Baby Burner detonate signal — into short-lived colored Sprite-quad particle entities so the flash appears on screen during skill and ultimate, with the egui chip kept alongside.
**Demo:** Headless: a structural test asserts the SpawnParticle/detonate seam yields a renderable particle spawn (entity with visual components, VfxLocus/VfxMotion honored) rather than only an opaque ParticleId, with no numeric gameplay payload in the serialized form (vfx_handle_seam parity preserved). Visual (user-run cargo winx, K001): the VFX flash appears as visible particles during skill and ultimate on both actors.

## Must-Haves

- Headless (the only CI gate): a structural test asserts SpawnParticle -> VfxSpawnDescriptor with visual-component (renderable) intent, VfxLocus/VfxMotion preserved, and no numeric gameplay payload; resolve_locus maps the three real VfxLocus variants to caster/target coordinates. `cargo test --test animation` green (new test + all 6 atlas_binding + 4 vfx_handle_seam tests still pass); full `cargo test` green (no windowed dep leak, R002/R005); `cargo build --features windowed` clean; `cargo test --features windowed` green (pure windowed helper unit tests). Visual (user manual, K001, not run by auto-mode): VFX particles appear during Baby Flame (skill) and Baby Burner (ultimate/detonate) on both actors at the correct locus and respect motion; chip still shows.

## Proof Level

- This slice proves: contract (headless structural test on the lib seam is the CI gate) + integration wiring (windowed entity spawn). Real windowed runtime / pixels are a user-manual K001 gate, not auto-mode.

## Integration Closure

Upstream consumed: Command::SpawnParticle/ParticleId/VfxLocus/VfxMotion (src/animation/anim_graph.rs), AnimNode.on_enter, AgumonSprite+Transform and advance_agumon_presentation (src/windowed/render.rs:357), barrier_targets_sprite caster gate (render.rs:696), latest_baby_burner_flash_trigger (src/ui/combat_panel/mod.rs:164). New wiring: pub use vfx::* in src/animation/mod.rs; a VfxParticle ttl/motion despawn system and a detonate->particles system added to RenderPlugin (render.rs:210). After this slice all five M003 surfaces render (idle/basic/skill/ultimate from S01-S02 + VFX flash here); milestone is end-to-end visually complete pending the user's cargo winx confirmation.

## Verification

- trace! on each particle spawn (particle id, resolved locus xy, motion, ttl, source unit) and on despawn; debug! if a SpawnParticle references an unresolvable target. No new failure-visibility surface beyond the existing windowed.agumon_playback trace target.

## Tasks

- [ ] **T01: Add Bevy-free VfxSpawnDescriptor + resolve_locus and its headless structural test** `est:1h`
  Why: the VFX seam (Command::SpawnParticle) is fully defined but has no renderable counterpart and nothing in src/ consumes it; the lib needs a pure, Bevy-free descriptor so tests/ (lib-only link) can prove the seam yields a renderable spawn rather than an opaque ParticleId — this is the slice's only CI-provable deliverable and de-risks the windowed split, exactly as S01's AtlasGeometry did.
  - Files: `src/animation/vfx.rs`, `src/animation/mod.rs`, `tests/animation/vfx_spawn_descriptor.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation

- [ ] **T02: Consume on_enter SpawnParticle into short-lived colored Sprite-quad particle entities (windowed)** `est:2h`
  Why: advance_agumon_presentation reads node `cues` for ReleaseKernel but never reads node `on_enter` commands, so the lone authored SpawnParticle on baby_flame_cast (assets/digimon/agumon/anim_graph.ron:8) is a no-op. This task makes the Baby Flame cast emit a visible particle, establishing the reusable windowed particle entity + ttl/motion despawn infra that T03 also uses.
  - Files: `src/windowed/render.rs`
  - Verify: cargo test --features windowed --bin bevyrogue

- [ ] **T03: Spawn world particles on the Baby Burner detonate signal (windowed), keeping the egui chip** `est:1.5h`
  Why: Baby Burner's 'detonate flash' is today only an egui chip (BabyBurnerFlashState, src/ui/combat_panel/mod.rs:128) driven by the OnKernelTransition::Blueprint detonate signal — never a world particle. The slice phrase 'SpawnParticle/detonate seam' names both sources; this funnels the detonate signal through the same renderable particle infra built in T02 so the ultimate's flash appears as pixels on both actors, while the chip stays as the UI affordance (boundary map is explicit on keeping it).
  - Files: `src/windowed/render.rs`
  - Verify: cargo test --features windowed --bin bevyrogue

## Files Likely Touched

- src/animation/vfx.rs
- src/animation/mod.rs
- tests/animation/vfx_spawn_descriptor.rs
- tests/animation.rs
- src/windowed/render.rs
