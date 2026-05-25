---
id: T01
parent: S02
milestone: M004
key_files:
  - src/animation/placement.rs
  - src/animation/vfx_asset.rs
  - src/animation/mod.rs
  - src/combat/runtime/registry.rs
  - src/combat/blueprints/agumon/mod.rs
  - tests/animation/placement_verbs.rs
  - tests/animation.rs
key_decisions:
  - PlacementCtx carries precomputed progress/phase scalars (no Bevy World/render handles) so verbs stay closed-form pure fns (R004/R016).
  - PlacementParams uses enum-level #[serde(deny_unknown_fields)] (applies to struct variants) to keep malformed RON failing with a field-naming parse error, mirroring the rest of the vfx_asset schema.
  - Verbs return [0,0] for a non-matching PlacementParams variant instead of panicking, so a mis-paired (verb, params) at load degrades to a static particle (slice graceful-skip contract).
  - arc_launch uses a deterministic closed-form lerp (target-caster)*progress, accepting a documented K001 visual deviation from the original stateful per-tick 0.55 ease-out lerp (endpoints match, mid-trajectory is now linear).
duration: 
verification_result: passed
completed_at: 2026-05-25T10:33:35.198Z
blocker_discovered: false
---

# T01: Stood up the PlacementExt Registry axis with four pure, deterministic placement verbs (converge_inward, fan_out, arc_launch, static) resolvable by namespaced id — additive, no serde shape change.

**Stood up the PlacementExt Registry axis with four pure, deterministic placement verbs (converge_inward, fan_out, arc_launch, static) resolvable by namespaced id — additive, no serde shape change.**

## What Happened

Implemented the S02 "First Proof": open-dispatch + typed-params for placement, end-to-end in the headless lib, with zero windowed dependency (R016) and full determinism (R004).

1. src/animation/vfx_asset.rs — added the render-free runtime context `PlacementCtx { age_ticks, ttl_ticks, progress, phase, caster_xy, target_xy }` (Debug+Clone+Copy+PartialEq, no Bevy World/render types), the closed editor-ready `PlacementParams` enum (ConvergeInward{radius_px,omega}, FanOut{spread_px}, ArcLaunch{}, Static) deriving Serialize+Deserialize+Reflect with enum-level `deny_unknown_fields`, and the `PlacementAnchor` enum (Mouth, CasterCenter, TargetCenter). These are purely additive — the existing Placement/Appearance serde shape is untouched (that surgery is T02), so all existing vfx_asset_* tests stay green.

2. src/animation/placement.rs (new) — four pure fns `fn(&PlacementCtx, &PlacementParams) -> [f32;2]`: converge_inward (ports baby_flame_ember_offset render.rs:948-957 closed form), fan_out (ports baby_flame_shard_offset render.rs:972-981 as pure direction*spread), arc_launch (closed-form lerp `(target-caster)*progress`, replacing the stateful 0.55 per-tick lerp at render.rs:1371-1385 — K001 visual deviation documented in the fn doc: endpoints match, in-between trajectory is now linear vs. ease-out), and static_placement. Each returns [0,0] for a non-matching params variant rather than panicking. Registered `pub mod placement;` in src/animation/mod.rs.

3. src/combat/runtime/registry.rs — added `pub struct PlacementExt` + `impl ExtPoint` (Fn = the placement signature, mirroring SelectorExt/CueExt) and `pub placements: Registry<PlacementExt>` to ExtRegistries.

4. src/combat/blueprints/agumon/mod.rs::register_agumon_ext — registered the four verbs under namespaced ids agumon/baby_flame/{converge_inward,fan_out,arc_launch,static}, copying the existing register("ns/name", fn) idiom.

5. tests/animation/placement_verbs.rs (new, registered in tests/animation.rs) — asserts all four verbs resolve via a freshly-built ExtRegistries+register_agumon_ext, ExtRegistries::default().placements.is_empty(), 1000-call bit-identical determinism per verb, converge_inward radius == radius_px at progress 0 and collapses to origin at progress 1, arc_launch endpoints [0,0]→target-minus-caster, fan_out reaches spread_px along phase at progress 1, and mismatched-params → [0,0]. Used a PlacementVerb type alias to avoid a clippy::type_complexity warning.

## Verification

Done-when gates both pass: `cargo build` (headless) finishes clean (exit 0), and `cargo test --test animation` reports 90 passed / 0 failed (7 new placement_verbs tests + all prior vfx_asset_* and animation tests still green). `cargo clippy --tests` produces no warning citing any of the new/modified files. No windowed feature was enabled (K001 honored — windowed binary never executed).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | pass | 6660ms |
| 2 | `cargo test --test animation` | 0 | pass — 90 passed; 0 failed | 10000ms |
| 3 | `cargo clippy --tests (grep placement_verbs.rs)` | 0 | pass — no warnings in new files | 1000ms |

## Deviations

none

## Known Issues

arc_launch's closed-form lerp is a deliberate visual deviation (K001) from the original ease-out per-tick lerp; the windowed dispatcher port (later S02 task) must account for this when reproducing the launch feel.

## Files Created/Modified

- `src/animation/placement.rs`
- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `tests/animation/placement_verbs.rs`
- `tests/animation.rs`
