---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T01: Windowed Agumon-vs-Agumon-dummy encounter bootstrap with two on-screen sprites

## Inputs

- None specified.

## Expected Output

- `src/combat/encounter/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `src/bin/combat_cli/config.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `tests/encounter_bootstrap_windowed.rs`

## Verification

cargo test --test encounter_bootstrap_windowed --features windowed
