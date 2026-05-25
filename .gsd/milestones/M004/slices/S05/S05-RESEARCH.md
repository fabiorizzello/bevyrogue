# S05 Research: Sharp Claws and rendering acceptance remediation

## Summary

S05 is a targeted-to-deep remediation slice. The codebase already has the M004 owned `VfxAsset` seam, registry-resolved placement verbs, on-expire chaining, and Baby Flame/Baby Burner data-driven effects. The remaining acceptance gap is real: `assets/digimon/agumon/vfx.ron` has no `sharp_claws.*` effect, `assets/digimon/agumon/anim_graph.ron` has no `SpawnParticle` in the Sharp Claws nodes, `src/windowed/render.rs::setup_camera` still spawns a bare `Camera2d`, and the current VFX colors/textures are alpha-blended `Sprite`s rather than an additive material.

There are no active canonical global requirements in preloaded requirements context. S05 supports milestone-local constraints from `M004-CONTEXT.md`: headless deterministic math (R004), windowed-only rendering tech (R002/R005), and K001 manual signoff remains deferred to S06.

Recommended delivery path: implement Sharp Claws in the existing RON-only data path using existing placement verbs, add HDR/bloom camera plumbing and bloom-intent color/compile tests, and explicitly document whether true additive blending is delivered or rescoped. If strict additive blending is required in S05, plan a separate risky task to replace VFX `Sprite` particles with `Mesh2d + MeshMaterial2d<CustomAdditiveVfxMaterial>`; Bevy 0.18's built-in 2D `ColorMaterial` only supports `Opaque`, `Mask`, and standard `Blend`, not additive.

## Skills Discovered

Installed skills already relevant from the prompt: `bevy`, `rust-development`, `rust-skills`, `rust-best-practices`, `rust-testing`, plus planning/doc skills `design-an-interface`, `grill-me`, `observability`, and `write-docs`.

`npx skills find` found external Bevy skills (`bfollington/terma@bevy`, `bevy-ecs-expert`, etc.) and Rust skills, but none were installed: project-local Bevy/Rust skills are already present and better aligned; the external Bevy skills had low install counts and no evidence of Bevy 0.18 VFX/bloom specificity.

Skill guidance applied from the installed skill descriptions:
- Bevy skill: keep Bevy ECS/windowed rendering concerns inside the windowed layer and respect existing ECS system ordering.
- Rust/testing skills: keep deterministic math headless-testable; use focused tests for asset/schema contracts before claiming completion.
- Design-an-interface/grill-me: separate the RON-only Sharp Claws option from the true-additive rendering option because they have different risks and proof surfaces.
- Observability/write-docs: make rescope/implementation status explicit in validation docs so the milestone validator cannot overcount S01-S04 proof.

## Implementation Landscape

### Already delivered upstream

- `src/animation/vfx_asset.rs`
  - Typed `VfxAsset`, `EffectDef`, `Appearance`, `Placement`, `PlacementParams`, `PlacementAnchor`, `eval_scale`, `eval_color`, `resolve_effect`, `validate_effects`, and variant selection.
  - `PlacementParams` is closed and typed. Reusing existing verbs is RON-only; adding a genuinely novel placement parameter still requires editing this core enum, despite the milestone ideal that a novel motion should be only one blueprint registration.
- `src/animation/placement.rs`
  - Existing pure verbs: `converge_inward`, `fan_out`, `arc_launch`, `static_placement`.
- `src/combat/blueprints/agumon/mod.rs`
  - Registers four placement verbs under `agumon/baby_flame/*` into `regs.placements`.
- `assets/digimon/agumon/vfx.ron`
  - Effects present: `baby_flame.charge`, `baby_flame.ember`, `baby_flame.projectile`, `baby_flame.impact`, `baby_flame.impact_flash`, `baby_burner.detonate`, `baby_burner.flash`.
  - No `sharp_claws.*` entry.
- `src/windowed/render.rs`
  - Loads `VfxAsset`, resolves effect ids, spawns particles from `EffectDef`, advances particles via placement registry, and handles on-expire chaining.
  - `on_enter_effect_ids` currently maps only Baby Flame particle names.
  - `VfxVisuals` loads only Baby Flame textures.
  - `setup_camera` is still `commands.spawn(Camera2d)`.
  - VFX particles are `Sprite`s, so they use Bevy's sprite path, not a custom additive material.
- `assets/digimon/agumon/anim_graph.ron`
  - Baby Flame has `SpawnParticle(name: "baby_flame_charge")` on `baby_flame_cast`.
  - Sharp Claws nodes (`sharp_claws_windup`, `sharp_claws_strike`, `sharp_claws_recover`) have no `SpawnParticle`; `sharp_claws_strike` only has `ReleaseKernel` at local frame 1.
- Existing tests to extend:
  - `tests/animation/vfx_asset_load.rs`
  - `tests/animation/vfx_asset_eval.rs`
  - `tests/animation/render_no_vfx_kind_guard.rs`
  - `tests/windowed_only/vfx_asset_impact_render.rs`
  - `src/windowed/render.rs` unit tests around `on_enter_effect_ids` and skill sync.

### Bevy 0.18 rendering facts

Local Bevy 0.18 examples show 2D bloom uses:

```rust
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};

commands.spawn((
    Camera2d,
    Camera { /* hdr can be set here */ ..default() },
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
));
```

Bevy's 2D bloom example makes objects bloom by using color channels above 1.0 (`Color::srgb(5.0, 5.0, 5.0)`). Current `vfx.ron` color comments say `0.0..=1.0`, and all authored RGB values are `<= 1.0`, so simply adding `Bloom` may compile but may not visibly satisfy the glow bar. S05 should either:

1. allow VFX RGB channels above 1.0 in `vfx.ron` and update doc/tests accordingly, or
2. apply a windowed-only emissive multiplier when converting evaluated VFX color into Bevy `Color`, and document that the glow intensity is renderer policy rather than asset data.

True additive blending is not available through built-in 2D `ColorMaterial`: `AlphaMode2d` variants are only `Opaque`, `Mask(f32)`, and `Blend`. Sprites are always transparent/alpha-blended. Delivering true additive needs a custom `Material2d` with `specialize()` overriding the pipeline blend state, plus particles spawned as textured rectangle meshes (`Mesh2d`, `MeshMaterial2d<...>`) instead of `Sprite` components.

## Recommended Scope Choice

### Preferred S05 scope

Deliver Sharp Claws + HDR/bloom now; rescope strict additive material unless the planner/user requires it before M004 validation.

Why:
- Sharp Claws can be added with low-risk RON + anim graph + tests.
- HDR/bloom camera is small, Bevy-supported, and windowed-gated.
- True additive materially changes the particle entity representation, requires custom shader/material plumbing, and is not needed to prove the data-driven VFX seam. It is better isolated as a follow-up or explicit extra task.

### If additive must be implemented

Make it its own first-class task after Sharp Claws is green:
- Add a windowed-only `VfxAdditiveMaterial` (or equivalent) in `src/windowed/render.rs` or a new `src/windowed/vfx_material.rs`.
- Register `Material2dPlugin::<VfxAdditiveMaterial>` in the windowed plugin.
- Spawn VFX as `Mesh2d(Rectangle)` + `MeshMaterial2d` rather than `Sprite`.
- Preserve `VfxParticle`, `VfxParticleSource`, `VfxParticleTarget` unchanged so the data-path and deterministic placement logic remain stable.
- Add windowed-only compile/contract tests that assert the material uses additive blend specialization. Expect more churn than the rest of S05.

## Natural Seams / Suggested Task Breakdown

### Task 1: Sharp Claws authored effect and trigger bridge

Files:
- `assets/digimon/agumon/vfx.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/windowed/render.rs`
- optionally new `assets/vfx/sharp_claws_slash.png`

Work:
- Add a `SpawnParticle(name: "sharp_claws_slash", origin: TargetCenter or CasterCenter, motion: Static)` to `sharp_claws_strike` in `anim_graph.ron`. Strike-node entry is the simplest visual trigger; if the exact local-frame-1 cue is desired, planner must add a cue-driven spawn branch instead of on-enter.
- Add `const AGUMON_SHARP_CLAWS_EFFECT_ID: &str = "sharp_claws.slash";` in `render.rs`.
- Extend `on_enter_effect_ids` with `"sharp_claws_slash" => &[AGUMON_SHARP_CLAWS_EFFECT_ID]`.
- Add `sharp_claws.slash` to `vfx.ron`. Lowest-risk RON-only design: use existing `FanOut` or `Static` placement, `anchor: TargetCenter`, short TTL, 3-5 particles, pale yellow/white color curve, and a slash texture key. If no new texture is created, reusing `baby_flame_impact` will prove the seam but may not satisfy visual quality in S06.
- Extend `VfxVisuals` + `load_vfx_visuals` + `vfx_texture_handle` if using a new texture key.

Watch-out: Existing `spawn_effect_by_id` only sets position/scale/color/texture. It does not rotate particles. A multi-slash effect using `FanOut` will move multiple same-oriented sprites outward; a convincing claw arc/diagonal slash would require either a texture that already contains multiple claw marks or render support for per-particle rotation.

### Task 2: Headless Sharp Claws contract tests

Files:
- `tests/animation/vfx_asset_load.rs`
- possibly `tests/animation/vfx_asset_schema.rs`
- `src/windowed/render.rs` unit tests

Work:
- Add `sharp_claws.slash` to real-asset presence checks. Consider renaming `agumon_vfx_contains_all_five_effects` because the asset now has more than Baby Flame effects.
- Assert the Sharp Claws effect has registered placement verb, expected spawn plan, short TTL, target/caster anchor, texture key, and fade curve.
- Assert `validate_effects_accepts_the_real_asset` remains green with unchanged `KNOWN_VERBS` if only existing verbs are reused.
- Add/update `render.rs` unit test for `on_enter_effect_ids("sharp_claws_slash")`.

### Task 3: HDR/bloom rendering acceptance implementation

Files:
- `src/windowed/render.rs`
- optional `tests/windowed_only/*`

Work:
- Change `setup_camera` to spawn camera tuple with Bevy 0.18 bloom components:
  - `Camera2d`
  - `Camera { hdr: true, ..default() }` if accepted by compile; local Bevy source confirms `Camera` has `pub hdr: bool`.
  - `Tonemapping::TonyMcMapface` (Bevy 2D bloom example recommends this family to desaturate toward white)
  - `Bloom::default()` or tuned `Bloom { composite_mode: BloomCompositeMode::Additive, ..Bloom::default() }`
  - `DebandDither::Enabled`
- Add a compile-level/windowed-only contract test if feasible, or at minimum static guard test that `setup_camera` source contains `Bloom`, `Tonemapping`, `DebandDither`, and `hdr: true`.
- Decide how particles become bright enough for bloom. Best data-driven approach: allow/author RGB values above `1.0` in `vfx.ron` for VFX and update tests/documentation to treat color channels as bloom-capable intensity, not clamped RGBA.

Watch-out: headless tests import `ColorKeyframe` comments but not validation; values above `1.0` parse as `f32`. Bevy `Color::srgba` examples use `Color::srgb(>1)` for bloom. Confirm whether `Color::srgba` preserves high channel values in compile tests; if not, switch to `Color::srgb` + alpha handling or use `LinearRgba` conversion in windowed code.

### Task 4: Additive decision artifact / formal rescope

Files:
- `.gsd/milestones/M004/slices/S05/*` via task summary and potentially a small `M004-RENDERING-ACCEPTANCE.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` or `M004-BOUNDARY-MAP.md` only if planner wants validation docs updated immediately

Work:
- If implementing only HDR/bloom, explicitly state that strict additive particle material is rescoped/deferred and why (Bevy 0.18 2D built-ins lack additive; true additive requires mesh/material refactor).
- If implementing true additive, add tests/proof and update S04/S05 validation artifacts to mark it delivered.
- Do not claim K001 visual signoff; S06 still owns human `cargo winx` review/waiver.

## First Proof

Highest-risk proof is compile viability of the rendering API choice, not RON parsing. First task should spike `setup_camera` with Bevy 0.18 `Camera { hdr: true }`, `Tonemapping`, `Bloom`, and `DebandDither`, then run:

```bash
cargo check --features windowed
```

If that fails, resolve Bevy imports/API before changing the VFX asset. Once windowed compile is green, Sharp Claws RON/anim graph work is straightforward and can be proven headlessly.

If additive is in scope, first proof becomes a tiny custom `Material2d` additive compile test before converting the particle spawn path. Do not start by replacing all particles.

## Verification Plan

Headless / CI-safe:

```bash
cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
```

Windowed compile/contract (does not run `cargo winx`):

```bash
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
```

Static guards to add or update:
- `render_no_vfx_kind_guard` should remain unchanged and green.
- New/updated guard for `setup_camera` should prove HDR/bloom code exists without launching a window.
- New asset tests should prove `sharp_claws.slash` exists, validates, and maps through registered placement verbs.

Manual-only (S06, not S05 completion proof):

```bash
cargo winx
```

User must visually review Baby Flame, Baby Burner, and Sharp Claws or formally waive K001. Auto-mode must not run this.

## Risks and Watch-outs

- **Additive ambiguity:** Bevy 0.18's built-in 2D path does not have an `AlphaMode2d::Add`; claiming additive without a custom material would be false. Either implement custom material or explicitly rescope additive.
- **Bloom without overbright colors may be invisible:** Current VFX RGB values are `<=1.0`; Bevy bloom examples use channels above `1.0`. Add HDR/bloom plus overbright VFX colors or an emissive multiplier.
- **Sharp Claws visual shape:** Existing placement verbs can create burst/static effects, not a true oriented claw swipe. For a convincing slash without new core params, use a texture containing the claw shape itself. Per-particle rotation is currently absent.
- **Closed `PlacementParams`:** A novel placement verb with new parameters requires editing the core enum, which weakens the milestone's ideal "one blueprint register only" claim. Prefer RON-only reuse for S05 unless necessary.
- **Keep windowed deps gated:** Do not move `Bloom`, `Material2d`, `Mesh2d`, `Image`, or render-specific imports into headless modules.
- **No manual signoff in S05:** S05 can deliver implementation and automated evidence, but S06 owns human UAT/waiver.

## Sources / Evidence Read

- `assets/digimon/agumon/vfx.ron` — current owned effects; no Sharp Claws.
- `assets/digimon/agumon/anim_graph.ron` — Sharp Claws nodes have no `SpawnParticle`.
- `src/animation/vfx_asset.rs` — typed schema, validation, eval helpers.
- `src/animation/placement.rs` — existing pure placement verbs.
- `src/combat/blueprints/agumon/mod.rs` — placement registry ids currently registered.
- `src/windowed/render.rs` — camera setup, VFX visuals, effect id bridge, spawn/advance particle path.
- `tests/animation/vfx_asset_load.rs`, `tests/animation/vfx_asset_eval.rs`, `tests/animation/render_no_vfx_kind_guard.rs`, `tests/windowed_only/vfx_asset_impact_render.rs` — proof surfaces to extend.
- Local Bevy 0.18.1 examples/source under cargo registry:
  - `bevy-0.18.1/examples/2d/bloom_2d.rs`
  - `bevy_sprite_render-0.18.1/src/mesh2d/material.rs`
  - `bevy_sprite_render-0.18.1/src/mesh2d/color_material.rs`
