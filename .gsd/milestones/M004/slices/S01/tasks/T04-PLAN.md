---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T04: Render Baby Flame impact fan-out from the vfx.ron data path (windowed glue)

Why: the slice demo requires the Baby Flame impact fan-out to render from the data path in cargo winx, proving the owned schema reaches pixels — the first hardcoded effect ported to data. Do: windowed-side in src/windowed/render.rs, register `RonAssetPlugin::<VfxAsset>` (bevy_common_assets, mirroring the AnimGraph loader) and load digimon/agumon/vfx.ron to a handle held in a resource (windowed-gated). In spawn_baby_flame_impact_burst (and the impact-shard branch of the per-tick particle update), source count/spread_px/ttl from `spawn_plan(resolve_effect(asset, "baby_flame_impact"))` and source per-tick shard scale/alpha/color from `eval_scale`/`eval_color` on the effect's curves at the shard's life progress, REPLACING the hardcoded BABY_FLAME_IMPACT_SHARD_* constants and baby_flame_shard_offset/alpha math for THIS effect only. Leave VfxParticleKind, vfx_particle_kind, and all other (charge/ember/projectile/burner) kinds untouched — S02 removes the enum; this is a surgical port. If the asset is missing or the effect id absent, warn! on target "windowed.agumon_playback" with the effect id and reason and fall back to the existing hardcoded impact path (observability/failure visibility). Add tests/windowed_only/vfx_asset_impact_render.rs and register it in tests/windowed_only.rs (feature-gated like the sibling windowed_only tests): build a VfxAsset from the authored RON and assert the resolver produces the impact spawn plan (count/spread/ttl) and curve evaluations the render path consumes, exercising the integration under the windowed feature without opening a window. Done when: `cargo build --features windowed` compiles, the new windowed_only test passes, and the full headless `cargo test` plus `cargo build` stay green (R016: no windowed dep leaks into headless; the resolver/eval used by render.rs lives in the headless lib). Visual confirmation of the fan-out via `cargo winx` is human UAT (K001 — auto-mode cannot open a window). Failure modes (Q5): missing/malformed vfx.ron -> warn + fall back to hardcoded impact path (no panic, VFX still renders). Requirement impact (Q4): re-verify R012 (no numeric gameplay payload in serialized command) and R016 (headless/windowed boundary) stay green.

## Inputs

- `assets/digimon/agumon/vfx.ron`
- `src/animation/vfx_asset.rs`
- `src/windowed/render.rs`
- `tests/windowed_only.rs`

## Expected Output

- `src/windowed/render.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only vfx_asset_impact
