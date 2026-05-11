
# S07: Toughness 3 categorie (Standard/Armored/Shielded) + Break Seal — Research

**Date:** 2026-04-27

## Summary

This research outlines the implementation for introducing three categories of Toughness (Standard, Armored, Shielded) and a "Break Seal" mechanic as per requirement R079. The current `Toughness` component is a simple gauge that breaks on a weak hit when its value crosses zero. The proposed changes will extend this system to differentiate enemy defensive archetypes and introduce a turn-based rhythm to the break mechanic.

## Recommendation

The implementation will follow the existing ECS patterns. A new `ToughnessCategory` enum will be added to the `Toughness` component and `UnitDef` data structure. A new `RoundFlags` component will be created to manage the per-turn `break_sealed` state, aligning with the architectural direction from decision D045. The core logic change will be in `Toughness::apply_hit` and its call site in `resolution.rs` to respect the new categories and the break seal.

## Implementation Landscape

### Key Files

-   `src/combat/toughness.rs`: To define `ToughnessCategory` enum and add it to the `Toughness` component. The `apply_hit` function will be modified to incorporate the new logic for categories and the break seal.
-   `src/data/units_ron.rs`: To add `toughness_category: Option<ToughnessCategory>` to `UnitDef`, allowing it to be specified in RON files.
-   `src/combat/bootstrap.rs`: To update `spawn_unit_from_def` to initialize the `Toughness` component with its category and to spawn the new `RoundFlags` component for each unit.
-   `src/combat/turn_system/mod.rs` (or a new file): To define the `RoundFlags` component and a `reset_round_flags_system` that runs at the end of each round to clear the `break_sealed` flag.
-   `src/combat/resolution.rs`: The `apply_effects` function will be updated to query for the defender's `RoundFlags`, pass the `break_sealed` status to `apply_hit`, and set the flag to `true` when a break occurs.
-   `assets/data/units.ron`: To be updated with `toughness_category` fields for some enemies to enable testing.

### Build Order

1.  **Component & Type Definitions**: Define the `ToughnessCategory` enum and the new `RoundFlags { break_sealed: bool }` component. This provides the foundational data structures.
2.  **Data Layer Integration**: Update `UnitDef` in `units_ron.rs` and the `Toughness` component in `toughness.rs` to include the new category.
3.  **Entity Spawning**: Modify `bootstrap.rs` to correctly initialize the `Toughness` category and add the `RoundFlags` component to newly spawned units.
4.  **Core Logic**: Update the `Toughness::apply_hit` function to check for the `break_sealed` flag and to handle the `Shielded` category (by preventing breaks). Update the call site in `resolution.rs` to pass the flag and update it on break.
5.  **State Reset**: Implement the `reset_round_flags_system` and schedule it to run at the end of each round to ensure the seal is temporary.
6.  **Testing**: Create a new integration test file (`tests/toughness_categories.rs`) to verify the new mechanics end-to-end.

### Verification Approach

-   A unit test in `toughness.rs` will assert that a `Toughness` component with category `Shielded` never enters the `broken = true` state from a normal `ToughnessHit`.
-   A unit test will verify that `apply_hit` returns `false` if the unit is already break-sealed.
-   A new integration test (`tests/toughness_categories.rs`) will cover the full loop:
    1.  Define an `Armored` enemy (high toughness) and a `Standard` enemy.
    2.  Verify the `Armored` enemy requires more hits to break.
    3.  Break an enemy and confirm an `OnBreak` event is dispatched.
    4.  In the same turn, hit the broken enemy again with an attack that would normally cause a break. Assert that no new `OnBreak` event is fired.
    5.  Advance the turn/round.
    6.  Confirm the break seal is lifted by breaking the same enemy again (after restoring its toughness for the test).
