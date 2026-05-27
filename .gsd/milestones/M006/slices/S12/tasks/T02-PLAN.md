---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Sweep residual singletons and unify roster registration

Audit render/registries.rs and the render submodules for any remaining single-slot registries and convert them to keyed maps; make every species register through the same uniform register() shape so the roster is symmetric.

## Inputs

- `src/windowed/render/registries.rs`
- `src/windowed/render/effects.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`

## Expected Output

- `No residual single-slot effect registries; uniform per-species registration across the roster`

## Verification

cargo test --features windowed --test windowed_only (green); cargo test (headless green)
