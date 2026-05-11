---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T01: Refactor Turn System to Action Value (AV) Model

The current `VecDeque` turn system is incompatible with the design for `Delay` effects. An Action Value (AV) system is required to correctly implement scaling resistance and thresholds. This task performs the foundational refactoring of the turn system to an AV-based model as a prerequisite for the slice's main features. A design decision will be made and documented for the `MAX_AV` constant, likely 10000 based on HSR.

## Inputs

- `docs/combat_design.md`
- `src/combat/turn_order.rs`
- `src/combat/turn_system/mod.rs`

## Expected Output

- `src/combat/av.rs`
- `src/combat/turn_order.rs`
- `src/combat/turn_system/mod.rs`
- `tests/turn_system_av.rs`

## Verification

cargo test --test turn_system_av -- --nocapture
