---
estimated_steps: 10
estimated_files: 3
skills_used: []
---

# T03: Rewrite the render dispatcher to drive all effects from data and delete VfxParticleKind + all per-kind fns

Why: This is the forcing function for success criterion 2 — the generic dispatcher replaces kind resolution for ALL five Baby Flame effects, and removing the enum removes everything keyed off it. After this task no hardcoded VFX-kind path may remain in render.rs. Depends on T01 (axis+verbs) and T02 (schema+asset+validation).

Do (in src/windowed/render.rs):
1. Re-shape the VfxParticle component (:37-47): drop `kind: VfxParticleKind`; add `effect_id: EffectId` (resolved at spawn) and `anchor: PlacementAnchor` (from the resolved effect's placement). Keep ttl_ticks/age_ticks/motion/phase.
2. Delete: the VfxParticleKind enum (:49-61); vfx_particle_kind (:863-872); vfx_particle_ttl (:874-884); vfx_particle_size (:886-895); vfx_particle_anchor (:897-907) and the VfxAnchor enum (replaced by PlacementAnchor from the schema); baby_flame_ember_offset/alpha + baby_flame_shard_offset/alpha (:948-991, math now lives in pure verbs + curves); spawn_baby_flame_embers (:994-1019, replaced by a data-driven ember spawn reading count from spawn_plan(ember effect)); the per-kind color match in vfx_particle_color (:1067-1085); the per-kind texture match in vfx_particle_image (:1087-1102) — replace with a texture-key->Handle resolution off Appearance.texture against VfxVisuals (a small string match or map keyed by "baby_flame_charge"/"baby_flame_projectile"/"baby_flame_impact" is allowed; it is NOT VfxParticleKind dispatch); the per-kind motion/appearance match in the update loop (:1341-1368); the kind-keyed shard branch + its None fallback (:1388-1423 — keep only the generalized data path, delete the None arm); the hardcoded projectile->impact burst (:1428-1436, now via on_expire); the kind-keyed charge/ember despawn-on-release special-case (:750-761 must despawn by effect_id membership, e.g. effect_id in {baby_flame.charge, baby_flame.ember}, not by VfxParticleKind).
3. Generalize effect ids: replace the single AGUMON_IMPACT_EFFECT_ID (:338) with consts for all five (charge, ember, projectile, impact, impact_flash). At each windowed spawn site (charge node-vfx :707, ember spawn :708, projectile-on-release :768, on_expire chain), carry the correct effect_id onto VfxParticle. Do NOT touch the D031/D032 authored-timeline release seam (barrier.request_release/fire_kernel_cue/VfxSpawnDescriptor around :735-818) — only the kind->effect_id mapping at the spawn boundary changes.
4. Make advance_vfx_particles (:1270) take `regs: Res<ExtRegistries>`. Per particle per tick: resolve_effect(asset, &particle.effect_id.0); if None, warn once + skip/despawn gracefully (no panic). Compute progress from age/ttl, build PlacementCtx (caster_xy from the anchored source / target.world_xy), resolve the verb via regs.placements.get(&effect.placement.verb); if None, warn once + skip. Apply the verb's [f32;2] offset relative to the PlacementAnchor (Mouth=mouth_anchor_xy, CasterCenter=source xy, TargetCenter=target.world_xy). Drive sprite size from Appearance.size_px, transform.scale and/or distance from eval_scale, and sprite.color from eval_color(progress). For fan_out shards keep S01's behavior (outward distance = spread_px * eval_scale(progress)). On ttl==0, if effect.on_expire is Some, spawn that effect at the current position (replacing the hardcoded burst).
5. Delete/replace the now-obsolete inline #[cfg(test)] tests in render.rs (around :1760-1863) that reference the deleted fns (vfx_particle_kind, baby_flame_ember_offset/alpha, baby_flame_shard_offset/alpha) so the windowed test build stays green.
6. Extend tests/windowed_only/vfx_asset_impact_render.rs (or add a sibling registered in tests/windowed_only.rs) to pin the generalized dispatcher's lib contract: a built ExtRegistries resolves all four placement verb ids; the loaded vfx.ron yields all five effects; PlacementCtx fed to converge_inward/arc_launch yields the expected anchored offsets; on_expire of projectile resolves to impact.

Failure modes (Q5): asset not loaded/failed -> resolve_effect None -> warn-once + particle skipped (no fallback path exists by design); unresolvable verb id -> regs.placements.get None -> warn-once + skip; both are non-panicking. Negative test (Q7): a particle carrying an effect_id absent from the asset must not panic the update loop.

Done when: cargo build --features windowed is clean; cargo test --features windowed --test windowed_only passes; and the headless cargo build stays clean (verbs/schema remain lib-side — R016).

## Inputs

- `src/windowed/render.rs`
- `src/animation/vfx_asset.rs`
- `src/animation/placement.rs`
- `src/combat/runtime/registry.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only.rs`

## Expected Output

- `src/windowed/render.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only

## Observability Impact

advance_vfx_particles now warns once on target "windowed.agumon_playback" for (a) missing effect id, (b) unresolvable placement verb id, naming the offending id; both then skip the particle without panic. The S01 None-fallback path is removed (no hardcoded path may remain).
