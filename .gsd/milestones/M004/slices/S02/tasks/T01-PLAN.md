---
estimated_steps: 8
estimated_files: 7
skills_used: []
---

# T01: Stand up the PlacementExt axis + pure placement verbs (lib, headless, additive)

Why: This is the First Proof from S02-RESEARCH — prove open-dispatch + typed-params end-to-end in the headless lib before any render.rs surgery, resolving the D035/D036 design tension with compiling code. Additive only: it must NOT change the existing Placement/Appearance serde shape yet (that is T02), so the build and all existing animation tests stay green.

Do:
1. In src/animation/vfx_asset.rs add the pure runtime context `PlacementCtx { age_ticks: u32, ttl_ticks: u32, progress: f32, phase: f32, caster_xy: [f32;2], target_xy: [f32;2] }` (Debug+Clone+Copy+PartialEq; NO Bevy World/render types — R004) and the closed, editor-ready `PlacementParams` enum with variants `ConvergeInward { radius_px: f32, omega: f32 }`, `FanOut { spread_px: f32 }`, `ArcLaunch {}`, `Static` plus a `PlacementAnchor` enum `{ Mouth, CasterCenter, TargetCenter }`; both derive Serialize+Deserialize+Reflect (+ deny_unknown_fields where struct-shaped) per D034/D035.
2. Create src/animation/placement.rs with four pure fns of signature `fn(&PlacementCtx, &PlacementParams) -> [f32;2]`: `converge_inward` (port baby_flame_ember_offset render.rs:948-957 closed-form: radius = radius_px*(1.0-progress), angle = phase + age_ticks as f32 * omega, returns [r*cos,r*sin]), `fan_out` (port baby_flame_shard_offset render.rs:972-981: eased = 1-(1-progress)^2, dist = spread_px*eased, returns [dist*cos(phase),dist*sin(phase)] — NOTE the windowed dispatcher will instead drive shard outward distance via eval_scale(scale_curve) as S01 does; fan_out itself stays a pure direction*spread fn), `arc_launch` (closed-form lerp toward target: returns [(target_xy[0]-caster_xy[0])*progress, (target_xy[1]-caster_xy[1])*progress] — this replaces the stateful per-tick 0.55 lerp at render.rs:1371-1385 with a deterministic closed form; document the visual deviation for K001), and `static_placement` (returns [0.0,0.0]). Each fn returns [0.0,0.0] for a non-matching PlacementParams variant rather than panicking. Add `pub mod placement;` to src/animation/mod.rs.
3. In src/combat/runtime/registry.rs add `pub struct PlacementExt;` with `impl ExtPoint for PlacementExt { type Fn = fn(&crate::animation::vfx_asset::PlacementCtx, &crate::animation::vfx_asset::PlacementParams) -> [f32;2]; }` (mirror SelectorExt/CueExt) and add `pub placements: Registry<PlacementExt>` to the ExtRegistries struct (D036).
4. In src/combat/blueprints/agumon/mod.rs::register_agumon_ext, register the four verbs under namespaced ids: regs.placements.register("agumon/baby_flame/converge_inward", placement::converge_inward) etc. (ids: agumon/baby_flame/converge_inward, agumon/baby_flame/fan_out, agumon/baby_flame/arc_launch, agumon/baby_flame/static). Copy the existing register("ns/name", fn) idiom verbatim.
5. Add tests/animation/placement_verbs.rs and register it in tests/animation.rs (#[path] line): assert each verb resolves via a freshly-built ExtRegistries (register_agumon_ext) + placements.get(id).is_some(); assert ExtRegistries::default().placements.is_empty(); R004 determinism — call each verb 1000 times with fixed PlacementCtx+PlacementParams and assert bit-identical [f32;2] output; assert converge_inward at progress=0 has radius==radius_px and at progress=1 collapses to origin; assert arc_launch at progress=0 returns [0,0] and at progress=1 returns target-minus-caster; assert fan_out at progress=1 returns spread_px along phase direction.

Done when: cargo test --test animation passes (new placement_verbs tests + all existing vfx_asset_* tests still green) and cargo build (headless) is clean — proving verbs/params/ctx stay lib-side with no windowed dep (R016) and the existing serde shape is untouched.

## Inputs

- `src/animation/vfx_asset.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/windowed/render.rs`
- `tests/animation.rs`

## Expected Output

- `src/animation/placement.rs`
- `src/animation/vfx_asset.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/animation/mod.rs`
- `tests/animation/placement_verbs.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation

## Observability Impact

None at runtime (pure lib fns). Determinism + registry-round-trip tests are the inspection surface.
