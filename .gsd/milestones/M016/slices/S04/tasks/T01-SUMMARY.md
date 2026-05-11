---
id: T01
parent: S04
milestone: M016
key_files:
  - src/combat/blueprints/agumon.rs
  - src/combat/blueprints/gabumon.rs
  - src/combat/blueprints/mod.rs
key_decisions:
  - Mapped Agumon and Gabumon custom signals to Twin Core tag transitions in dedicated blueprints.
duration: 
verification_result: passed
completed_at: 2026-05-10T21:31:39.342Z
blocker_discovered: false
---

# T01: Create Agumon and Gabumon blueprints to handle Twin Core custom signals.

**Create Agumon and Gabumon blueprints to handle Twin Core custom signals.**

## What Happened

I created the Agumon and Gabumon blueprint modules in `src/combat/blueprints/`. 

Each blueprint implements a `dispatch` function that parses character-specific custom signals (`apply_heated`, `apply_chilled`, etc.) and converts them into `twin_core_added_tag_transition` emissions with the appropriate `TwinCoreDesignTag`. 

Finally, I registered both blueprints in `src/combat/blueprints/mod.rs` by adding them to the `BLUEPRINTS` registry, ensuring the shared signal dispatch layer can route signals from Agumon and Gabumon to their respective logic.

## Verification

Ran `cargo check` to ensure all new modules and registrations compile correctly. verified that all required signals (apply_heated, apply_meltdown_crack, apply_thermal_spark for Agumon; apply_chilled, apply_deep_crack, apply_thermal_spark for Gabumon) are handled.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ pass | 4430ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/agumon.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/mod.rs`
