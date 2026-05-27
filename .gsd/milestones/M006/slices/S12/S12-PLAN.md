# S12: Replace singleton effect registries with keyed-per-effect registries

**Goal:** Replace the singleton effect registries (DetonateEffectRegistry and any residual singletons in src/windowed/render.rs) with keyed-per-effect registries, and make roster presentation data uniform so every species is registered the same way through the per-species seam.
**Demo:** DetonateEffectRegistry and residual singletons become keyed; uniform roster presentation data

## Must-Haves

- Detonate and other effect lookups are keyed per (species, cue/effect) rather than a single shared slot; two species can each carry their own detonate effect without collision; presentation registration is uniform across the roster; windowed_only and headless suites green.

## Proof Level

- This slice proves: headless/windowed test proving no cross-species effect collision

## Verification

- Keyed registry lookup warns once (with the missing key) on a miss, replacing the silent fallback a singleton allowed.

## Tasks

- [ ] **T01: Convert DetonateEffectRegistry to a keyed registry** `est:M`
  Change DetonateEffectRegistry from a singleton slot to a keyed map (per species/effect id) in render/registries.rs, updating Agumon's registration and the render/effects.rs consumer accordingly. On a keyed miss, warn once using the shared warn-once util from S09 (do not re-implement a dedup).
  - Files: `src/windowed/render/registries.rs`, `src/windowed/render/effects.rs`, `src/windowed/digimon/agumon/mod.rs`
  - Verify: RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)

- [ ] **T02: Sweep residual singletons and unify roster registration** `est:M`
  Audit render/registries.rs and the render submodules for any remaining single-slot registries and convert them to keyed maps; make every species register through the same uniform register() shape so the roster is symmetric.
  - Files: `src/windowed/render/registries.rs`, `src/windowed/render/effects.rs`, `src/windowed/digimon/mod.rs`, `src/windowed/digimon/renamon/mod.rs`
  - Verify: cargo test --features windowed --test windowed_only (green); cargo test (headless green)

- [ ] **T03: Cross-species no-collision test** `est:S`
  Add a test where two species register distinct detonate/effect entries and assert each resolves to its own effect (no overwrite), proving the keyed registry fixed the singleton collision.
  - Files: `tests/windowed_only/keyed_effect_registry.rs`
  - Verify: cargo test --features windowed --test windowed_only (no-collision case green)

## Files Likely Touched

- src/windowed/render/registries.rs
- src/windowed/render/effects.rs
- src/windowed/digimon/agumon/mod.rs
- src/windowed/digimon/mod.rs
- src/windowed/digimon/renamon/mod.rs
- tests/windowed_only/keyed_effect_registry.rs
