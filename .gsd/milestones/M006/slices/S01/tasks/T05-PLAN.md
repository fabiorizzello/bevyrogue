---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Update source-contract tests to the enoki-only state and prove no regression

Why: the existing source-contract test tests/windowed_only/enoki_impact_render.rs asserts `block.contains("for i in 0..count")` — i.e. it pins the quad loop's continued existence. T04 deletes that loop, so this assertion must invert. The contract should now assert the quad system is GONE and the enoki-only seam + lifecycle layer is present. Do: in enoki_impact_render.rs, (1) remove/replace the `for i in 0..count` assertion with an assertion that render.rs no longer contains the quad-loop / VfxParticle / advance_vfx_particles tokens; (2) broaden enoki_handle_map_is_keyed_by_all_three_contact_burst_ids (rename if appropriate) to assert all six ids are inserted in load_agumon_enoki_vfx (charge, ember, projectile, impact, detonate, slash); (3) add assertions pinning the T03 lifecycle layer (ChargeEmberEnokiMarker, ProjectileFlight, advance_enoki_projectiles) and that fire_kernel_cue / request_release control flow is untouched (D031/D032). Confirm vfx_asset_impact_render.rs and render_no_vfx_kind_guard.rs still pass unchanged (they use lib symbols / on_enter_effect_ids that survive). Done-when: full headless `cargo test`, `cargo test --features windowed --test windowed_only`, and `cargo test --test dependency_gating` are all green, and `cargo build` (headless) stays green proving no enoki/windowed leak.

## Inputs

- `tests/windowed_only/enoki_impact_render.rs`
- `src/windowed/render.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/dependency_gating.rs`

## Expected Output

- `tests/windowed_only/enoki_impact_render.rs`

## Verification

cargo test --features windowed --test windowed_only
