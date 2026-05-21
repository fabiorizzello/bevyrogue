---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T04: Close R013 dead-target mid-loop and regression sweep

Ensure the target-dead-mid-loop test asserts the presentation/runtime flow does not branch on liveness and that the event log makes the overshoot/death state inspectable. Run focused R009/R013 tests plus the previously affected windowed/headless suites to prove S08 did not regress S01-S07.

## Inputs

- `tests/timeline/r013_failure_visibility.rs`
- `tests/animation.rs`
- `tests/timeline.rs`

## Expected Output

- `tests/timeline/r013_failure_visibility.rs`
- `tests/timeline.rs`
- `.gsd/REQUIREMENTS.md`

## Verification

cargo test --features windowed --test animation --test timeline --test windowed_only

## Observability Impact

Confirms failure evidence is durable through tests and prepares requirement validation notes for slice closeout.
