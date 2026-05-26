---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Register all six effect ids in the enoki handle map and carry per-id anchor

Why: only sharp_claws.slash, baby_flame.impact, baby_burner.detonate are in AgumonEnokiVfx today (render.rs ~571-585). To make enoki the sole renderer, charge/ember/projectile must be registered too. Additionally, the enoki spawn branch currently sources its placement anchor from resolve_effect(asset, id).placement.anchor — i.e. from the windowed vfx.ron VfxAsset (AgumonVfx). To let T04 delete that windowed loader cleanly, migrate the anchor into the enoki resource now. Do: (1) extend the AgumonEnokiVfx map value to carry both the Handle<Particle2dEffect> and the placement Anchor (e.g. a small struct { handle, anchor } or a parallel anchor lookup mirroring enoki_effect_path); (2) in load_agumon_enoki_vfx, insert all six ids — add baby_flame.charge (anchor Mouth), baby_flame.ember (anchor Mouth), baby_flame.projectile (anchor CasterCenter) with new const asset paths, plus the existing three (impact/detonate→TargetCenter, slash→TargetCenter) — anchors taken from vfx.ron; (3) add the new ids to enoki_effect_path so the diagnostics WARN reports their source paths; (4) rewrite the enoki branch in spawn_effect_by_id (render.rs ~1481-1491) to compute `base` via anchor_base_xy from the map's anchor instead of from resolve_effect, so the enoki path no longer depends on VfxAsset. Keep the quad fallback loop unchanged for now (it stays for any unmapped id and is still resolve_effect-driven) — this task is additive. Done-when: cargo build --features windowed is green and the source-contract test asserting the map is keyed by the contact-burst ids still passes (it will be broadened in T05 to all six).

## Inputs

- `src/windowed/render.rs`
- `assets/digimon/agumon/baby_flame_charge.particle.ron`
- `assets/digimon/agumon/baby_flame_ember.particle.ron`
- `assets/digimon/agumon/baby_flame_projectile.particle.ron`
- `assets/digimon/agumon/vfx.ron`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
