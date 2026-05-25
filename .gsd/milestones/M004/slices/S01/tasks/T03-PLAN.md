---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Author assets/digimon/agumon/vfx.ron (Baby Flame impact fan-out) + headless load test

Why: the slice's headless demo is loading the real authored asset into VfxAsset and evaluating its curves deterministically — this proves content-as-data, not just the in-memory schema. Do: author assets/digimon/agumon/vfx.ron as a `VfxAsset` map containing (at minimum) the Baby Flame impact fan-out effect (e.g. id "baby_flame_impact") with an Appearance whose `count`/`spread_px`/`ttl_ticks` reproduce today's hardcoded impact-shard burst (see src/windowed/render.rs constants BABY_FLAME_IMPACT_SHARD_COUNT/SPREAD_PX/TTL and spawn_baby_flame_impact_burst) and whose `scale`/`color` keyframe curves reproduce the current ease-out/fade behavior (baby_flame_shard_offset eased spread + baby_flame_shard_alpha linear fade + the impact color). Include the central impact flash as either the same effect's first keyframe state or a sibling effect, and reference chaining via on_expire only if needed (keep minimal for the tracer bullet). Write tests/animation/vfx_asset_load.rs and register it in tests/animation.rs: load the file with `ron::from_str::<VfxAsset>(include_str!("../../assets/digimon/agumon/vfx.ron"))` (compile-time include, matching tests/animation/anim_validation.rs; do NOT read any .gsd/.planning/.audits path), assert the impact effect is present, and assert eval_scale/eval_color at sampled progresses (0.0, ~0.5, 1.0) return the expected deterministic values. Done when: vfx.ron parses into VfxAsset and the load test's eval assertions pass. Requirement impact (Q4): re-verify R012 — the new presentation asset carries numeric appearance values but is a SEPARATE asset from the SpawnParticle command, so the command's serialized form still carries no numeric gameplay payload; do not add numeric appearance values into Command/SpawnParticle.

## Inputs

- `src/animation/vfx_asset.rs`
- `src/windowed/render.rs`
- `tests/animation.rs`

## Expected Output

- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation vfx_asset_load
