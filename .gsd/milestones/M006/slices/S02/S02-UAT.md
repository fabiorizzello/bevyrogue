# S02: S02 — UAT

**Milestone:** M006
**Written:** 2026-05-26T11:13:34.398Z

# S02 UAT: CueRegistry seam + parametric math

## UAT Type
Automated headless integration tests (no windowed runtime, no K001 sign-off required — slice ships no rendered output).

## Preconditions
- Project builds clean (`cargo build --features windowed` succeeds)
- `src/ui/cues.rs` is present and registered in `src/ui/mod.rs` without a `#[cfg(feature = "windowed")]` gate
- `tests/ui.rs` and `tests/ui/cue_registry.rs` are present

## Test Steps

1. **Dependency boundary** — run `cargo test --no-default-features --features dev --test dependency_gating`
   - Expected: 2/2 pass; `bevy_enoki_absent_from_headless_graph` confirms enoki not present headless; `bevy_enoki_present_in_windowed_graph` confirms it is present windowed.

2. **CueRegistry contract** — run `cargo test --no-default-features --features dev --test ui`
   - Expected: all 10 tests pass, including:
     - `cue_registry_lookup_returns_registered_def` — registered cue is retrievable by id
     - `cue_registry_unknown_id_returns_none` — unknown id returns None (no panic)
     - `cue_registry_collision_panics_on_different_def` — conflicting re-register panics with both defs named
     - `cue_registry_idempotent_reregister_same_def` — identical re-register is a silent no-op

3. **Flash math** — covered by `tests/ui/cue_registry.rs`:
   - `flash_tint_parametric_zero_remaining_is_white` — `(1.0, 1.0, 1.0)` when remaining = 0
   - `flash_tint_parametric_matches_legacy_at_peak` — result within EPSILON of the legacy hit_feedback formula at remaining = total

4. **Shake math** — covered by `tests/ui/cue_registry.rs`:
   - `shake_offset_parametric_zero_remaining_is_zero` — Vec2::ZERO when remaining = 0
   - `shake_offset_parametric_nonzero_at_peak` — amplitude envelope in expected bounds
   - `shake_offset_parametric_determinism` — same inputs produce identical Vec2 across two calls
   - `camera_shake_uses_same_shake_math` — SpriteShake and CameraShake with identical params yield identical shake_offset_parametric result

5. **Windowed regression** — run `cargo test --features windowed`
   - Expected: 54/54 pass; no test regressions from the new ungated module.

## Expected Outcomes
- All 5 command sequences exit 0
- 10 ui integration tests pass headless and windowed
- 2 dependency_gating tests pass
- 54 windowed regression tests pass

## Edge Cases
- `CueDef::ParticleBurst { effect_id }` is not executed — only the type structure and registry lookup are proven here; effect_id → enoki handle resolution is S03's responsibility
- `hit_feedback.rs` is untouched and co-exists; its consts are duplicated inline in the test file because the module is windowed-gated

## Not Proven By This UAT
- Actual visual rendering of flash/shake/camera-shake (no windowed binary launched; K001 sign-off not applicable here)
- Windowed dispatch wiring of CueRegistry → sprite/camera components (S03 scope)
- ParticleBurst resolution to a bevy_enoki spawner (S03 scope)
