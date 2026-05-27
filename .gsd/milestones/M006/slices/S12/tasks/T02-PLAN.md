---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T02: Sweep residual singletons and unify roster registration

Audit render.rs for any remaining single-slot registries and convert them to keyed maps; make every species register through the same uniform register() shape so the roster is symmetric.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`

## Expected Output

- `No residual single-slot effect registries; uniform per-species registration`

## Verification

cargo test --features windowed --test windowed_only (green); cargo test (headless green)
