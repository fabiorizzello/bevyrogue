# S02: Placement verbs in Registry + generic render dispatcher

**Goal:** Port all five Baby Flame VFX effects (charge, ember, projectile/launch, impact, impact_shard via the existing impact + flash) to the owned data path and drive every per-tick particle position and appearance through a generic, Registry-resolved placement-verb dispatcher — deleting VfxParticleKind, kind_from_name/vfx_particle_kind, and all per-kind helper fns from src/windowed/render.rs so no hardcoded VFX-kind path remains.
**Demo:** cargo winx shows Baby Flame charge ember-swirl and fast launch rendered through Registry-resolved placement verbs; a static grep confirms VfxParticleKind and kind_from_name no longer exist in render.rs.

## Must-Haves

- (1) Baby Flame charge ember-swirl and fast launch render through Registry-resolved placement verbs (converge_inward, arc_launch) sourced from assets/digimon/agumon/vfx.ron. (2) A headless grep-guard test confirms VfxParticleKind, kind_from_name, and vfx_particle_kind no longer exist in src/windowed/render.rs. (3) All placement-verb math is pure and deterministic (R004): no World/render types in verb fns; 1000-call bit-identical determinism tests green. (4) Placement params are typed + Reflect-introspectable (D034), not a string map. (5) projectile→impact chaining is driven by the data `on_expire` field, not a hardcoded burst. (6) Load-time validation surfaces an unresolvable verb id or malformed/missing effect/on_expire id as an explicit warning (never a render-loop panic or silent mis-render).

## Proof Level

- This slice proves: Headless half is contract+integration and auto-mode-certifiable: cargo test --test animation (verb determinism, schema round-trip/Reflect, vfx.ron load + on_expire resolution, grep-guard), cargo build (headless, R016), cargo build --features windowed (dispatcher compiles), cargo test --features windowed --test windowed_only (dispatcher lib contract). Visual confirmation that charge swirl + fast launch look right is K001 — human-only UAT via cargo winx; auto-mode must NOT run it.

## Integration Closure

Upstream consumed: src/animation/vfx_asset.rs resolver/eval API (resolve_effect/spawn_plan/eval_scale/eval_color, S01), the D031 Registry<E>/ExtRegistries substrate (src/combat/runtime/registry.rs), and the S01 RonAssetPlugin::<VfxAsset> + AgumonVfx handle wiring in render.rs. New wiring: PlacementExt axis registered into ExtRegistries via register_agumon_ext (Agumon blueprint), and advance_vfx_particles takes Res<ExtRegistries> to resolve placement verbs per tick. The D031/D032 authored-timeline release seam (barrier.request_release / fire_kernel_cue / VfxSpawnDescriptor) is NOT modified — only per-tick kind resolution is replaced. After S02: charge + launch + impact all render from data with zero hardcoded VFX-kind paths; Baby Burner detonate + skill-tree variant selection remain for S03.

## Verification

- Generalizes the S01 once-only warn on target "windowed.agumon_playback": load failure stays surfaced by diagnose_agumon_vfx_load (LoadState::Failed); a loaded asset missing a required effect id, an unresolvable placement verb id, or a malformed param payload each warn once with the offending effect/verb id and a reason, then the affected particle is skipped/despawned gracefully (no panic). New: a headless load-validation pass over the asset reports the first invalid (effect, verb) pair.

## Tasks

- [x] **T01: Stand up the PlacementExt axis + pure placement verbs (lib, headless, additive)** `est:2h`
  Why: This is the First Proof from S02-RESEARCH — prove open-dispatch + typed-params end-to-end in the headless lib before any render.rs surgery, resolving the D035/D036 design tension with compiling code. Additive only: it must NOT change the existing Placement/Appearance serde shape yet (that is T02), so the build and all existing animation tests stay green.
  - Files: `src/animation/vfx_asset.rs`, `src/animation/placement.rs`, `src/animation/mod.rs`, `src/combat/runtime/registry.rs`, `src/combat/blueprints/agumon/mod.rs`, `tests/animation/placement_verbs.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation

- [x] **T02: Extend the VFX schema (typed params + anchor + size + texture), author all five effects, add load-time validation** `est:2h`
  Why: Removing VfxParticleKind in T03 removes the information the enum carried — per-kind ttl, size, anchor, texture, and the projectile→impact chain. That information must move into the typed, Reflect-able schema and the asset BEFORE the dispatcher can read it. This task is atomic: the serde-shape change to Placement/Appearance breaks the existing vfx.ron and the S01 schema/load tests simultaneously, so the asset migration and test updates land together to keep the build green.
  - Files: `src/animation/vfx_asset.rs`, `assets/digimon/agumon/vfx.ron`, `tests/animation/vfx_asset_schema.rs`, `tests/animation/vfx_asset_load.rs`
  - Verify: cargo test --test animation

- [x] **T03: Rewrite the render dispatcher to drive all effects from data and delete VfxParticleKind + all per-kind fns** `est:4h`
  Why: This is the forcing function for success criterion 2 — the generic dispatcher replaces kind resolution for ALL five Baby Flame effects, and removing the enum removes everything keyed off it. After this task no hardcoded VFX-kind path may remain in render.rs. Depends on T01 (axis+verbs) and T02 (schema+asset+validation).
  - Files: `src/windowed/render.rs`, `tests/windowed_only/vfx_asset_impact_render.rs`, `tests/windowed_only.rs`
  - Verify: cargo test --features windowed --test windowed_only

- [x] **T04: Add the headless grep-guard test proving the enum and string-match are gone** `est:20m`
  Why: Success criterion 2 (a static grep confirms VfxParticleKind and kind_from_name/vfx_particle_kind no longer exist in render.rs) must be made CI-provable in the headless lane so auto-mode can certify it without a window. src/ is git-tracked (not gitignored), so a test may read src/windowed/render.rs at compile time.
  - Files: `tests/animation/render_no_vfx_kind_guard.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation

## Files Likely Touched

- src/animation/vfx_asset.rs
- src/animation/placement.rs
- src/animation/mod.rs
- src/combat/runtime/registry.rs
- src/combat/blueprints/agumon/mod.rs
- tests/animation/placement_verbs.rs
- tests/animation.rs
- assets/digimon/agumon/vfx.ron
- tests/animation/vfx_asset_schema.rs
- tests/animation/vfx_asset_load.rs
- src/windowed/render.rs
- tests/windowed_only/vfx_asset_impact_render.rs
- tests/windowed_only.rs
- tests/animation/render_no_vfx_kind_guard.rs
