---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T02: Repoint species imports and thin windowed/mod.rs

Update agumon/renamon windowed modules to import from the new registries path, and reduce src/windowed/mod.rs to panel + validation wiring plus the digimon::register_all call. Confirm species modules still only populate their own entries.

## Inputs

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`

## Expected Output

- `Species imports repointed; windowed/mod.rs thinned; warnings-clean windowed build`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)
