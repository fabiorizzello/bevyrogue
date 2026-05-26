---
estimated_steps: 9
estimated_files: 2
skills_used: []
---

# T02: Headless tests: tests/ui.rs harness + tests/ui/cue_registry.rs (registry contract + math)

Why: The cue seam is pure lib logic and must be proven headless (R003) without any windowed feature gate, mirroring the `tests/animation.rs` aggregator pattern. These tests are the slice's proof and pin the D047 collision/idempotency convention plus the parametric-math determinism (R004) that S03 depends on.

Do:
1. Confirm `tests/ui.rs` does not already exist (it does not as of planning). Create `tests/ui.rs` (NEW) as an aggregator mirroring `tests/animation.rs`: `#[path = "ui/cue_registry.rs"] mod cue_registry;` — NO cfg gate.
2. Create `tests/ui/cue_registry.rs` (NEW). Import `use bevyrogue::ui::cues::{CueDef, CueRegistry, flash_tint_parametric, shake_offset_parametric};` and `use bevy::prelude::*;` for `Color`/`Vec2`. CueRegistry tests need only a plain `CueRegistry::default()` — no `App` required.
3. Registry tests: `cue_registry_lookup_returns_registered_def` (register then get == Some(&def)); `cue_registry_unknown_id_returns_none` (get on empty/other id == None); `cue_registry_collision_panics_on_different_def` (`#[should_panic]`: register same id with two DIFFERENT defs); `cue_registry_idempotent_reregister_same_def` (register identical def twice does NOT panic and get still returns it).
4. Flash math tests: `flash_tint_parametric_zero_remaining_is_white` (remaining==0 → Color::WHITE); `flash_tint_parametric_matches_legacy_at_peak` (with peak (1.0,0.45,0.45), remaining==total → srgb ≈ (1.0,0.45,0.45) within f32 epsilon).
5. Shake math tests: `shake_offset_parametric_zero_remaining_is_zero` (remaining==0 → Vec2::ZERO); `shake_offset_parametric_nonzero_at_peak` (remaining==total, amp 4.0 → non-zero offset within amp bound); `shake_offset_parametric_determinism` (same inputs twice → identical Vec2); `camera_shake_uses_same_shake_math` (assert CameraShake-param call equals SpriteShake-param call for identical amp/freqs — the shared-fn contract).
6. Compare against legacy formulas inline (recompute expected with the documented constants) rather than importing windowed-gated `hit_feedback` (which is unavailable in the headless build).

Done-when: `cargo test --no-default-features --features dev` runs the new `ui` harness green AND `dependency_gating` stays green; `cargo test --features windowed` does not regress.

## Inputs

- `src/ui/cues.rs`
- `src/ui/hit_feedback.rs`
- `tests/animation.rs`
- `tests/dependency_gating.rs`

## Expected Output

- `tests/ui.rs`
- `tests/ui/cue_registry.rs`

## Verification

cargo test --no-default-features --features dev && cargo test --features windowed

## Observability Impact

Test names encode the contract (collision/idempotency/determinism) so a future failure points directly at the violated cue invariant.
