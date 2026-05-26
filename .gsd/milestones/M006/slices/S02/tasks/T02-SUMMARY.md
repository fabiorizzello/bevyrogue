---
id: T02
parent: S02
milestone: M006
key_files:
  - tests/ui.rs
  - tests/ui/cue_registry.rs
key_decisions:
  - tests/ui.rs is ungated because the cue seam is pure lib logic provable headless (R002/R005)
  - Legacy hit_feedback constants/formulas duplicated inline rather than imported, since hit_feedback is windowed-gated and absent headless
  - camera_shake_uses_same_shake_math proves the shared-fn contract by feeding both SpriteShake and CameraShake params into shake_offset_parametric
duration: 
verification_result: untested
completed_at: 2026-05-26T11:11:02.462Z
blocker_discovered: false
---

# T02: Added headless tests/ui.rs harness + tests/ui/cue_registry.rs proving the cue seam contract through the public lib surface with no windowed gate

**Added headless tests/ui.rs harness + tests/ui/cue_registry.rs proving the cue seam contract through the public lib surface with no windowed gate**

## What Happened

Created the ui integration scope per R003, mirroring tests/animation.rs. tests/ui.rs is a NEW ungated aggregator (#[path = "ui/cue_registry.rs"] mod cue_registry;), deliberately no windowed cfg since the cue seam is pure lib logic provable headless (R002/R005). tests/ui/cue_registry.rs exercises bevyrogue::ui::cues through its public surface only (CueDef, CueRegistry, SrgbTriple, flash_tint_parametric, shake_offset_parametric), importing only bevy::math::Vec2. 10 tests cover the registry contract (D044/D047: lookup, unknown-id-none for empty and populated, collision should_panic, idempotent re-register), flash math (zero-remaining-white, matches-legacy-at-peak within EPSILON), and shake math (zero-remaining-zero, nonzero-at-peak envelope-bounded, determinism, and camera_shake_uses_same_shake_math which feeds params from both SpriteShake and CameraShake variants into the shared shake_offset_parametric and asserts identical output across the window). Legacy hit_feedback constants/formulas are duplicated inline since hit_feedback is windowed-gated and absent headless.</narrative>
<parameter name="verificationEvidence">[{"command": "cargo test --no-default-features --features dev --test ui --test dependency_gating", "exitCode": 0, "verdict": "pass", "durationMs": 702}, {"command": "cargo test --features windowed --test ui", "exitCode": 0, "verdict": "pass", "durationMs": 6285}]

## Verification

cargo test --no-default-features --features dev --test ui --test dependency_gating: ui harness 10/10 passed, dependency_gating 2/2 passed. cargo test --features windowed --test ui: 10/10 passed (no regression). Only new test files touched.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

Used bevy::math::Vec2 only instead of bevy::prelude (Color), since flash_tint_parametric returns SrgbTriple; Color is render-stack-only and absent headless. "White" asserted as the (1.0,1.0,1.0) triple.

## Known Issues

none

## Files Created/Modified

- `tests/ui.rs`
- `tests/ui/cue_registry.rs`
