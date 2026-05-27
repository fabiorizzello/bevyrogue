---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Add Agumon cast-driven particle proof

Add a windowed-scope test proving Agumon's cast cue resolves to its registered enoki effect through the same seam, locking the cast->effect contract for the reference Digimon.

## Inputs

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`

## Expected Output

- `Windowed test asserting Agumon cast cue maps to its enoki effect`

## Verification

cargo test --features windowed --test windowed_only (Agumon cast proof green)
