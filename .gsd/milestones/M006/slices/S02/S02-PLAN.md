# S02: Cue primitive math + CueRegistry seam in lib

**Goal:** Deliver a headless-testable cue seam in the lib crate: a `CueRegistry` Bevy resource (string id → `CueDef`) and pure, deterministic parametric math for the flash and shake primitives, ported and generalized from the windowed-only `hit_feedback.rs` consts. `src/ui/cues.rs` is exposed with NO `#[cfg(feature = "windowed")]` gate and imports no `bevy_enoki`/`bevy_render` types (ParticleBurst stores an opaque `effect_id: String`), so the headless build and the `dependency_gating` test stay green. No windowed wiring — S03 consumes this seam.
**Demo:** cargo test (headless) covers flash/blink/shake/camera-shake param+decay math and CueRegistry lookup; all green. No windowed wiring yet — this is the generic seam downstream slices consume.

## Must-Haves

- `src/ui/cues.rs` compiles in the headless (`--no-default-features --features dev`) build with no cfg gate and no enoki/render imports.
- `CueDef` enum has `Flash`, `SpriteShake`, `CameraShake`, and `ParticleBurst { effect_id: String }` variants; `CueRegistry` is `Resource + Default` wrapping `HashMap<String, CueDef>` with `register()` and `get()`.
- `register()` is idempotent for an identical def and panics on a conflicting def for the same id (D047); `get()` returns `Option` and never panics.
- Parametric `flash_tint_parametric(remaining, total, peak)` and `shake_offset_parametric(remaining, total, amp, freq_x, freq_y)` reproduce the existing `hit_feedback.rs` formulas when given the legacy constants; CameraShake reuses the same shake fn.
- New headless tests under `tests/ui.rs` → `tests/ui/cue_registry.rs` cover registry lookup/unknown/collision/idempotency and the math endpoints + determinism; all green.
- `dependency_gating` tests stay green (bevy_enoki absent from headless graph); `cargo build --features windowed` and `cargo test --features windowed` do not regress.

## Proof Level

- This slice proves: Headless unit tests (real assertions) on pure functions + the registry contract, plus the existing `dependency_gating` boundary test re-run to prove no enoki leak. No manual/K001 sign-off needed — this slice ships no rendered output.

## Integration Closure

S03 consumes `CueRegistry` + `CueDef` + the parametric math (`flash_tint_parametric`, `shake_offset_parametric`) and wires dispatch to `DigimonSprite`/`Camera2d`; `ParticleBurst.effect_id` is resolved to a `bevy_enoki` handle only in the windowed binary. This slice closes the boundary by exposing the types lib-wide (no cfg gate) and proving the headless dep boundary holds. `hit_feedback.rs` is intentionally left untouched and co-exists this slice; S03 refactors it to delegate.

## Verification

- `register()` panics carry a clear message naming the conflicting id and both defs (startup fail-fast, never runtime). `get()` returning `None` is the documented "caller logs and no-ops" path consumed in S03; no new runtime logging added in this slice.

## Tasks

- [x] **T01: CueDef + CueRegistry + parametric cue math in src/ui/cues.rs (lib, no cfg gate)** `est:M`
  Why: M006 replaces the ad-hoc Agumon-coupled flash/shake consts with a data-driven CueRegistry of cosmetic primitives (D044/MEM108). S02 delivers the generic, headless-testable seam in the lib crate; S03 consumes it for windowed dispatch. The seam MUST stay enoki-free so the headless build and the dependency_gating test stay green (R002/R005).
  - Files: `src/ui/cues.rs`, `src/ui/mod.rs`
  - Verify: cargo check --no-default-features --features dev && cargo test --no-default-features --features dev dependency_gating && cargo build --features windowed

- [x] **T02: Headless tests: tests/ui.rs harness + tests/ui/cue_registry.rs (registry contract + math)** `est:M`
  Why: The cue seam is pure lib logic and must be proven headless (R003) without any windowed feature gate, mirroring the `tests/animation.rs` aggregator pattern. These tests are the slice's proof and pin the D047 collision/idempotency convention plus the parametric-math determinism (R004) that S03 depends on.
  - Files: `tests/ui.rs`, `tests/ui/cue_registry.rs`
  - Verify: cargo test --no-default-features --features dev && cargo test --features windowed

## Files Likely Touched

- src/ui/cues.rs
- src/ui/mod.rs
- tests/ui.rs
- tests/ui/cue_registry.rs
