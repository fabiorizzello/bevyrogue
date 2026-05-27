# S09: Extract shared registries and types out of render.rs into render/registries.rs

**Goal:** Extract the shared engine-generic registries and types out of the monolithic src/windowed/render.rs into a dedicated src/windowed/render/registries.rs module, repoint Agumon and Renamon imports, and thin src/windowed/mod.rs toward panels + validation only. Pure structural move, no behavior change.
**Demo:** render.rs imports from registries module; species imports repointed; tests green

## Must-Haves

- render.rs imports registry/type definitions from the new registries module; species modules import from the new location; no public behavior changes; full headless suite and windowed_only tests stay green. Diff is a move + re-export, not a rewrite.

## Proof Level

- This slice proves: full test suite green before and after (refactor parity)

## Verification

- None new; preserve existing warn-once diagnostics added in S06/S08 across the move.

## Tasks

- [ ] **T01: Carve registries and types into render/registries.rs** `est:M`
  Move the engine-generic registry structs/resources and shared presentation types out of render.rs into a new src/windowed/render/registries.rs, re-exporting as needed so external call sites keep compiling. No logic edits.
  - Files: `src/windowed/render.rs`, `src/windowed/render/registries.rs`
  - Verify: cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)

- [ ] **T02: Repoint species imports and thin windowed/mod.rs** `est:M`
  Update agumon/renamon windowed modules to import from the new registries path, and reduce src/windowed/mod.rs to panel + validation wiring plus the digimon::register_all call. Confirm species modules still only populate their own entries.
  - Files: `src/windowed/digimon/agumon/mod.rs`, `src/windowed/digimon/renamon/mod.rs`, `src/windowed/mod.rs`
  - Verify: RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)

## Files Likely Touched

- src/windowed/render.rs
- src/windowed/render/registries.rs
- src/windowed/digimon/agumon/mod.rs
- src/windowed/digimon/renamon/mod.rs
- src/windowed/mod.rs
