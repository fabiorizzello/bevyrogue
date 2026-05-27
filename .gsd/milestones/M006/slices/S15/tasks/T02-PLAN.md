---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Thin source-token tests to absence-guards

Reduce the remaining source-token/source-contract tests to assert only the durable invariant (engine render core contains no per-species id) and drop assertions tied to exact file layout, so the S09/S10 module split does not break them gratuitously.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`

## Expected Output

- `Source-token tests reduced to absence-guards that still fail on a reintroduced species id`

## Verification

cargo test --features windowed --test windowed_only (green); deliberately reintroduce a species id locally and confirm the guard fails, then revert
