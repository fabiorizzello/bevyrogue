---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Break up advance_digimon_presentation

Decompose the advance_digimon_presentation function into named per-concern steps (e.g. tick clock, advance playback, release barriers, drive feedback) so each is independently readable and testable. Behavior identical.

## Inputs

- `src/windowed/render.rs`

## Expected Output

- `advance_digimon_presentation split into named per-concern functions, behavior unchanged`

## Verification

cargo test --features windowed --test windowed_only (green); cargo test (headless green)
