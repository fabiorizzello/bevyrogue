---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T01: Variant-selection seam: VfxContext + variants map + select_variant + validation

Why: This is the only net-new architecture in S03 (D033 graft 5) and the only CI-provable success criterion — a deterministic mapping from a selection context to an effect-tree variant, mirroring how anim_graph state picks which tree to instantiate. It must stay a pure free function over data (D035: vocabulary is closed data, dispatch stays open), NOT a new ExtRegistries axis (premature per D033's deferred variation layer). No real gameplay unlock is wired (no unlock system exists; the context is synthetic per the slice contract).

Do: In src/animation/vfx_asset.rs add (1) `pub struct VfxContext { pub skill_id: String, pub variant_key: String }` — a render-free, gameplay-numeric-free selection input (R012/MEM044); (2) a top-level optional field on VfxAsset `#[serde(default)] pub variants: BTreeMap<String, BTreeMap<String, EffectId>>` keyed skill_id -> variant_key -> target EffectId (nested BTreeMap, not tuple key, for RON-friendliness + deterministic ordering per R004); keep `#[serde(deny_unknown_fields)]` (the defaulted field is compatible and existing assets without a `variants` block keep loading); (3) `pub fn select_variant<'a>(asset: &'a VfxAsset, ctx: &VfxContext) -> Option<&'a EffectDef>` — looks up variants[skill_id][variant_key] then delegates to resolve_effect, returning None for any unmapped key (caller falls back to base effect — mirrors resolve_effect's None-not-panic discipline); (4) extend validate_effects with a new `VfxValidationError::DanglingVariant { skill_id, variant_key, missing }` variant returned (in deterministic BTreeMap order, after the existing effect checks) when any variant target EffectId is absent from effects, plus its Display arm. Re-export VfxContext and select_variant from src/animation/mod.rs alongside the existing resolve_effect/validate_effects exports. Author a small `variants` block in assets/digimon/agumon/vfx.ron mapping a synthetic skill_id (e.g. "baby_burner") with two variant_keys (e.g. "base" -> baby_burner.detonate, "empowered" -> baby_flame.impact) to EXISTING effect ids so this task is self-contained and the real asset still validates. Add tests/animation/vfx_variant_selection.rs (registered in tests/animation.rs) following the include_str!/KNOWN_VERBS pattern of vfx_asset_load.rs: assert a synthetic VfxContext selects the expected EffectId; assert identical context yields a byte-identical selected EffectId across 1000 calls (R004 determinism, no RNG/clock); assert an unmapped (skill_id, variant_key) returns None; assert validate_effects on the real asset still passes; and a Q7 negative test that validate_effects on a synthetic asset with a dangling variant target returns DanglingVariant naming the offending skill_id/variant_key/missing id.

Done-when: cargo test --test animation passes including the new vfx_variant_selection tests and the unchanged render_no_vfx_kind_guard; cargo build is clean with no windowed dependency leaked into the headless lib (R016).

## Inputs

- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation.rs`
- `.gsd/milestones/M004/slices/S03/S03-RESEARCH.md`

## Expected Output

- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_variant_selection.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation 2>&1 | tail -20 && cargo build 2>&1 | tail -5
