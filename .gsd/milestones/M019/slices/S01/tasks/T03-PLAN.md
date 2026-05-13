---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Wire DrBag through resolution.rs call sites + per-turn tick

Update the two `calculate_damage` call sites in `src/combat/resolution.rs` (~line 478 and ~line 636) to fetch the defender's `DrBag` from the world using the same query pattern that already reads `defender_status`/`StatusBag`, and pass `Option<&DrBag>` into `calculate_damage`. Then extend the per-turn tick block in `src/combat/turn_system/mod.rs` (lines 518 and 569 — both spots that call `bag.tick_all()` on `StatusBag`) so that the matching `DrBag` for the same entity also has `tick_all()` called and any drop count is logged through the existing log seam (or simply discarded — match what `StatusBag::tick_all` does). Do not introduce new events or change `OnDamageDealt` payload.

## Inputs

- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/damage.rs`
- `src/combat/buffs.rs`

## Expected Output

- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`

## Verification

cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_breakdown_log
