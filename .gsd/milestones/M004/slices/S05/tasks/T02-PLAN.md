---
estimated_steps: 5
estimated_files: 4
skills_used: []
---

# T02: Author and trigger Sharp Claws VFX

Expected executor skills: bevy, rust-development, rust-testing, tdd, verify-before-complete.

Why: The roadmap's explicit S05 gap is that Sharp Claws has no owned Agumon VFX effect and no AnimGraph particle trigger. This task makes Sharp Claws use the same RON-owned data path as Baby Flame/Baby Burner, rather than adding new hardcoded renderer logic.

Do: Add `sharp_claws.slash` to `assets/digimon/agumon/vfx.ron` using only existing registered placement verbs, with a short TTL, target-anchored placement, a bloom-capable pale yellow/white appearance curve, and a texture key suited for a claw slash. Add or generate `assets/vfx/sharp_claws_slash.png` if a distinct texture is needed; prefer a texture containing the oriented claw marks because current particle rendering does not support per-particle rotation. Add `SpawnParticle(name: "sharp_claws_slash", ...)` to the Sharp Claws strike node in `assets/digimon/agumon/anim_graph.ron`, using the existing node-entry bridge rather than adding local-frame cue plumbing unless code proves the bridge cannot satisfy the acceptance criteria. Extend `src/windowed/render.rs` with a Sharp Claws effect id constant, `on_enter_effect_ids("sharp_claws_slash")`, texture loading/selection in `VfxVisuals`, and no new `VfxParticleKind`-style branching.

Done when: Sharp Claws has an authored effect, an AnimGraph trigger, a render bridge mapping to the effect id, and a texture path that the windowed asset loader can request.

Quality gates: Q3 has no exploitable external surface; the only input is trusted local asset data. Q4 supports M004 success criteria and D033/D034-owned data-driven VFX direction while respecting D037. Q5 malformed/missing assets should continue to use existing warn-once/fallback or skip behavior, not panic. Q6 particle count should remain small and bounded by spawn_plan. Q7 negative coverage is completed in T03 by validation/mapping tests for missing ids and unregistered verbs.

## Inputs

- `assets/digimon/agumon/vfx.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/windowed/render.rs`
- `assets/vfx/baby_flame_impact.png`
- `tests/animation/vfx_asset_load.rs`
- `tests/windowed_only/vfx_rendering_acceptance.rs`

## Expected Output

- `assets/digimon/agumon/vfx.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/windowed/render.rs`
- `assets/vfx/sharp_claws_slash.png`

## Verification

cargo check --features windowed
cargo test --test animation vfx_asset_load -- --nocapture

## Observability Impact

Keeps Sharp Claws failures on the existing VFX asset/effect-resolution warning seams; no new runtime diagnostic channel is needed if missing texture/effect behavior remains visible through current Bevy asset logs and render warnings.
