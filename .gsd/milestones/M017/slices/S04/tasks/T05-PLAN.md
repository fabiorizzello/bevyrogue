---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T05: Full-suite verification and grep guard

Run the full headless verification: `cargo check`, `cargo test` (entire integration suite). Confirm zero failures, zero ignored. Re-run the S01 grep guard: ensure no occurrences of `Burn|Freeze|Shock|DeepFreeze` in `src/` and `tests/` outside the reserved-variant declarations in `src/combat/status_effect.rs` and `src/data/skills_ron.rs`. Capture exit codes and a brief evidence summary into the task verification record. Pure verification task — no source files modified.

## Inputs

- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/status_paralyzed_skip.rs`
- `tests/status_slowed_delay.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo check && cargo test && grep -rn -E 'Burn|Freeze|Shock|DeepFreeze' src/ tests/ | grep -v 'reserved' | wc -l
