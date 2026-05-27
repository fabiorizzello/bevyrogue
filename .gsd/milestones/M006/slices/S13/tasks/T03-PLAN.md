---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Scale-proof tests and zero-edit assertion

Add headless tests for the two new species' catalog discovery and skill legality, and extend the source-contract test to assert the render core control flow was not edited to accommodate them.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`

## Expected Output

- `Tests prove both new species discover, are legal, and required no render-core edits`

## Verification

cargo test (headless green); cargo test --features windowed --test windowed_only (green)
