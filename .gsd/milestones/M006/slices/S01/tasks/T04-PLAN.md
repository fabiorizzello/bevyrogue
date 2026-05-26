---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Delete the custom quad VFX system from the windowed engine

Why: D043 retires the quad system entirely once enoki renders everything (achieved by T01-T03). Do: in src/windowed/render.rs delete — the advance_vfx_particles system (~1733-1904) including its decrement_vfx_ttl helper and unit test; the quad spawn loop in spawn_effect_by_id (~1492-1538), leaving only the enoki branch so the function reduces to: look up (handle, anchor) in the map → compute base → spawn enoki (with the T03 lifecycle tagging) → return; the VfxParticle/VfxParticleTarget/VfxParticleSource components (~49-75); the VfxVisuals resource (~219-234), load_vfx_visuals (~468-481), and vfx_texture_handle (~1409-1424); the windowed AgumonVfx resource + load_agumon_vfx + diagnose_agumon_vfx_load (~503-548) and the windowed RonAssetPlugin::<VfxAsset> registration / vfx.ron load; and all corresponding entries in RenderPlugin::build (load_vfx_visuals, load_agumon_vfx, diagnose_agumon_vfx_load, advance_vfx_particles in the chain). Trim system parameter lists that referenced the deleted items (advance_agumon_presentation and any other system taking vfx_visuals / agumon_vfx / the vfx_particles query / vfx_assets where now unused) and update spawn_effect_by_id's signature to drop `asset: &VfxAsset` and `visuals` plus update its remaining call sites (~1038, ~1708). KEEP: the lib src/animation/vfx_asset.rs (VfxAsset/resolve_effect/spawn_plan/eval_*) UNCHANGED — it is used by headless tests and the windowed data-contract test vfx_asset_impact_render.rs; keep on_enter_effect_ids (pinned by render_no_vfx_kind_guard); keep AgumonEnokiVfx + diagnose_agumon_enoki_vfx_load + the T03 lifecycle systems. Done-when: cargo build --features windowed green AND `cargo build --features windowed 2>&1` contains no VfxParticle/VfxVisuals/AgumonVfx/advance_vfx_particles symbols (dead-code/undefined). Verify via build (the verify command); a grep guard is added in T05.

## Inputs

- `src/windowed/render.rs`
- `src/animation/vfx_asset.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
