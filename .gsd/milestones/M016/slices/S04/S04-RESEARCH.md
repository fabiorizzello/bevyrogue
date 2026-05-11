# S04 — Research

**Date:** 2025-05-10

## Summary

Slice S04 is the final migration slice for M016, moving the Twin Core logic for Agumon and Gabumon into their respective blueprints. Currently, `TwinCoreHook` in `src/combat/twin_core.rs` responds to tag additions (like `TAG_HEATED` or `TAG_CHILLED`), but these tags are currently only tested in isolated tests and aren't connected to actual gameplay definitions in `assets/data/skills.ron`. 

The goal is to update `skills.ron` to add `custom_signals` for Agumon and Gabumon's skills (e.g., `pepper_breath`, `agumon_ult`, `bubble_blast`, `gabumon_ult`) and create two new blueprints: `src/combat/blueprints/agumon.rs` and `src/combat/blueprints/gabumon.rs`. These blueprints will interpret the signals and emit the corresponding `CombatKernelTransition` objects.

## Recommendation

We should introduce `custom_signals` on Agumon and Gabumon skills. The new blueprints (`agumon.rs` and `gabumon.rs`) should interpret these signals and emit `twin_core_added_tag_transition(...)` (which adds the appropriate `TwinCoreDesignTag`). This keeps `TwinCoreHook` functional without modification, as it already listens for these tags.

Alternatively, the blueprints could directly emit `CombatKernelTransition::TwinCore(...)`, but utilizing the existing tag hooks maintains the design that tags act as the bridge to Twin Core states (e.g., Heated, Chilled, ThermalSpark) which are also used elsewhere.

## Implementation Landscape

### Key Files

- `assets/data/skills.ron` — Update Agumon (`pepper_breath`, `agumon_follow_up`, `agumon_ult`) and Gabumon (`bubble_blast`, `gabumon_follow_up`, `gabumon_ult`) skills to include `custom_signals` mapping to their elemental applications.
- `src/combat/blueprints/agumon.rs` — New file. Dispatch function handles `"agumon"` signals (e.g., `apply_heated`, `apply_meltdown_crack`) and translates them to `CombatKernelTransition` tag additions.
- `src/combat/blueprints/gabumon.rs` — New file. Dispatch function handles `"gabumon"` signals (e.g., `apply_chilled`, `apply_deep_crack`).
- `src/combat/blueprints/mod.rs` — Register `agumon::dispatch` and `gabumon::dispatch` into the main blueprint dispatcher.

### Build Order

1. **Create Blueprints:** Create `agumon.rs` and `gabumon.rs` defining the custom signal strings and the mappings to `twin_core_added_tag_transition`.
2. **Register Blueprints:** Add them to `src/combat/blueprints/mod.rs`.
3. **Update RON:** Add `custom_signals` arrays to the corresponding skills in `assets/data/skills.ron`.
4. **Validation:** Run existing `twin_core_integration` tests, and add any specific action tests if missing.

### Verification Approach

- Ensure that `cargo test --no-fail-fast` remains green.
- Verify through `combat_cli` proof: `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` to confirm `ValidationSnapshot` and `CombatEvent` structures correctly record `TwinCore` state changes when Agumon and Gabumon act.

## Constraints

- **RON is Declarative Only:** Only use `custom_signals` like `(owner: "agumon", signal: "apply_heated")`. Do not encode execution rules directly in data.
- **Generic Kernel:** `twin_core.rs` already safely processes these through its hook; ensure no Agumon/Gabumon specific ID checks exist in `turn_system.rs` or `resolution.rs`.
