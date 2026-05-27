---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Adjust source-contract tests to new module shape

Update the windowed source-contract tests (renamon_extension_contract.rs, agumon_module_extraction.rs) to assert the new module boundaries (engine-generic render core vs per-species data) rather than the old single-file layout. Tests must still enforce the zero-engine-edit contract.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`

## Expected Output

- `Source-contract tests assert the new module boundaries and pass`

## Verification

cargo test --features windowed --test windowed_only (green, contracts re-pointed)
