# S03: Skill-tree variation via variant selection + Baby Burner port

**Goal:** Add a pure, deterministic variant-selection seam (D033 graft 5) that maps a synthetic VfxContext (skill_id, variant_key) to a selected effect-tree EffectId over the owned VfxAsset, and enrich the placeholder baby_burner.detonate into a real data-driven burst+flash — all reusing the existing S01/S02 resolver, placement verbs, and validate_effects path with no core registry-axis change and no real gameplay unlock wiring.
**Demo:** Headless test maps a VfxContext (e.g. a skill-tree unlock) to a selected effect-tree variant deterministically; cargo winx shows Baby Burner detonate rendered from assets/digimon/agumon/vfx.ron with no hardcoded VFX paths left in render.rs.

## Must-Haves

- A headless test maps a synthetic VfxContext to a selected effect-tree variant deterministically (identical context -> identical EffectId across 1000 calls; unmapped -> None); validate_effects rejects a dangling variant target by name; the enriched baby_burner.detonate (+ its on_expire flash) round-trips and passes validation against the real asset; the existing render_no_vfx_kind_guard continues to prove zero hardcoded VFX paths in render.rs. Visual sign-off of Baby Burner detonate in cargo winx is K001 (human-only, not auto-certifiable).

## Proof Level

- This slice proves: Headless determinism + validation tests are the full CI-provable surface (R004/R002/R016). Windowed build/contract tests confirm no regression in the data-driven detonate spawn. Visual quality is K001 manual sign-off only.

## Integration Closure

select_variant slots in front of the existing spawn_effect_by_id path as a pure data lookup; no new ExtRegistries axis (D035 — only vocabulary is closed data, dispatch stays open). variants is presentation-only data inside VfxAsset, carrying no gameplay numeric onto the SpawnParticle command surface (R012/MEM044). No windowed wiring of selection is required for the slice contract — burden of proof is headless; a real skill-tree unlock source is explicitly out of scope (no unlock system exists).

## Verification

- None new. validate_effects already surfaces the first offending entry as data (warn-once + skip at the windowed layer); the new DanglingVariant variant extends that same pattern (MEM076).

## Tasks

- [x] **T01: Variant-selection seam: VfxContext + variants map + select_variant + validation** `est:M`
  Why: This is the only net-new architecture in S03 (D033 graft 5) and the only CI-provable success criterion — a deterministic mapping from a selection context to an effect-tree variant, mirroring how anim_graph state picks which tree to instantiate. It must stay a pure free function over data (D035: vocabulary is closed data, dispatch stays open), NOT a new ExtRegistries axis (premature per D033's deferred variation layer). No real gameplay unlock is wired (no unlock system exists; the context is synthetic per the slice contract).
  - Files: `src/animation/vfx_asset.rs`, `src/animation/mod.rs`, `assets/digimon/agumon/vfx.ron`, `tests/animation/vfx_variant_selection.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation 2>&1 | tail -20 && cargo build 2>&1 | tail -5

- [ ] **T02: Enrich baby_burner.detonate into a real data-driven burst + flash** `est:S`
  Why: Today's baby_burner.detonate (vfx.ron:140-161) is a deliberate S02 placeholder — a single static size-18 flat quad reproducing the old Generic-kind detonate. The 'no hardcoded VFX paths' criterion is already satisfied (grep-guard), but the K001 visual review needs a detonate worth signing off. This must reuse the existing pure verbs (fan_out + static) and the on_expire chaining mechanism (MEM076/MEM077) — no parallel math, no novel placement verb (so no register_agumon_ext change), demonstrating the milestone's RON-only reuse path.
  - Files: `assets/digimon/agumon/vfx.ron`, `tests/animation/vfx_asset_load.rs`
  - Verify: cargo test --test animation 2>&1 | tail -20 && cargo build --features windowed 2>&1 | tail -5 && cargo test --features windowed --test windowed_only 2>&1 | tail -20

## Files Likely Touched

- src/animation/vfx_asset.rs
- src/animation/mod.rs
- assets/digimon/agumon/vfx.ron
- tests/animation/vfx_variant_selection.rs
- tests/animation.rs
- tests/animation/vfx_asset_load.rs
