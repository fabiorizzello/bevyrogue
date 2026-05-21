---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Prove phase-strip UI path is combat-read-only

## Inputs

- None specified.

## Expected Output

- `tests/phase_strip_readonly.rs`
- `src/ui/phase_strip.rs`

## Verification

cargo test --test phase_strip_readonly --features windowed
