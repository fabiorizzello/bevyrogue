---
id: S03
parent: M004
milestone: M004
provides:
  - VfxContext + select_variant seam (headless, deterministic)
  - baby_burner.detonate enriched fan-out burst + flash (data-driven, RON-only)
  - DanglingVariant validation coverage
requires:
  - S01: typed VfxAsset schema; resolve/eval API (`resolve_effect`, `eval_scale`, `eval_color`); owned `assets/digimon/agumon/vfx.ron` load path
  - S02: `PlacementExt` registry axis; registered Agumon placement verbs; `validate_effects`; registry-resolved windowed VFX data path
affects:
  []
key_files:
  - src/animation/vfx_asset.rs
  - assets/digimon/agumon/vfx.ron
  - tests/animation/vfx_variant_selection.rs
  - tests/animation/vfx_asset_load.rs
key_decisions:
  - select_variant is a pure free fn over VfxAsset data, not a new registry axis — consistent with D033/D035
  - VfxContext carries only render-free scalars (skill_id, variant_key) — no gameplay numeric (R012)
  - RON-only reuse: detonate enrichment uses fan_out + static, no new placement verb, no register_agumon_ext change
  - Visual K001 sign-off deferred to manual cargo winx user review
patterns_established:
  - Variant selection via pure free fn over VfxAsset.variants map — select_variant mirrors resolve_effect discipline (None not panic)
  - DanglingVariant validation follows DanglingOnExpire pattern: first offender in BTreeMap order, named in error
  - RON-only feature extension: new behavior (variant selection) via data + free fn, zero src/ changes for the data path
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-25T14:49:51.313Z
blocker_discovered: false
---

# S03: Variant-selection seam + Baby Burner detonate enrichment

**VfxContext variant-selection seam (pure, headless, deterministic) + baby_burner.detonate enriched to fan-out burst + flash via RON-only verb reuse**

## What Happened

T01 delivered the variant-selection seam: VfxContext type, variants map in VfxAsset schema, select_variant pure fn, DanglingVariant validation extension, and a full headless determinism test suite (vfx_variant_selection.rs). T02 enriched baby_burner.detonate in vfx.ron from a placeholder static quad to a real data-driven fan-out shard burst (8 shards, spread 64px, ease-out scale, alpha-fade color) chaining baby_burner.flash (static at TargetCenter, ttl 2, bright alpha-fade), using only the four already-registered placement verbs — demonstrating the milestone's RON-only reuse path. Tests in vfx_asset_load.rs cover both effects with spawn plan, curve, and on_expire chain assertions. validate_effects covers DanglingVariant in addition to the existing UnknownVerb and DanglingOnExpire cases. The windowed detonate spawn contract (baby_burner.detonate → spawn_effect_by_id via Registry) continues to hold per windowed_only tests. Visual K001 sign-off (cargo winx) is a manual user step not assertable in CI.

## Verification

cargo test --test animation: 110 passed, 0 failed. cargo build --features windowed: clean. cargo test --features windowed --test windowed_only: 32 passed, 0 failed.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
