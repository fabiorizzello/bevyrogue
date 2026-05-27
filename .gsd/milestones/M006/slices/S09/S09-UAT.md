# S09: Extract shared registries and types out of render.rs into render/registries.rs — UAT

**Milestone:** M006
**Written:** 2026-05-27T11:50:03.263Z

## Preconditions

- Codebase at end-of-S09 commit state
- Rust toolchain available (`cargo build`, `cargo test`)

## Steps

1. Run `RUSTFLAGS="-D warnings" cargo build --features windowed`
   - **Expected**: exit 0, zero warnings or errors
2. Run `cargo test --features windowed --test windowed_only`
   - **Expected**: 75 tests pass, 0 fail
3. Run `cargo test --lib warn_once`
   - **Expected**: 2 tests pass (per-key dedup; has_warned/clear lifecycle)
4. Grep `SpritePresentationRegistry` in the repo
   - **Expected**: type definition found in `src/windowed/render/registries.rs`; internal `use` reference in `src/windowed/render.rs`; no `pub use` re-export remaining
5. Grep for `render::registries` in `src/windowed/digimon/agumon/mod.rs` and `src/windowed/digimon/renamon/mod.rs`
   - **Expected**: both files import from the new canonical `crate::windowed::render::registries` path
6. Grep for `WarnOnce` in `src/warn_once.rs`, `src/lib.rs`, `src/animation/registry.rs`
   - **Expected**: struct defined in warn_once.rs, declared as `pub mod warn_once` in lib.rs, and used as `Local&lt;WarnOnce&lt;AssetId&lt;AnimGraph&gt;&gt;&gt;` in animation/registry.rs
7. Run `cargo build` (headless, no features)
   - **Expected**: exit 0, no windowed types leaked into the headless build

## Expected Outcomes

- Clean build and 75/75 windowed tests green after a pure structural move
- Registry type definitions live in `render/registries.rs`, not in `render.rs`
- Species modules import from `render::registries` directly (no pass-through re-export)
- `WarnOnce&lt;K&gt;` is a shared lib-level util, no longer inline in animation/registry.rs
- Headless build remains clean (R016 invariant preserved)

## Edge Cases

- `RUSTFLAGS="-D warnings"` must pass: any unused import introduced by the relocation would surface here; T02 already confirmed the EnokiEffect trim resolves the only such case
- windowed/mod.rs should be untouched: it already held only panels + validation + register_all wiring; no further thinning required

## UAT Type

Structural refactor parity — automated (cargo build + cargo test only)

## Not Proven By This UAT

Visual correctness of presentation effects at runtime (Digimon sprites, VFX, cue feedback) — requires K001 manual windowed run. No functional behavior was changed by this slice.
