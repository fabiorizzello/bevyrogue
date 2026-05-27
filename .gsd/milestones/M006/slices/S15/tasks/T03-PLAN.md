---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Record the anti-churn rule

Append a DECISIONS.md rule (via gsd_save_decision) stating that windowed presentation correctness is proven by behavior tests, and source-shape assertions are limited to the engine-no-species-id absence guard, to prevent future brittle-test churn.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`

## Expected Output

- `Anti-churn decision recorded in DECISIONS.md`

## Verification

cargo test --features windowed --test windowed_only (green); DECISIONS.md contains the anti-churn rule
