---
estimated_steps: 11
estimated_files: 4
skills_used: []
---

# T03: Project detonate transitions into a windowed flash indicator

---
estimated_steps: 7
estimated_files: 4
skills_used:
  - bevy
  - make-interfaces-feel-better
  - rust-testing
---
Why: S04 needs a visible flash proof while preserving R005: presentation must stay behind `feature = "windowed"` and must not drive combat damage.

Do: Add a feature-gated, deterministic flash projection that listens to the generic Baby Burner detonate transition from T02. A small resource/helper such as `BabyBurnerFlashState` with a frame-count lifetime is sufficient; do not add wall-clock combat gating, RNG, RON/editor support, or wgpu-only dependencies to `src/combat`. Surface the flash through the existing combat panel/windowed helper pattern (for example a chip/overlay label plus tooltip containing target/cast/signal details), and wire the UI/render path to read this state without mutating combat. Extend `tests/windowed_preview_cache.rs` or a new feature-gated test to drive synthetic detonate transitions into the helper/resource and assert show, decrement, and hide behavior without opening a real window.

Done when: the feature-gated test proves the flash appears from the detonate transition, persists for a deterministic number of frames, hides after expiry, and leaves combat HP/events unchanged.

## Inputs

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/render.rs`
- `src/combat/runtime/signal.rs`
- `src/combat/observability/events.rs`
- `tests/windowed_preview_cache.rs`

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/render.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache

## Observability Impact

Adds a feature-gated inspection surface for detonate presentation state so missing VFX can be diagnosed independently from missing combat damage.
