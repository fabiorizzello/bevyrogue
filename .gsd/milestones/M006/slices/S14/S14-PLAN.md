# S14: VfxAsset to enoki compile/adapt layer making VfxAsset the runtime source of truth

**Goal:** Build the VfxAsset -> enoki compile/adapt layer so authored VfxAsset data becomes the runtime source of truth for particle effects, with enoki as the rendering backend (per decision D052). Today enoki effects are hand-registered per Digimon; this slice makes them derive from VfxAsset.
**Demo:** VfxAsset drives enoki spawn via adapter; decision VfxAsset-canonical recorded first

## Must-Haves

- An adapter compiles a VfxAsset into the enoki effect representation the windowed renderer spawns; at least Agumon and Renamon drive their cast/impact effects through VfxAsset rather than hand-written enoki registration; round-trip parse of VfxAsset stays headless-testable; windowed_only suite green; manual K001 confirms unchanged on-screen VFX.

## Proof Level

- This slice proves: headless adapter test (VfxAsset compiles to expected enoki spec) + manual windowed parity sign-off (K001)

## Verification

- The adapter warns once when a VfxAsset verb has no enoki mapping yet, naming the verb, so unsupported authoring is visible rather than silently dropped.

## Tasks

- [ ] **T01: Define the VfxAsset to enoki adapter** `est:L`
  Implement an adapter that maps VfxAsset verbs/parameters (introspectable per D033) into the enoki effect representation the windowed renderer consumes. Cover the verbs Agumon/Renamon currently use. Warn-once on an unmapped verb.
  - Files: `src/animation/vfx_asset.rs`, `src/windowed/render.rs`
  - Verify: RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test (headless adapter test green)

- [ ] **T02: Drive Agumon and Renamon effects through VfxAsset** `est:M`
  Repoint Agumon and Renamon windowed registration so their cast/impact effects are produced by the VfxAsset->enoki adapter instead of hand-registered enoki structs, making VfxAsset canonical (D052).
  - Files: `src/windowed/digimon/agumon/mod.rs`, `src/windowed/digimon/renamon/mod.rs`
  - Verify: cargo test --features windowed --test windowed_only (green); manual cargo winx shows unchanged VFX

- [ ] **T03: Adapter round-trip and coverage test** `est:M`
  Add a headless test that parses a VfxAsset and asserts the adapter produces the expected enoki spec for the covered verbs, and that an unmapped verb triggers the warn path. Locks D052's source-of-truth contract.
  - Files: `tests/windowed_only/vfx_asset_adapter.rs`
  - Verify: cargo test --features windowed --test windowed_only (adapter test green)

## Files Likely Touched

- src/animation/vfx_asset.rs
- src/windowed/render.rs
- src/windowed/digimon/agumon/mod.rs
- src/windowed/digimon/renamon/mod.rs
- tests/windowed_only/vfx_asset_adapter.rs
