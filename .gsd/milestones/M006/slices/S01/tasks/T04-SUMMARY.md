---
id: T04
parent: S01
milestone: M006
key_files:
  - src/windowed/render.rs
  - tests/windowed_only/enoki_impact_render.rs
  - tests/windowed_only/vfx_windowed_contracts.rs
key_decisions:
  - Reduced spawn_effect_by_id to enoki-only: an unmapped id (or absent AgumonEnokiVfx resource) now returns 0 instead of falling through to a quad spawn — enoki is the sole particle renderer (D043).
  - Re-pointed sample_animation_ticks.before(...) and the presentation chain head at advance_enoki_projectiles, the slot the deleted advance_vfx_particles occupied, preserving T03 chain ordering before advance_agumon_presentation.
  - Updated (rather than deleted) the two windowed_only source-contract tests that pinned the quad system, since their intent — proving enoki routing and camera HDR wiring — is still valid; only their now-false assumptions about the quad loop / load_vfx_visuals adjacency were corrected.
duration: 
verification_result: passed
completed_at: 2026-05-26T10:55:20.211Z
blocker_discovered: false
---

# T04: Deleted the custom quad VFX system from the windowed engine, leaving bevy_enoki as the sole particle renderer for every Agumon effect.

**Deleted the custom quad VFX system from the windowed engine, leaving bevy_enoki as the sole particle renderer for every Agumon effect.**

## What Happened

D043 retires the quad VFX system now that enoki renders everything (T01–T03). In src/windowed/render.rs I removed: the VfxParticle/VfxParticleTarget/VfxParticleSource components; the VfxVisuals resource + load_vfx_visuals + vfx_texture_handle; the windowed AgumonVfx resource + load_agumon_vfx + diagnose_agumon_vfx_load + AGUMON_VFX_PATH; the RonAssetPlugin::<VfxAsset> registration; the advance_vfx_particles system + its decrement_vfx_ttl helper + the decrement_vfx_ttl unit test; and the quad spawn loop inside spawn_effect_by_id.

spawn_effect_by_id now reduces to the enoki path: look up (handle, anchor) in the AgumonEnokiVfx map → compute base via anchor_base_xy → spawn the enoki ParticleSpawner with the T03 lifecycle tagging (ChargeEmberEnokiMarker for charge/ember, ProjectileFlight for the projectile, OneShot::Despawn for contact bursts) → return 1; an absent resource or unmapped id returns 0. Its signature dropped the `asset: Option<&VfxAsset>` and `visuals` params, and all four remaining call sites were updated (on_enter loop and projectile-on-release in advance_agumon_presentation, the detonate spawn in spawn_detonate_particles, and the impact chain in advance_enoki_projectiles).

Trimmed system parameter lists that referenced the deleted items: advance_agumon_presentation dropped vfx_visuals/agumon_vfx/vfx_assets and its derived vfx_asset local plus the `Some(asset)` guards; spawn_detonate_particles dropped the same three plus its asset-resolution guard. Updated RenderPlugin::build to remove load_vfx_visuals/load_agumon_vfx from Startup, the diagnose_agumon_vfx_load Update system, and advance_vfx_particles from the presentation chain, and re-pointed sample_animation_ticks.before(...) at advance_enoki_projectiles (the new chain head). Trimmed the now-unused imports (VfxAsset, VfxMotion, EffectId, PlacementCtx, PlacementParams, resolve_effect, eval_color, eval_scale, eval_rotation, ExtRegistries, RonAssetPlugin) and a stale comment.

KEPT (unchanged) per plan: the lib src/animation/vfx_asset.rs; on_enter_effect_ids (pinned by render_no_vfx_kind_guard); AgumonEnokiVfx + diagnose_agumon_enoki_vfx_load + the T03 lifecycle systems (advance_enoki_projectiles, ProjectileFlight, ChargeEmberEnokiMarker). The diagnose_agumon_enoki_vfx_load WARN survives and covers all six ids; diagnose_agumon_vfx_load (quad) is gone.

Two windowed_only source-contract tests pinned the deleted quad system and were updated as a direct consequence of the deletion (not in the plan's Inputs list): enoki_impact_render.rs (its "for i in 0..count quad loop must remain" assertion was inverted to assert the loop is gone + spawn_effect_by_id early-returns 0 for an unmapped id) and vfx_windowed_contracts.rs (its setup_camera_block slice boundary moved from the deleted load_vfx_visuals to load_agumon_enoki_vfx).

## Verification

cargo build --features windowed is green with zero warnings; scanning the build output shows no VfxParticle/VfxVisuals/AgumonVfx/advance_vfx_particles symbols (done-when satisfied). The headless render_no_vfx_kind_guard tests pass (on_enter_effect_ids + Sharp Claws boundary intact, no forbidden VFX-kind identifiers). The dependency_gating tests pass (bevy_enoki present in windowed graph, absent from headless graph). Full headless cargo test is green; full cargo test --features windowed is green (all windowed_only harnesses including the two updated source-contract tests pass).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass — green, zero warnings | 2160ms |
| 2 | `cargo build --features windowed 2>&1 | grep -E 'VfxParticle|VfxVisuals|AgumonVfx|advance_vfx_particles'` | 1 | pass — no dead/undefined symbols (none matched) | 200ms |
| 3 | `cargo test --test animation render_rs` | 0 | pass — 2 guard tests ok | 490ms |
| 4 | `cargo test --test dependency_gating` | 0 | pass — 2 ok (enoki present windowed, absent headless) | 280ms |
| 5 | `cargo test --features windowed --test windowed_only` | 0 | pass — 52 passed, 0 failed | 30ms |
| 6 | `cargo test` | 0 | pass — full headless suite green | 1000ms |
| 7 | `cargo test --features windowed` | 0 | pass — full windowed suite green, no failures | 2000ms |

## Deviations

Two test files outside the plan's Inputs list (tests/windowed_only/enoki_impact_render.rs and tests/windowed_only/vfx_windowed_contracts.rs) were edited. Both are static source-contract tests that asserted the existence of the quad system T04 deletes (the `for i in 0..count` quad loop, and a setup_camera_block slice ending at the deleted load_vfx_visuals). Leaving them would have made `cargo test --features windowed` red, violating the slice goal. Edits were minimal and intent-preserving: the quad-loop assertion was inverted to pin its absence + the new early-return-0 behavior, and the slice boundary was moved to load_agumon_enoki_vfx.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/windowed_only/vfx_windowed_contracts.rs`
