---
id: T03
parent: S02
milestone: M004
key_files:
  - src/windowed/render.rs
  - assets/digimon/agumon/vfx.ron
  - tests/windowed_only/vfx_asset_impact_render.rs
  - tests/animation/vfx_asset_schema.rs
key_decisions:
  - advance_vfx_particles computes ABSOLUTE position each tick (anchor base + verb offset) rather than incrementally lerping, so spawn position is just a placeholder and there is no drift; this let me delete the VfxMotion-based lerp entirely.
  - FanOut is special-cased in the dispatcher: outward distance = spread_px * eval_scale(progress) along particle.phase (preserving S01 behavior + the windowed test contract), but the fan_out verb is still resolved from the Registry so an unregistered id is still caught and warned.
  - No hardcoded fallback path remains: if the asset/effect/verb can't resolve, the particle is warned-once (naming the offending id) and despawned gracefully. If the asset or ExtRegistries is entirely absent, particles just age out and despawn.
  - Baby Burner detonate (out of S02 scope) was kept rendering by authoring a 6th owned effect baby_burner.detonate routed through spawn_effect_by_id, instead of reintroducing a hardcoded non-kind path; the animation round-trip count test was updated 5->6 to match the genuinely-authored set.
  - The separate baby_flame.impact_flash central flash is not spawned by the chain (projectile.on_expire must stay = baby_flame.impact per the pinned asset test); the impact fan's bright opaque first frame covers the flash read. impact_flash stays authored + validated for future use. Documented K001 visual deviation.
duration: 
verification_result: passed
completed_at: 2026-05-25T11:08:04.137Z
blocker_discovered: false
---

# T03: Rewrote the windowed VFX dispatcher to drive all effects from the owned vfx.ron data path via Registry-resolved placement verbs, deleting VfxParticleKind and every per-kind helper.

**Rewrote the windowed VFX dispatcher to drive all effects from the owned vfx.ron data path via Registry-resolved placement verbs, deleting VfxParticleKind and every per-kind helper.**

## What Happened

Resumed a broken intermediate state: the prior session had reshaped the `VfxParticle` component and rewired the spawn call sites to `on_enter_effect_ids`/`spawn_effect_by_id`, AND deleted the `VfxParticleKind`/`VfxAnchor` enums plus all `BABY_FLAME_*`/`VFX_PARTICLE_*` constants — but left ~55 dangling references and never defined the new helpers. Because render.rs is `windowed`-gated, the prior `cargo build` (headless) never compiled it, so the breakage was hidden. (The prior `gsd_task_complete` also never persisted — no T03-SUMMARY existed.)

Completed the rewrite:
- Defined `on_enter_effect_ids` (charge SpawnParticle fans out to charge+ember effect ids; a name->effect map at the spawn boundary, NOT kind dispatch), `vfx_texture_handle` (Appearance.texture string->Handle, unknown/empty -> flat-color quad), `anchor_base_xy` (Mouth/CasterCenter/TargetCenter -> world origin), and `spawn_effect_by_id` (spawns `count` particles, reading ttl/size/texture/spawn-color from the resolved EffectDef).
- Deleted vfx_particle_kind/ttl/size/anchor, the four baby_flame_*_descriptor fns, baby_flame_ember/shard offset+alpha, spawn_baby_flame_embers, spawn_baby_flame_impact_burst, vfx_particle_color/image, spawn_vfx_particle, resolve_vfx_spawn_xy, detonate_vfx_descriptor.
- Rewrote `advance_vfx_particles` to take `Res<ExtRegistries>`: per particle/tick it `resolve_effect`s the owned id (absent -> warn-once-per-id on target windowed.agumon_playback + despawn, no panic, no fallback path), resolves the placement verb via `regs.placements.get` (unregistered -> warn-once + despawn), then sets absolute position = anchor base + verb offset, scale from eval_scale, color from eval_color. FanOut shards keep S01 behavior (distance = spread_px * eval_scale along phase) while the verb is still resolved to catch a bad id. on ttl==0, `effect.on_expire` spawns the chained effect at the current position (projectile->impact replaces the hardcoded burst).
- Routed Baby Burner detonate (S03 scope) through the same unified path by authoring a minimal `baby_burner.detonate` effect in vfx.ron, so it keeps rendering with no hardcoded kind.
- Deleted the obsolete inline render.rs tests that referenced deleted fns; added on_enter_effect_ids + anchor_base_xy unit tests. Extended tests/windowed_only/vfx_asset_impact_render.rs with four contract tests: a built ExtRegistries resolves all four verb ids, every authored effect resolves and its verb is registered, converge_inward/arc_launch yield the expected anchored offsets, and projectile.on_expire chains the impact fan.

## Verification

cargo build --features windowed clean (no warnings). cargo test --features windowed --test windowed_only: 32 passed/0 failed (incl. 4 new dispatcher-contract tests). Headless cargo build clean (R016 — verbs/schema stay lib-side). cargo test --features windowed --bin bevyrogue: 19 inline tests passed. cargo test --test animation: 100 passed (updated the round-trip count assertion 5->6 for the added detonate effect; validate_effects positive+negative still pass). Full headless cargo test: all suites green. Static grep confirms VfxParticleKind/kind_from_name/VfxAnchor appear only in comments, never in code.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 11000ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 10000ms |
| 3 | `cargo build` | 0 | pass | 200ms |
| 4 | `cargo test --features windowed --bin bevyrogue` | 0 | pass | 9000ms |
| 5 | `cargo test --test animation` | 0 | pass | 1000ms |
| 6 | `cargo test` | 0 | pass | 30000ms |

## Deviations

Plan listed expected outputs as render.rs + the windowed test + windowed_only.rs aggregator. Two unplanned edits were needed: (1) authored baby_burner.detonate in vfx.ron and (2) updated the count assertion in tests/animation/vfx_asset_schema.rs from 5 to 6, both consequences of routing detonate through the unified data path rather than dropping its rendering. windowed_only.rs needed no change (the test was added to the existing sibling file).

## Known Issues

The separate baby_flame.impact_flash central flash is authored + validated but not currently spawned by any chain (a documented K001 visual deviation; on_expire is single-valued and projectile must chain to baby_flame.impact per the pinned asset test). The Q7 negative case (a particle carrying an absent effect_id must not panic the update loop) is satisfied by design (warn-once + graceful despawn) and covered lib-side by resolve_effect returning None; it cannot be exercised directly because render.rs internals live in the binary crate. Pre-existing unrelated `unused import: BeatEdge` warning in another test file was left untouched.

## Files Created/Modified

- `src/windowed/render.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/animation/vfx_asset_schema.rs`
