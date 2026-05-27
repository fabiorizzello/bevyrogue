---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Warn-once on cast cue spawn miss

Reuse the S06 deduplicated warn helper so a cast cue with no registered effect logs once with the cue id instead of silently spawning nothing.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/digimon/mod.rs`

## Expected Output

- `Unregistered cast cue warns once with cue id; registered cues silent`

## Verification

cargo test --features windowed --test windowed_only (no warn on happy path); manual winx clean for registered cues
