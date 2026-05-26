---
id: T02
parent: S01
milestone: M006
key_files:
  - src/windowed/render.rs
key_decisions:
  - Modeled the per-id enoki map value as a new EnokiEffect { handle, anchor } struct (kept the field name `handles` so the existing source-contract test's `enoki.handles.get(effect_id)`/`handles.insert(...)` assertions still pass).
  - Moved the enoki intercept in spawn_effect_by_id ABOVE resolve_effect so the enoki path computes its placement base purely from the map-carried anchor, removing all VfxAsset dependency from the enoki renderer ahead of T04's loader deletion.
  - Read each anchor from vfx.ron (charge/ember Mouth, projectile CasterCenter, slash/impact/detonate TargetCenter) to preserve placement parity rather than re-deriving anchors.
duration: 
verification_result: passed
completed_at: 2026-05-26T10:40:51.576Z
blocker_discovered: false
---

# T02: Registered all six Agumon effect ids in the enoki handle map with per-id anchors and made the enoki spawn path source placement from the map instead of VfxAsset

**Registered all six Agumon effect ids in the enoki handle map with per-id anchors and made the enoki spawn path source placement from the map instead of VfxAsset**

## What Happened

Extended the enoki renderer registration so bevy_enoki can become the sole VFX renderer (the rest of the deletion lands in T04/T05).

(1) Changed the `AgumonEnokiVfx` map value from a bare `Handle<Particle2dEffect>` to a new `EnokiEffect { handle, anchor: PlacementAnchor }` struct so each registered effect carries its placement anchor migrated out of the windowed `VfxAsset`/vfx.ron.

(2) Added three new const asset paths — `AGUMON_ENOKI_CHARGE_PATH`, `AGUMON_ENOKI_EMBER_PATH`, `AGUMON_ENOKI_PROJECTILE_PATH` (names mirrored by the existing parse tests' comments) — and inserted all six ids in `load_agumon_enoki_vfx`: charge (Mouth), ember (Mouth), projectile (CasterCenter), plus the existing slash/impact/detonate (all TargetCenter). Anchors were read directly from vfx.ron to preserve placement parity.

(3) Added the three new ids to `enoki_effect_path` so the `diagnose_agumon_enoki_vfx_load` WARN reports their source paths; updated that diagnose loop's binding from `handle` to `entry` (`entry.handle.id()`).

(4) Rewrote the enoki branch in `spawn_effect_by_id` and moved it ABOVE the `resolve_effect(asset, ...)` call. The branch now computes `base` via `anchor_base_xy(entry.anchor, ...)` from the map entry, so the enoki path no longer reads the `VfxAsset` at all — clearing the way for T04 to delete the windowed vfx.ron loader. The quad fallback loop (resolve_effect-driven, `for i in 0..count`) is unchanged and still serves any unmapped id. As an intended consequence, charge/ember/projectile now route through enoki instead of the quad path. Kernel/FSM control flow (D031/D032) untouched; `OneShot::Despawn` keeps bursts out of the kernel timeline.

The source-contract test still requires the field name `handles` and `enoki.handles.get(effect_id)` / `handles.insert(...)` syntax, so those were preserved.

## Verification

cargo build --features windowed → exit 0 (Finished dev profile). cargo test --features windowed --test windowed_only → exit 0, 52 passed / 0 failed, including enoki_impact_render (the source-contract test asserting the handle map is keyed by the contact-burst id consts and the enoki branch attaches ParticleSpawner/ParticleEffectHandle/OneShot while the quad loop remains) and enoki_skill_effects_parse (charge/ember/projectile/slash/detonate parse into Particle2dEffect).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 2816ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 2651ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
