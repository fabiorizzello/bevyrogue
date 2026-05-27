---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Scale-proof tests and zero-edit assertion

Add headless tests for the two new species' catalog discovery and skill legality, and extend the source-contract test to assert the render core control flow was not edited to accommodate them. Per the S15 anti-churn rule, the contract-test additions must assert ONLY the durable invariant — the engine render core contains no per-species id — as an absence-guard, NOT exact file shape, so S15 does not have to undo them.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/assets_data/catalog_discovery.rs`

## Expected Output

- `Tests prove both new species discover, are legal, and required no render-core edits`
- `contract additions are absence-guards on the no-per-species-id invariant only (S15-compliant)`

## Verification

cargo test (headless green); cargo test --features windowed --test windowed_only (green)
