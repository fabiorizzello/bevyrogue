# S01 Research: Enoki as Sole VFX Renderer

## Summary

The quad VFX system lives almost entirely in `src/windowed/render.rs` (2432 lines). It consists of three components (`VfxParticle`, `VfxParticleTarget`, `VfxParticleSource`), a resource (`VfxVisuals` — 10 image handles), a per-tick system (`advance_vfx_particles`), a spawn function (`spawn_effect_by_id` — quad loop branch), and `AgumonVfx`/`diagnose_agumon_vfx_load` for loading `vfx.ron`. The headless lib contributes `VfxAsset`/`resolve_effect` from `src/animation/vfx_asset.rs` — these must survive (they are used in headless tests and the lib's public API), but the windowed consumers of them can be removed.

Three effects already route through enoki via the `AgumonEnokiVfx` handle map: `baby_flame.impact`, `sharp_claws.slash`, and `baby_burner.detonate`. The remaining five quad-only effects are `baby_flame.charge`, `baby_flame.ember`, `baby_flame.projectile`, `baby_flame.impact_flash`, and `baby_burner.flash`. For S01 to complete, all five need `.particle.ron` assets authored and registered in `load_agumon_enoki_vfx`, after which `spawn_effect_by_id`'s quad loop and `advance_vfx_particles` can be deleted, along with `VfxVisuals`/`load_vfx_visuals`/`AgumonVfx`/`diagnose_agumon_vfx_load`/`vfx_texture_handle`.

A significant complication: the charge and ember effects are despawned mid-flight (when the projectile launches — render.rs lines 1083–1091) by querying `VfxParticle` components. Once all effects go through enoki's `OneShot::Despawn`, mid-flight despawn of charge/ember becomes impossible unless enoki entities can be tracked/queried. The `on_expire` chain (`projectile -> impact -> impact_flash`) must also be reproduced; enoki's `OneShot::Despawn` is fire-and-forget, so the chain must be either built into a single combined `.particle.ron` or triggered sequentially via a new lightweight mechanism.

The dep-gating test (`tests/dependency_gating.rs`) checks that `bevy_enoki` is absent from the headless graph (`--features dev`) and present under `--features windowed`. It does not test any VFX internals, so it is unaffected by the quad system deletion as long as `bevy_enoki` stays behind `dep:bevy_enoki` (windowed feature only).

## Recommendation

**Build order:**

1. **T01 — Author missing `.particle.ron` assets** for `baby_flame.charge`, `baby_flame.ember`, `baby_flame.projectile`, and `baby_flame.impact_flash`/`baby_burner.flash` (or consolidate them). This unblocks T02. `baby_flame.impact` and the two contact bursts already exist. The charge/ember can be single-burst approximations; the projectile should be a continuous emitter (non-zero `spawn_rate`) or a single burst that mimics arc motion; impact_flash and baby_burner.flash fold naturally into their predecessor effects (`baby_flame_impact.particle.ron` already folds the flash, per its existing comments).

2. **T02 — Register all effect ids in `load_agumon_enoki_vfx`** (add 3–4 new `handles.insert(...)` calls with new const paths). This makes `spawn_effect_by_id`'s enoki branch fire for every effect id — the quad loop becomes dead code for all Agumon effects. Verify with `cargo build --features windowed` and the existing source-contract tests.

3. **T03 — Resolve the charge/ember mid-flight despawn problem.** Since enoki entities are fire-and-forget (`OneShot::Despawn`), the launch-time despawn of charge/ember quads (lines 1083–1091) cannot target enoki entities by `VfxParticle` query. Options: (a) mark enoki spawner entities with a thin `ChargeEmberMarker` component at spawn, despawn by that marker at launch; (b) consolidate charge+ember into a single timed burst that auto-expires before the projectile would normally launch (simplest, loses interruptibility); (c) keep `VfxParticle` despawn logic for a charge-specific marker. Option (a) is cleanest and preserves the existing launch-timing contract.

4. **T04 — Delete the quad system.** Remove `advance_vfx_particles`, the quad loop in `spawn_effect_by_id`, `VfxParticle`/`VfxParticleTarget`/`VfxParticleSource` components, `VfxVisuals`/`load_vfx_visuals`/`vfx_texture_handle`, `AgumonVfx`/`diagnose_agumon_vfx_load`. Remove `AgumonVfx` from all system parameters. Remove `VfxMotion`/`VfxSpawnDescriptor`/`resolve_effect`/`eval_scale`/`eval_color`/`eval_rotation`/`PlacementCtx` imports that are no longer used windowed-side (lib exports survive).

5. **T05 — Update/delete affected tests.** Several source-contract and windowed tests pin the quad system. `enoki_impact_render.rs` asserts the quad loop still exists (`for i in 0..count`). `vfx_asset_impact_render.rs` is a headless-safe data-contract test that uses `resolve_effect`/`spawn_plan` from the lib — it survives. `render_no_vfx_kind_guard.rs` pins `on_enter_effect_ids` — survives if that function stays. The `enoki_impact_render.rs` source-contract test that asserts `for i in 0..count` must be updated or replaced.

## Implementation Landscape

### Files and their purpose

**`src/windowed/render.rs`** — Primary refactor target (2432 lines).
- Lines 49–75: `VfxParticle`, `VfxParticleTarget`, `VfxParticleSource` components — DELETE.
- Lines 219–234: `VfxVisuals` resource (10 image handles for quad textures) — DELETE.
- Lines 468–481: `load_vfx_visuals` startup system — DELETE.
- Lines 503–548: `AgumonVfx` resource + `load_agumon_vfx` + `diagnose_agumon_vfx_load` — DELETE (enoki handles replace this; `vfx.ron` no longer needed windowed-side).
- Lines 551–638: `AgumonEnokiVfx` resource + `load_agumon_enoki_vfx` + `diagnose_agumon_enoki_vfx_load` + `enoki_effect_path` — EXPAND (add 3–5 new effect ids).
- Lines 374–455: `RenderPlugin::build` — remove `load_vfx_visuals`, `load_agumon_vfx`, `diagnose_agumon_vfx_load`, `advance_vfx_particles`; keep `load_agumon_enoki_vfx`, `diagnose_agumon_enoki_vfx_load`.
- Lines 1395–1403: `on_enter_effect_ids` — SURVIVES (name→effect-id boundary; the render_no_vfx_kind_guard test pins it).
- Lines 1409–1424: `vfx_texture_handle` — DELETE.
- Lines 1443–1539: `spawn_effect_by_id` — KEEP function signature, DELETE the quad loop (lines 1492–1538), keep the enoki branch (lines 1481–1491). The function reduces to: resolve effect → compute `base` → look up handle → spawn enoki one-shot → return.
- Lines 1733–1904: `advance_vfx_particles` system — DELETE entirely.
- Lines 1080–1092: Mid-flight charge/ember despawn (inside `advance_agumon_presentation`) — REPLACE with marker-query (see T03).
- Lines 787–793: `advance_agumon_presentation` system parameter list includes `vfx_visuals`, `agumon_vfx`, `vfx_assets`, `vfx_particles` query — TRIM.

**`src/animation/vfx_asset.rs`** — Headless lib, UNCHANGED. `VfxAsset`, `resolve_effect`, `spawn_plan`, `eval_scale`, `eval_color`, `eval_rotation` stay public; they are used in headless tests and the `vfx_asset_impact_render.rs` windowed data-contract test.

**`src/animation/vfx.rs`** — `VfxSpawnDescriptor`, `VfxMotion`, `resolve_locus` — UNCHANGED. Used headless-side.

**`assets/digimon/agumon/`** — Three `.particle.ron` files exist. Need to add:
- `baby_flame_charge.particle.ron` — charge orb burst at mouth anchor
- `baby_flame_ember.particle.ron` — ember swirl (or fold into charge)
- `baby_flame_projectile.particle.ron` — arc projectile (or fold into impact)
- `baby_flame_impact_flash.particle.ron` — already folded into `baby_flame_impact.particle.ron` (40 particles, comment says "folds the radiating impact_flash shards")
- `baby_burner_flash.particle.ron` — already folded into `baby_burner_detonate.particle.ron` (comment confirms)

So only **3 new assets** are strictly needed: charge, ember, projectile.

**`tests/windowed_only/enoki_impact_render.rs`** — Source-contract test (M005/S04 T03). Asserts `for i in 0..count` (quad loop still present). This test MUST be updated or deleted when the quad loop is removed.

**`tests/windowed_only/vfx_asset_impact_render.rs`** — Data-contract test using headless lib symbols (`resolve_effect`, `spawn_plan`, `eval_scale`, `eval_color`). SURVIVES unchanged.

**`tests/dependency_gating.rs`** — Dep-gating test. SURVIVES unchanged (checks dep graph only).

**`tests/animation/render_no_vfx_kind_guard.rs`** — Guards against `VfxParticleKind` re-introduction and pins `on_enter_effect_ids`. SURVIVES if `on_enter_effect_ids` stays.

### Natural seams (independent work units)

- **Asset authoring** (T01): purely additive, no code changes. Three new `.particle.ron` files. Can be done and reviewed in isolation.
- **Handle map expansion** (T02): additive changes to `load_agumon_enoki_vfx` + `enoki_effect_path`. No deletions yet.
- **Charge/ember despawn fix** (T03): self-contained change inside `advance_agumon_presentation`. Does not touch `advance_vfx_particles`.
- **Quad deletion** (T04): destructive — remove `advance_vfx_particles`, the quad branch, `VfxVisuals`, `AgumonVfx`, system registrations. Depends on T01+T02+T03 completing first.
- **Test updates** (T05): delete/update `enoki_impact_render.rs` source-contract test. Depends on T04.

## First Proof

**The highest-risk step is T03 — the charge/ember mid-flight despawn problem.**

Currently, render.rs lines 1083–1091 despawn `VfxParticle` entities matching `AGUMON_CHARGE_EFFECT_ID | AGUMON_EMBER_EFFECT_ID` at projectile launch time. Once those effects route through enoki, those entities carry no `VfxParticle` component and the query silently matches nothing — the charge orb persists through and after the projectile, breaking the visual.

Prove the seam before deleting the quad system: author `baby_flame_charge.particle.ron` and `baby_flame_ember.particle.ron`, register them in the handle map (so they now go through enoki), and confirm at runtime (K001 manual) that the charge/ember disappear correctly at launch. The recommended approach is to introduce a thin marker component (`ChargeEmberEnokiMarker`) inserted at spawn alongside `ParticleSpawner` for these two effect ids, and despawn by that marker at launch. This is a 10-line addition to `spawn_effect_by_id` (conditional on `effect_id` matching charge or ember) plus a `despawn` call in `advance_agumon_presentation`.

An alternative that sidesteps the despawn problem entirely: keep the charge/ember TTL short enough (≤ projectile launch delay, ~6 animation ticks) so they auto-expire before the projectile fires. This sacrifices the snappy launch-time clearing but avoids any new component.

## Verification

```
# Headless build must stay clean (dep-gating rule)
cargo build

# Full headless test suite
cargo test

# Dep-gating test specifically
cargo test --test dependency_gating

# Windowed build must be green
cargo build --features windowed

# Windowed tests (includes enoki parse contracts, vfx_asset_impact_render, source-contracts)
cargo test --features windowed --test windowed_only

# Source-contract test that pins the quad loop — will need updating in T05
cargo test --features windowed --test windowed_only enoki_handle_map_is_keyed_by_all_three_contact_burst_ids

# After T04: ensure no VfxParticle/VfxVisuals/AgumonVfx symbols compile
cargo build --features windowed 2>&1 | grep -E "VfxParticle|VfxVisuals|AgumonVfx|advance_vfx_particles"
```

K001 manual verification is required after T02 (all effects on enoki) and after T04 (quad system deleted). The slice goal says "VFX quality is K001 manual."

## Risks & Watch-outs

1. **Mid-flight charge/ember despawn** (highest risk). The `VfxParticle` query that clears them at launch becomes a no-op silently. Must be solved before or as part of T03.

2. **`on_expire` chain broken silently.** `projectile -> impact -> impact_flash` currently uses the quad `advance_vfx_particles` to fire the chain at TTL expiry. Once all three go through enoki `OneShot::Despawn`, the chain no longer fires. The existing `baby_flame_impact.particle.ron` (40 particles) already folds the `impact_flash` shards into the single enoki burst — this is already the correct approach. Projectile can similarly be a combined burst or a separate enoki one-shot with no explicit chain. Verify at K001 that the visual reads the same.

3. **`enoki_impact_render.rs` source-contract test asserts the quad loop.** The test at line 63 asserts `block.contains("for i in 0..count")`. Deleting the quad loop breaks this test. The test must be updated in T05 to reflect the post-deletion state (enoki-only branch).

4. **`vfx_asset_impact_render.rs` uses `resolve_effect`/`spawn_plan`** — these are lib symbols, not windowed. They survive. No risk here.

5. **`RonAssetPlugin::<VfxAsset>` registration.** After quad deletion, `vfx.ron` is never loaded windowed-side, so `AgumonVfx` and `load_agumon_vfx` are removed. However, `VfxAsset` is still a lib type used in headless tests; only the Bevy asset loader registration (which is windowed-only) goes away. The lib's `VfxAsset`/`resolve_effect` must NOT be deleted.

6. **`VfxVisuals` image handles.** `VfxVisuals` holds handles to `vfx/*.png` textures used only by the quad system's `vfx_texture_handle`. Once the quad loop is gone, these assets are never referenced. Deleting `VfxVisuals` + `load_vfx_visuals` is safe. The image files under `assets/vfx/` can be kept (no CI cost) or cleaned up separately.

7. **`VfxMotion::Static` written into every `VfxParticle` spawn.** After deletion, `VfxMotion` is only used headless-side (`VfxSpawnDescriptor`, `anim_graph.rs`). The `use bevyrogue::animation::VfxMotion` import in `render.rs` disappears; no risk to the lib.

8. **`decrement_vfx_ttl` utility.** A small pure helper used only in `advance_vfx_particles` and its unit test inside `render.rs`. Goes with the system; no external consumers.

9. **Blast radius of `advance_agumon_presentation` system parameter list.** That system currently takes `vfx_visuals`, `agumon_vfx`, `agumon_enoki_vfx`, `vfx_assets`, and a `vfx_particles` query. After deletion it drops all but `agumon_enoki_vfx` (still needed for `spawn_effect_by_id`). The parameter trim is mechanical but touches the 800-line system — careful not to disturb unrelated state.

10. **`.particle.ron` must list all 19 fields.** `Particle2dEffect` has no serde defaults (MEM098). Every new `.particle.ron` asset must enumerate all 19 fields (using `None` for unused optionals) or the RON parse fails at asset load.

## Sources

- MEM107 (architecture): retire quad VFX, enoki is sole renderer.
- MEM105 (pattern): enoki one-shot seam pattern.
- MEM104 (architecture): prior decision to keep quad as fallback (now superseded by M006/S01 D043).
- MEM098 (gotcha): all 19 `.particle.ron` fields must be explicit.
- MEM101 (pattern): source-contract tests via `include_str!` for windowed-only wiring.
- `src/windowed/render.rs` read directly (lines cited above).
- `tests/windowed_only/enoki_impact_render.rs`, `vfx_asset_impact_render.rs`, `enoki_skill_effects_parse.rs` read directly.
- `tests/dependency_gating.rs` read directly.
