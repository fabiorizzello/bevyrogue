---
id: S02
parent: M006
milestone: M006
provides:
  - CueRegistry type (Resource, HashMap<String, CueDef>) with register()/get() contract
  - CueDef enum: Flash, SpriteShake, CameraShake, ParticleBurst { effect_id: String }
  - flash_tint_parametric(remaining, total, peak) -> SrgbTriple
  - shake_offset_parametric(remaining, total, amp, freq_x, freq_y) -> Vec2
  - SrgbTriple type alias = (f32, f32, f32)
requires:
  []
affects:
  []
key_files:
  - src/ui/cues.rs
  - src/ui/mod.rs
  - tests/ui.rs
  - tests/ui/cue_registry.rs
key_decisions:
  - flash_tint_parametric returns SrgbTriple = (f32,f32,f32) not bevy::Color, because bevy_color is render-stack-only and absent headless; S03 maps to Color::srgb(r,g,b) (MEM113/MEM114)
  - ParticleBurst.effect_id is an opaque String — enoki handle resolution happens only in S03's windowed binary (enoki-isolation seam)
  - CameraShake reuses shake_offset_parametric verbatim — no separate camera math needed
  - register() conflict is a startup fail-fast panic naming id+both defs; equal-def re-register is idempotent no-op (D047/D044)
patterns_established:
  - Ungated lib module pattern: src/ui/cues.rs has no cfg gate and imports no render/enoki types, making the cue seam provable headless
  - tests/ui.rs aggregator mirrors tests/animation.rs — pure lib test harness with no windowed gate
  - Legacy windowed constants duplicated inline in headless tests rather than imported, since the source module is windowed-gated
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T11:13:34.398Z
blocker_discovered: false
---

# S02: S02

**Delivered headless-testable CueDef/CueRegistry seam + parametric flash/shake math in lib with zero enoki/windowed coupling; 10 integration tests green, dependency boundary holds.**

## What Happened

S02 delivered the generic cue seam that S03 will consume for windowed dispatch. Two tasks were executed sequentially.

**T01** created `src/ui/cues.rs` as an ungated lib module (no `#[cfg(feature = "windowed")]`) and registered it in `src/ui/mod.rs` alongside the windowed-gated `hit_feedback` and `phase_strip` modules. `CueDef` (Debug/Clone/PartialEq) has four variants: `Flash { peak: SrgbTriple, ticks: u32 }`, `SpriteShake { amp, freq_x, freq_y, ticks }`, `CameraShake { amp, freq_x, freq_y, ticks }`, and `ParticleBurst { effect_id: String }`. The `String` effect-id is the structural enoki-isolation seam — resolved to a bevy_enoki handle only in S03's windowed binary. `CueRegistry` (Resource/Default/Debug/Clone) wraps `HashMap<String, CueDef>`; `register()` is idempotent for identical defs and fail-fast panics naming both defs on conflict (D047/D044); `get()` returns `Option` and never panics. Two pure parametric math functions were delivered: `shake_offset_parametric` generalizes hit_feedback's shake formula with decay envelope and per-axis sin/cos; `CameraShake` reuses the same function verbatim. `flash_tint_parametric` generalizes the flash-tint formula, returning `SrgbTriple = (f32, f32, f32)` — **not** `bevy::Color** — because bevy_color is render-stack-only and absent from the headless graph (MEM114); S03 maps to `Color::srgb(r,g,b)` at the call site.

**T02** created the `tests/ui.rs` aggregator (ungated, mirroring `tests/animation.rs`) and `tests/ui/cue_registry.rs` with 10 integration tests exercising the public surface only: registry lookup, unknown-id returns None, collision should_panic, idempotent re-register, flash white-guard, flash legacy-match at peak (within EPSILON), shake zero-guard, shake nonzero envelope bound, shake determinism, and camera-shake-uses-same-math (feeds both SpriteShake and CameraShake params into `shake_offset_parametric` and asserts identical output). Legacy hit_feedback constants are duplicated inline since `hit_feedback` is windowed-gated.

**Key deviation:** T01 plan specified `flash_tint_parametric` returning `bevy::Color`; changed to `SrgbTriple` because `bevy_color` is absent from the headless dependency graph and importing it would break R005 and the dependency_gating test. Documented as MEM113/MEM114.

`hit_feedback.rs` was intentionally left untouched; it co-exists and S03 will delegate to the new primitives.

## Verification

All slice verification commands passed fresh in this session:

1. `cargo test --no-default-features --features dev --test ui --test dependency_gating` — 10/10 ui cue_registry tests pass, 2/2 dependency_gating tests pass (bevy_enoki absent headless, present windowed). Exit 0.
2. `cargo test --features windowed --test ui` — 10/10 ui tests pass under windowed feature flag. Exit 0.
3. `cargo test --no-default-features --features dev` (full headless suite) — all tests pass including the new ui harness. Exit 0.
4. `cargo build --features windowed` — Finished dev profile. Exit 0.
5. `cargo test --features windowed` — 54/54 windowed tests pass; no regression. Exit 0.

Dependency boundary confirmed: bevy_enoki absent from headless graph, `src/ui/cues.rs` imports only `bevy::prelude::Resource` (via derive) + `bevy::math::Vec2` + `std::collections::HashMap` — no enoki/render types.

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

["T01 plan specified flash_tint_parametric returning bevy::Color; changed to SrgbTriple=(f32,f32,f32) because bevy_color is absent headless and importing it would break R005/dependency_gating. S03 maps via Color::srgb(r,g,b). Legacy equivalence preserved through that mapping."]

## Known Limitations

None.

## Follow-ups

["S03: wire CueRegistry + CueDef dispatch to DigimonSprite/Camera2d in windowed binary; map SrgbTriple to Color::srgb(r,g,b); resolve ParticleBurst.effect_id to bevy_enoki handle; refactor hit_feedback.rs to delegate to the new parametric math"]

## Files Created/Modified

None.
