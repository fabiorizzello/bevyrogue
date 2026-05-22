---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T03: Spawn world particles on the Baby Burner detonate signal (windowed), keeping the egui chip

Why: Baby Burner's 'detonate flash' is today only an egui chip (BabyBurnerFlashState, src/ui/combat_panel/mod.rs:128) driven by the OnKernelTransition::Blueprint detonate signal — never a world particle. The slice phrase 'SpawnParticle/detonate seam' names both sources; this funnels the detonate signal through the same renderable particle infra built in T02 so the ultimate's flash appears as pixels on both actors, while the chip stays as the UI affordance (boundary map is explicit on keeping it).

Do: In src/windowed/render.rs add a new windowed system `spawn_detonate_particles(mut commands: Commands, mut events: MessageReader<CombatEvent>, sprites: Query<(&AgumonSprite, &Transform)>)` that calls bevyrogue::ui::combat_panel::latest_baby_burner_flash_trigger(events.read()) (already pub) to get the trigger; for each target UnitId in trigger.targets, look up that unit's sprite Transform (and the source's Transform as caster) and call the T02 spawn_vfx_particle helper with a VfxSpawnDescriptor synthesized for the detonate (origin: VfxLocus::TargetCenter, motion: VfxMotion::Static, particle id e.g. ParticleId("baby_burner_detonate")) — or, if cleaner, build the descriptor from a Command::SpawnParticle constructed in-system. Do NOT remove or alter observe_baby_burner_flash / the chip — this system runs alongside it. Register the system in RenderPlugin (render.rs:210), ordered after resolve_action_system so the detonate event exists. Each system gets its own MessageReader cursor so reading CombatEvent here does not starve observe_baby_burner_flash. trace! each detonate particle spawn. Add a #[cfg(test)] unit test asserting the synthesized detonate descriptor is renderable, carries VfxLocus::TargetCenter, and (no-numeric-payload) its serialized Command form has no ascii digit when the particle id is non-numeric.

Done when: cargo build --features windowed clean; cargo test --features windowed green (new unit test + all T02 tests still pass); chip code path untouched (grep confirms observe_baby_burner_flash and BabyBurnerFlashState unchanged).

## Inputs

- `src/windowed/render.rs`
- `src/animation/vfx.rs`
- `src/ui/combat_panel/mod.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo test --features windowed --bin bevyrogue

## Observability Impact

trace!(target: "windowed.agumon_playback") on each detonate-driven particle spawn (target unit, resolved xy).
