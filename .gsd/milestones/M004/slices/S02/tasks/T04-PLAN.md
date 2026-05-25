---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T04: Add the headless grep-guard test proving the enum and string-match are gone

Why: Success criterion 2 (a static grep confirms VfxParticleKind and kind_from_name/vfx_particle_kind no longer exist in render.rs) must be made CI-provable in the headless lane so auto-mode can certify it without a window. src/ is git-tracked (not gitignored), so a test may read src/windowed/render.rs at compile time.

Do: Add tests/animation/render_no_vfx_kind_guard.rs and register it in tests/animation.rs via a #[path] line. The test does `const SRC: &str = include_str!("../../src/windowed/render.rs");` (compile-time, feature-independent) and asserts SRC does not contain the identifiers "VfxParticleKind", "vfx_particle_kind", or "kind_from_name". Keep it a plain headless test (no windowed feature needed) so it runs in cargo test --test animation. If any identifier is still present, the assertion message should name which one, so a future agent sees exactly what remains.

Done when: cargo test --test animation passes including the new guard test (which transitively proves T03 fully removed the enum + string-match).

## Inputs

- `src/windowed/render.rs`
- `tests/animation.rs`

## Expected Output

- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation

## Observability Impact

The guard test is itself the inspection surface: on failure it names the surviving forbidden identifier.
