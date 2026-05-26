# S02 Research: Cue primitive math + CueRegistry seam in lib

## Summary

S02 must deliver a `CueRegistry` Bevy resource (string id → cue definition) and pure deterministic math functions for four cue primitives — `Flash`, `SpriteShake`, `CameraShake`, and `ParticleBurst` — living entirely in the lib crate so they are headless-testable. The math to port is already implemented in `src/ui/hit_feedback.rs` (currently `#[cfg(feature = "windowed")]`); the goal is to generalize those hardcoded-const functions into parametrized versions and register them in a `CueRegistry`. No windowed wiring happens in this slice: S03 consumes the registry and wires dispatch to `DigimonSprite` and `Camera2d`. The `ParticleBurst` variant must store only an opaque `String` effect-id (not a `bevy_enoki` handle) so the headless build never pulls in enoki types; the dep-gating test (`tests/dependency_gating.rs`) must remain green. Registration collisions must be surfaced at startup, not silently last-writer-wins.

---

## Recommendation

### CueRegistry shape

```rust
// src/ui/cues.rs  (lib-crate, no cfg gate)
#[derive(Debug, Clone, PartialEq)]
pub enum CueDef {
    Flash { peak: (f32,f32,f32), ticks: u32 },
    SpriteShake { amp: f32, freq_x: f32, freq_y: f32, ticks: u32 },
    CameraShake { amp: f32, freq_x: f32, freq_y: f32, ticks: u32 },
    /// Opaque string id — resolves to a bevy_enoki handle only in the windowed
    /// binary. Storing a String keeps the headless lib enoki-free.
    ParticleBurst { effect_id: String },
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CueRegistry {
    entries: HashMap<String, CueDef>,
}

impl CueRegistry {
    /// Register a cue. Panics (at startup, never at runtime) if id is already
    /// registered to a different definition — surfaces collisions loudly.
    pub fn register(&mut self, id: impl Into<String>, def: CueDef) { ... }

    /// Look up a cue by id. Returns None for unknown ids (caller logs and
    /// no-ops; never panics).
    pub fn get(&self, id: &str) -> Option<&CueDef> { ... }
}
```

### Pure math functions (generalized from `hit_feedback.rs`)

```rust
// In the same file or a sub-module src/ui/cue_math.rs

pub fn flash_tint_parametric(remaining: u32, total: u32, peak: (f32,f32,f32)) -> Color { ... }
pub fn shake_offset_parametric(remaining: u32, total: u32, amp: f32, freq_x: f32, freq_y: f32) -> Vec2 { ... }
```

### Module path

New file `src/ui/cues.rs` exposed as `pub mod cues;` in `src/ui/mod.rs` **without any `#[cfg(feature = "windowed")]` gate** — this is the key change that makes it lib-only/headless. The existing `hit_feedback.rs` stays behind `#[cfg(feature = "windowed")]` for now (S03 will remove it or refactor it to call the new parametric functions).

### Build order (within the slice)

1. `src/ui/cues.rs` — `CueDef` enum + `CueRegistry` impl (registration + collision detection + lookup)
2. Pure math functions (`flash_tint_parametric`, `shake_offset_parametric`) in the same file or `cue_math.rs`
3. Headless unit tests in `tests/ui/cue_registry.rs` registered under a new `tests/ui.rs` harness (mirroring `tests/animation.rs` pattern)

---

## Implementation Landscape

### Files to read / modify

| File | Purpose |
|---|---|
| `src/ui/hit_feedback.rs` (lines 1–156) | Source of all current flash/shake math and consts — port to parametric |
| `src/ui/mod.rs` (4 lines) | Add `pub mod cues;` (no cfg gate) |
| `src/ui/cues.rs` (**new**) | `CueDef`, `CueRegistry`, parametric math |
| `tests/ui.rs` (**new harness**) | Aggregator mirroring `tests/animation.rs` |
| `tests/ui/cue_registry.rs` (**new**) | Headless unit tests for registry + math |
| `tests/dependency_gating.rs` (lines 1–83) | Must stay green — bevy_enoki absent from headless graph |

### Current hardcoded consts and formulas in `src/ui/hit_feedback.rs`

```rust
// Constants (lines 24-33)
pub const FLASH_TICKS: u32 = 8;
pub const SHAKE_TICKS: u32 = 8;
pub const SHAKE_MAX_PX: f32 = 4.0;
pub const DAMAGE_NUMBER_RISE_PX: f32 = 24.0;

// Flash formula (lines 121-130):
// peak tint = (1.0, 0.45, 0.45)
// t = (remaining / total).clamp(0,1)
// color = srgb( lerp(1.0, peak.r, t), lerp(1.0, peak.g, t), lerp(1.0, peak.b, t) )
pub fn flash_tint(remaining: u32, total: u32) -> Color {
    let t = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    const TINT: (f32, f32, f32) = (1.0, 0.45, 0.45);
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Color::srgb(lerp(1.0, TINT.0), lerp(1.0, TINT.1), lerp(1.0, TINT.2))
}

// Shake formula (lines 135-143):
// decay = (remaining / total).clamp(0,1)
// amplitude = SHAKE_MAX_PX * decay
// phase = remaining as f32
// offset = Vec2::new(amplitude * sin(phase * 1.7), amplitude * cos(phase * 2.3))
pub fn shake_offset(remaining: u32, total: u32) -> Vec2 {
    let decay = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    let amplitude = SHAKE_MAX_PX * decay;
    let phase = remaining as f32;
    Vec2::new(amplitude * (phase * 1.7).sin(), amplitude * (phase * 2.3).cos())
}
```

The parametric generalization replaces `SHAKE_MAX_PX` with `amp`, `1.7`/`2.3` with `freq_x`/`freq_y`, and `TINT` with a `peak: (f32,f32,f32)` param. The default cue registrations in S03 will use the existing values as defaults.

### Existing registry pattern to mirror

`src/animation/registry.rs` — `SkillGraphRegistry` (lines 91–127): a `HashMap<K,V>` wrapped in a `Resource`, `Default`, plus a `resolve()` / `resolve_snapshot()` lookup API. `AnimationGraphLookupDiagnostics` (lines 50–84) shows the structured-diagnostic pattern for failures (MEM040). The `CueRegistry` should follow the same shape: `HashMap<String, CueDef>` + `Resource + Default`, with `register()` and `get()` methods.

### `ui::mod.rs` gate situation

`src/ui/mod.rs` currently:
```rust
pub mod combat_panel;
#[cfg(feature = "windowed")] pub mod hit_feedback;
#[cfg(feature = "windowed")] pub mod phase_strip;
```

The new `pub mod cues;` must **not** carry a `cfg` gate — that is what makes it lib-accessible. `hit_feedback.rs` stays gated for now (S03 will refactor it to call the parametric functions from `cues.rs`).

### Natural seams

- `CueDef` enum + `CueRegistry` struct are pure data — no Bevy systems yet; no windowed feature gate.
- Parametric math functions are pure `fn(u32, u32, ...) -> Vec2/Color` — trivially unit-testable headless.
- `ParticleBurst { effect_id: String }` is the enoki-isolation seam: the string is resolved to a `Handle<Particle2dEffect>` only in the windowed binary (S03), never in the lib.
- Registration collision detection: `register()` panics with a clear message if two callers claim the same id with different definitions. This is a startup-only path (panic is acceptable; no runtime lookup panics).

---

## First Proof

**Highest-risk / biggest-unblocker:** prove `pub mod cues;` (no cfg gate) compiles cleanly in the headless build AND `tests/dependency_gating.rs::bevy_enoki_absent_from_headless_graph` still passes. The risk is accidentally importing any `bevy_enoki` or `bevy_render` type into `cues.rs`. The proof step is:

1. Write `src/ui/cues.rs` with `CueDef` (ParticleBurst uses `String`, not enoki handle) + `CueRegistry` stub.
2. Add `pub mod cues;` to `src/ui/mod.rs` without any cfg gate.
3. Run `cargo test --no-default-features --features dev dependency_gating` — both gating tests must pass.

This gates everything else because the entire slice depends on the headless boundary holding.

---

## Verification

```bash
# 1. Headless build must be clean
cargo check --no-default-features --features dev

# 2. Dep-gating: bevy_enoki must remain absent from headless graph
cargo test --no-default-features --features dev dependency_gating

# 3. Full headless test suite including new cue_registry tests
cargo test --no-default-features --features dev

# 4. Windowed build must not regress
cargo build --features windowed

# 5. Windowed test suite must not regress
cargo test --features windowed

# Specific new tests to add (all headless, no cfg gate):
# tests/ui/cue_registry.rs:
#   - cue_registry_lookup_returns_registered_def
#   - cue_registry_unknown_id_returns_none
#   - cue_registry_collision_panics_on_different_def
#   - cue_registry_idempotent_reregister_same_def (should succeed or panic — decide convention)
#   - flash_tint_parametric_endpoints (peak==(1.0,0.45,0.45) matches existing formula at t=1.0)
#   - flash_tint_parametric_zero_remaining_is_white
#   - shake_offset_parametric_zero_remaining_is_zero
#   - shake_offset_parametric_nonzero_at_peak
#   - shake_offset_parametric_determinism (same inputs → same outputs, always)
#   - camera_shake_parametric_same_math_as_sprite_shake (CameraShake uses the identical formula)
```

---

## Risks & Watch-outs

1. **Enoki-leak risk (highest):** If any import of `bevy_enoki` types sneaks into `src/ui/cues.rs` (e.g., by storing a `Handle<Particle2dEffect>` instead of `String`), the dep-gating test will fail and the headless build breaks. The `ParticleBurst { effect_id: String }` design is the explicit guard. The planner MUST ensure no enoki import appears in `cues.rs`.

2. **Determinism (R004):** Both `shake_offset_parametric` and `camera_shake_offset_parametric` use a fixed sinusoid scaled by `remaining/total` — no wall-clock, no RNG. The `phase` variable is `remaining as f32` (integer-tick-based), giving the same output for the same inputs every time. This must be preserved exactly; do not introduce `std::time` or `rand` calls.

3. **Collision handling:** Two modules registering the same cue id with different params is a programming error, not a runtime condition. The `register()` panic is correct here (fail-fast at startup). However, registering the *same* id with the *identical* definition (idempotent re-registration) could be allowed as a convenience (registration order independence). The convention needs to be decided and encoded in a test.

4. **`hit_feedback.rs` vs `cues.rs` co-existence:** During S02, both files will coexist — `hit_feedback.rs` (windowed-only, hardcoded consts) and `cues.rs` (lib, parametric). S03 will refactor `hit_feedback.rs` to delegate to `cues.rs` and remove the duplication. This is intentional staged delivery; the planner should NOT delete `hit_feedback.rs` in S02.

5. **Test harness placement:** The new `tests/ui/cue_registry.rs` needs a `tests/ui.rs` aggregator. Check that `tests/ui.rs` does not already exist before creating it. The pattern is: `tests/ui.rs` uses `#[path = "ui/cue_registry.rs"] mod cue_registry;` (no cfg gate), mirroring `tests/animation.rs`.

---

## Open Questions

1. **Cue-id naming scheme:** Per-skill vs per-event? Current thinking (M006-CONTEXT): string tags addressed like effect ids. Likely convention: `"<digimon>/<cue_name>"` (e.g., `"agumon/hit_flash"`, `"agumon/sprite_shake"`, `"agumon/camera_shake"`). The CueRegistry API is id-agnostic; naming is a caller convention. Resolve before S03 when the first `register()` calls are written.

2. **Idempotent re-registration:** Should `register()` allow the same id with the same definition twice (idempotent) or always panic on duplicate? The SkillGraphRegistry silently overwrites; the D044 constraint says collisions must be surfaced. Recommendation: panic on any duplicate, even identical — forces callers to be explicit. Decide and encode in a test.

3. **`CueDef::Flash` curve field:** D044 mentions `Flash{peak,ticks,curve}`. The current `flash_tint` formula is a linear lerp. Is a `curve` param needed in S02, or is linear-only sufficient with a future extension point? Recommendation: defer curve to S03+; use linear for S02 to keep the math trivial and testable.

4. **Camera-shake vs sprite-shake formula identity:** Both write a sinusoidal offset. Camera-shake writes `Camera2d.translation`; sprite-shake writes the sprite transform. The math is identical. Should they share one parametric function or be separate entry points? Recommendation: one shared `fn shake_offset_parametric(...)` with `CameraShake` and `SpriteShake` both dispatching to it — avoids math duplication and is easiest to test.

---

## Sources

- `src/ui/hit_feedback.rs` — existing flash/shake math (verbatim formulas extracted above)
- `src/animation/registry.rs` — `SkillGraphRegistry` / `StanceGraphRegistry` registry pattern
- `src/animation/reaction.rs` — `StanceReaction` pure-mapping headless pattern
- `tests/animation/stance_reaction_mapping.rs` — headless test pattern for pure lib functions
- `tests/windowed_only/windowed_hit_feedback.rs` — existing test coverage of hit_feedback (windowed-only; shows what tests exist and must not regress)
- `tests/dependency_gating.rs` — dep-gating test that must remain green
- `src/ui/mod.rs` — current cfg gate placement
- `Cargo.toml` — feature flags (`dev`, `windowed`, `bevy_enoki = { optional = true }`)
- `.gsd/milestones/M006/M006-CONTEXT.md` — D044 decision, error handling strategy
- `MEM108` (GSD memory) — CueRegistry architectural decision
- `MEM094` (GSD memory) — AgumonSprite shake/flash as absolute offset from SpriteRest
