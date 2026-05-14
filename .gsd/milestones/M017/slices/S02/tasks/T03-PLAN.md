---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Migrate tick + expiration to StatusBag

## Inputs

- None specified.

## Expected Output

- `src/combat/turn_system/mod.rs`

## Verification

`cargo check` clean. The tick system emits exactly one `OnStatusExpired` per expired instance (verified later by T05 tests).
