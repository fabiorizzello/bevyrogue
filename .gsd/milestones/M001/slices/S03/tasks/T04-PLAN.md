---
estimated_steps: 6
estimated_files: 9
skills_used: []
---

# T04: Run full validation closeout and protect prior contracts

Why: S03 changes a central asset plugin and public animation module exports, so closeout must prove the new validator and prior S01/S02 contracts together. Expected executor skills: verify-before-complete, test, rust-best-practices.

Do: Run the focused validation tests and the prior graph/clip parse and asset tests before the full suite. If failures appear, fix the implementation or tests rather than weakening assertions. Do not use `windowed`; this slice is headless only. Confirm that no test reads `.gsd/`, `.planning/`, `.audits/`, or other gitignored planning paths. If any validation diagnostics are hard to understand during failures, improve the typed diagnostic detail before closing the task.

Done when: All listed commands pass in a fresh closeout run and the slice has executable evidence for happy path, negative validation fixtures, adapter catalog proof, and full regression.

Q4 Requirement Impact: validates R004, R005, and R008; protects R001/R002/R003 behaviors already shipped by S01/S02.
Q5 Failure Modes: any regression in typed schema loading, asset readiness, or validation state blocks completion.
Q7 Negative Tests: focused tests must include validation-passing-parse broken fixtures and adapter catalog omissions.

## Inputs

- `src/animation/validation.rs`
- `src/animation/plugin.rs`
- `tests/anim_validation.rs`
- `tests/anim_asset_validation.rs`
- `tests/anim_graph_parse.rs`
- `tests/anim_graph_asset.rs`
- `tests/clip_parse.rs`
- `tests/clip_asset.rs`
- `tests/clip_geometry_parity.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test

## Observability Impact

Closeout verifies that typed diagnostics and validation state are sufficient to debug failures without UI or manual inspection.
