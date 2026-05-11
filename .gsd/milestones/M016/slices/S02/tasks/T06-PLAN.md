---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T06: Refresh predator-loop kernel snapshot fixture

Update `tests/predator_loop_kernel.rs` so its `ValidationSnapshot` fixture matches the current observability struct shape, including the battery-loop field now present on the snapshot. Preserve the existing intent of proving predator-loop event serialization and snapshot readability.

## Inputs

- `tests/predator_loop_kernel.rs`
- `src/combat/observability.rs`

## Expected Output

- `predator_loop_kernel test passes against current ValidationSnapshot shape`

## Verification

cargo test --test predator_loop_kernel --no-fail-fast
