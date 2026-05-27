---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Resolve idle-only-vs-hurt design call and document it

Decide and implement whether Renamon's non-idle reactions (hurt/death) use shared engine reaction defaults or species-specific data, per the open design question from spike 2. Record the choice with gsd_save_decision and ensure the chosen behavior is covered by the windowed reaction path.

## Inputs

- `src/animation/reaction.rs`
- `src/windowed/digimon/renamon/mod.rs`

## Expected Output

- `Renamon reaction behavior implemented and a decision recorded for the idle-only-vs-hurt call`

## Verification

cargo test --features windowed --test windowed_only (reaction behavior covered); cargo test (headless green)
