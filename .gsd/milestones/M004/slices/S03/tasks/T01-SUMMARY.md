---
id: T01
parent: S03
milestone: M004
key_files:
  - src/animation/vfx_asset.rs
  - src/animation/mod.rs
  - assets/digimon/agumon/vfx.ron
  - tests/animation/vfx_variant_selection.rs
  - tests/animation.rs
key_decisions:
  - Variant table modeled as nested BTreeMap<String, BTreeMap<String, EffectId>> (skill_id→variant_key→target) rather than a tuple key, for RON readability and deterministic iteration order (R004)
  - Kept the seam a pure free function (select_variant) + closed data (variants field), NOT a new ExtRegistries axis — premature per D033's deferred variation layer (D035)
  - select_variant returns None for unmapped keys and mapped-but-absent targets, mirroring resolve_effect's None-not-panic discipline so callers fall back to the base effect
  - DanglingVariant validation runs after the existing effect/on_expire checks in deterministic BTreeMap order, extending the MEM076 warn-once+skip data pattern
duration: 
verification_result: passed
completed_at: 2026-05-25T11:38:31.910Z
blocker_discovered: false
---

# T01: Added a pure, deterministic VfxContext → effect-tree variant selection seam (variants map + select_variant + DanglingVariant validation) over the owned Agumon vfx.ron

**Added a pure, deterministic VfxContext → effect-tree variant selection seam (variants map + select_variant + DanglingVariant validation) over the owned Agumon vfx.ron**

## What Happened

Implemented the only net-new architecture in S03 (D033 graft 5): a deterministic mapping from a synthetic selection context to an effect-tree variant, kept as a pure free function over closed data (D035) rather than a new ExtRegistries axis.

In `src/animation/vfx_asset.rs`:
- Added `VfxContext { skill_id: String, variant_key: String }` — a render-free, gameplay-numeric-free selection input (R012/MEM044).
- Added `#[serde(default)] pub variants: BTreeMap<String, BTreeMap<String, EffectId>>` to `VfxAsset` (skill_id → variant_key → target EffectId). Nested BTreeMaps (not a tuple key) keep RON readable and give deterministic ordering (R004). `deny_unknown_fields` is retained; the defaulted field is compatible so existing assets without a `variants` block keep loading.
- Added `select_variant<'a>(asset, ctx) -> Option<&'a EffectDef>`: looks up `variants[skill_id][variant_key]` then delegates to `resolve_effect`, returning None for any unmapped key or mapped-but-absent target (None-not-panic discipline, caller falls back to base).
- Extended `VfxValidationError` with `DanglingVariant { skill_id, variant_key, missing }` plus its Display arm, and extended `validate_effects` to check every variant target against `effects` in deterministic BTreeMap order, after the existing effect/on_expire checks (MEM076 warn-once+skip pattern).

`src/animation/mod.rs` already re-exports `vfx_asset::*` via glob, so `VfxContext`/`select_variant`/`VfxValidationError::DanglingVariant` are exported without change.

In `assets/digimon/agumon/vfx.ron`: authored a `variants` block mapping synthetic skill_id `baby_burner` with `base` → `baby_burner.detonate` and `empowered` → `baby_flame.impact` (both EXISTING effect ids), so the task is self-contained and the real asset still validates.

Added `tests/animation/vfx_variant_selection.rs` (registered in `tests/animation.rs`) following the include_str!/KNOWN_VERBS pattern: asserts the context selects the expected EffectDef; asserts byte-identical selection across 1000 calls (R004); asserts unmapped skill_id and unmapped variant_key both return None; asserts validate_effects on the real asset still passes; and a Q7 negative test that a synthetic asset with a dangling variant target returns DanglingVariant naming the offending skill_id/variant_key/missing id.

The baby_burner.detonate burst+flash enrichment named in the slice goal is out of T01's scope (a later task); T01 delivers only the variant-selection seam and its validation. No new GSD decisions were warranted — everything follows existing D033/D035 and the established validate_effects pattern.

## Verification

Ran `cargo test --test animation` (106 passed, 0 failed) — includes the 5 new vfx_variant_selection tests (expected-mapping, 1000-call determinism, unmapped→None, real-asset validation, Q7 dangling-variant) and the unchanged render_no_vfx_kind_guard. Ran `cargo build` — clean, no windowed dependency leaked into the headless lib (R016). Both via gsd_exec, exit 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 9191ms |
| 2 | `cargo build` | 0 | pass | 9191ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_variant_selection.rs`
- `tests/animation.rs`
