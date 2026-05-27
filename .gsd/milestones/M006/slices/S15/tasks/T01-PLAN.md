---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Baseline-green, then cut the Tier-1 contract test

Record cargo test --features windowed green as a baseline, then delete tests/windowed_only/vfx_windowed_contracts.rs whose coverage is subsumed by behavior tests (S08/S12 cast and effect proofs). Confirm no unique assertion is lost.

## Inputs

- `tests/windowed_only/vfx_windowed_contracts.rs`
- `tests/windowed_only/renamon_extension_contract.rs`

## Expected Output

- `vfx_windowed_contracts.rs removed; windowed suite still green`

## Verification

cargo test --features windowed --test windowed_only (green after removal)
