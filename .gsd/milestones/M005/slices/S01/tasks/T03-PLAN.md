---
estimated_steps: 3
estimated_files: 3
skills_used: []
---

# T03: Regression sweep: headless and windowed test suites green, no windowed dep leak

Why: The milestone contract requires the full headless suite, the windowed test suite, and the windowed build to stay green after the reaction wiring, and that the new lib mapping introduces no windowed/render dependency into the headless build (R002/R005).

Do: Run the full headless suite (`cargo test`), the windowed test suite (`cargo test --features windowed`), and the windowed build (`cargo build --features windowed`). Confirm all three are green. Confirm the new `src/animation/reaction.rs` module compiles into the default (headless) lib build — i.e. it pulls in no `#[cfg(feature = "windowed")]`-gated symbols — which a green default `cargo test` already proves since tests/ link only the headless lib. If any command fails, fix the regression (do not weaken or delete the new tests to make it pass).

Done when: all three commands exit 0 and the new stance-reaction tests remain present and passing.

## Inputs

- `src/animation/reaction.rs`
- `src/windowed/render.rs`
- `tests/animation/stance_reaction_mapping.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test --features windowed
