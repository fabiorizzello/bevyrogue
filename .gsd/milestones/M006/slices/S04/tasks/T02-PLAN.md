---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Convert all enoki effect-id data + on-enter/release/arrival/detonate spawns to registries

Why: the enoki path consts, effect-id consts, the closed on_enter_effect_ids match, the spawn_effect_by_id lifecycle match, and the three imperative engine spawn sites all reference the same AGUMON_*_EFFECT_ID / AGUMON_ENOKI_*_PATH / AGUMON_PROJECTILE_FLIGHT_TICKS consts — so per D049 they must all be converted in one atomic task or the tree will not compile. Do: (1) Define engine resource EnokiVfxRegistry (rename of AgumonEnokiVfx, render.rs:621) as HashMap<String, EnokiEffect>; extend EnokiEffect (render.rs:631) with `path: String` and `lifecycle: EnokiLifecycle`; add `enum EnokiLifecycle { PersistentEmitter, Projectile { flight_ticks: u32, on_arrival: String }, OneShot }`. init_resource::<EnokiVfxRegistry>() empty in RenderPlugin::build (replacing the Startup `load_agumon_enoki_vfx` add at render.rs:407). (2) Define engine resources OnEnterEffectRegistry (HashMap<String, Vec<String>>), SkillReleaseEffectRegistry (HashMap<String, String>), DetonateEffectRegistry (e.g. resource holding Option<String>); init-empty in RenderPlugin::build. (3) Move load_agumon_enoki_vfx (render.rs:637-696) + the AGUMON_ENOKI_*_PATH consts (603-613) + AGUMON_*_EFFECT_ID consts (580-593) + AGUMON_PROJECTILE_FLIGHT_TICKS (598) into the agumon module as register Startup systems that populate EnokiVfxRegistry (each entry carries path + anchor + lifecycle: charge/ember=PersistentEmitter, projectile=Projectile{flight_ticks:5, on_arrival:"baby_flame.impact"}, impact/detonate/sharp_claws=OneShot), OnEnterEffectRegistry (baby_flame_charge->[charge,ember], baby_flame_projectile->[projectile], baby_flame_impact->[impact], sharp_claws_slash->[slash]), SkillReleaseEffectRegistry (baby_flame->baby_flame.projectile), and DetonateEffectRegistry (baby_burner.detonate). Add these systems to agumon::register(app). (4) Re-point engine: spawn_effect_by_id (render.rs:1536) matches on entry.lifecycle not effect_id; add `on_arrival: String` to ProjectileFlight and have advance_enoki_projectiles (render.rs:1798) spawn flight.on_arrival instead of AGUMON_IMPACT_EFFECT_ID; the on_enter loop (render.rs:1149) reads OnEnterEffectRegistry; the release boundary (render.rs:1189-1214) replaces `if mode_skill_id == Some(BABY_FLAME_SKILL_ID)` with `if let Some(eid) = skill_release_reg.get(skill_id)` (still clears ChargeEmberEnokiMarker generically, then spawns eid); spawn_detonate_particles (render.rs:1743) reads DetonateEffectRegistry. (5) Replace effect_path/enoki_effect_path match (render.rs:735) by reading EnokiEffect.path in diagnose_enoki_vfx_load (rename of diagnose_agumon_enoki_vfx_load). (6) Delete the on_enter_effect_ids fn (render.rs:1500) and its consts. (7) Move the on_enter_effect_ids unit test (render.rs:2104-2129) into the agumon module's #[cfg(test)] tests, asserting the registry contents instead. Keep all trace/info/warn targets as "windowed.agumon_playback". Done when: cargo build --features windowed green zero warnings; `grep -rq AGUMON_.*EFFECT_ID src/windowed/render.rs` finds nothing; no `fn on_enter_effect_ids`, no `fn load_agumon_enoki_vfx`, no `fn enoki_effect_path` in render.rs; windowed_only + dependency_gating green.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/mod.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`

## Verification

cargo test --features windowed --test windowed_only
