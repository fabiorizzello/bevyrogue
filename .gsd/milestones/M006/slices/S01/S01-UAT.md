# S01: Enoki as sole VFX renderer — UAT

**Milestone:** M006
**Written:** 2026-05-26T11:00:17.581Z

# UAT: S01 — Enoki as sole VFX renderer

## UAT Type
Manual visual + automated source-contract (visual portion is K001-manual, deferred)

## Preconditions
- `cargo build --features windowed` is green (verified ✓)
- `cargo test --features windowed --test windowed_only` passes 54 tests (verified ✓)
- `cargo test --test dependency_gating` passes 2 tests (verified ✓)
- Full headless `cargo test` is green (verified ✓)

## Automated Steps (all verified in this session)

1. **Build gate**: Run `cargo build --features windowed`. Expected: exit 0, Finished, zero warnings.
2. **Windowed source-contract suite**: Run `cargo test --features windowed --test windowed_only`. Expected: 54 passed, 0 failed.
   - `quad_vfx_system_is_fully_deleted_from_render_src` passes (fn/component/loop tokens absent from render.rs).
   - `enoki_handle_map_is_keyed_by_all_six_agumon_ids` passes (charge/ember/projectile/impact/detonate/slash all inserted).
   - `enoki_lifecycle_layer_is_present` passes (ChargeEmberEnokiMarker, ProjectileFlight, advance_enoki_projectiles present).
   - `kernel_and_fsm_control_flow_remains_untouched` passes (fire_kernel_cue/request_release untouched, D031/D032).
3. **Dep-gating**: Run `cargo test --test dependency_gating`. Expected: 2 passed — bevy_enoki absent from headless graph, present in windowed graph.
4. **Headless suite**: Run `cargo test`. Expected: exit 0 (lib VfxAsset/resolve_effect data-contract tests unaffected).
5. **Quad-deletion grep**: `grep -n "fn advance_vfx_particles\|VfxParticle {\|for i in 0..count" src/windowed/render.rs`. Expected: no output.

## Manual Steps (K001 — deferred, not proven by this UAT)

6. Run `cargo run --features windowed`. Select a match with Agumon.
   - **Baby Flame**: observe charge orb building at the mouth during windup → ember swirl appears → charge/ember clear the instant the flame launches → projectile travels caster→target with a flame trail → impact burst fires on arrival. No residual emitters after the skill completes.
   - **Sharp Claws**: slash burst appears at the target on strike frame.
   - **Baby Burner**: detonate burst appears at the target on impact frame.
   - No quad-rendered rectangles/sprites visible at any point.

## Edge Cases

- Rapid successive Baby Flame casts: old charge/ember emitters must be despawned before new ones spawn (ChargeEmberEnokiMarker query clears by casting unit_id).
- Projectile arrives with ticks_elapsed >= ticks_total: impact burst fires exactly once at to_xy with no leftover ProjectileFlight entity.

## Not Proven By This UAT

- Visual quality/aesthetics of the charge orb, ember swirl, or projectile trail — K001 manual sign-off required.
- Camera-shake, flash/blink, or cue-registry wiring — out of scope for S01 (S02/S03 work).
- Renamon or any second Digimon — out of scope (S05).
