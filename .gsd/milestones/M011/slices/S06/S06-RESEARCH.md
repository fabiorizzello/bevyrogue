# S06 — Research

**Date:** 2026-04-27

## Summary

The goal of this slice is to implement Tempo Resistance for bosses and a Minimum Action Threshold for `Delay` effects, as per requirement R078 and the v5.3 combat design document. The design specifies an Action Value (AV) based turn system where `Delay` pushes a unit's AV back.

However, a critical discrepancy exists: the current implementation in `src/combat/turn_order.rs` uses a simple `VecDeque` for turn order, not an AV system. Effects like `TurnAdvance` (the implementation of `Delay`) manipulate the queue order directly, rather than modifying an AV score. The `Speed` stat exists but is not used to determine turn order.

This architectural drift makes it impossible to correctly implement the "Minimum Action Threshold" (which requires a value to threshold) and the scaling resistance. The slice is blocked until the turn system is refactored to align with the AV-based design specified in `docs/combat_design.md`.

## Recommendation

This slice should be broken into two distinct, sequential phases: a prerequisite refactoring phase, followed by the feature implementation phase.

**Phase 1: Refactor Turn System to an Action Value model.**
This is a foundational change that unblocks S06.
1.  Introduce a new `ActionValue(u32)` component for all combat units.
2.  Replace `TurnOrder` resource's `VecDeque` with a sorted list of units based on their `ActionValue`. The unit with the lowest AV is next to act.
3.  Refactor `advance_turn_system`. Instead of rotating a queue, it should find the next actor (lowest AV), let them act, and then calculate their next turn's AV based on their `Speed` stat (e.g., `next_av = current_av + (MAX_AV / speed)`).
4.  Refactor the handler for the `TurnAdvance` event. Instead of manipulating the queue, it should now directly add to or subtract from a unit's `ActionValue` component. `Delay` increases AV, `Self-Advance` decreases it.

**Phase 2: Implement S06 Features.**
Once the AV system from Phase 1 is in place:
1.  Introduce a new `TempoResistance` component. This component will be added to boss/elite units and will track the number of `Delay` effects they have received.
2.  In the `TurnAdvance` event handler, if the target has a `TempoResistance` component, check the counter. Apply the resistance multiplier (1.0, 0.5, 0.25) to the incoming `Delay` amount before adding it to the unit's `ActionValue`. Increment the counter.
3.  Implement the Minimum Action Threshold within the same `TurnAdvance` handler. After calculating the new `ActionValue` (with resistance), clamp it to a maximum value defined as a constant (e.g., `MIN_ACTION_THRESHOLD_AV`). This ensures a unit can't be delayed indefinitely.

## Implementation Landscape

### Key Files

*   **Phase 1 (Refactor):**
    *   `src/combat/turn_order.rs`: The `TurnOrder` resource needs a complete overhaul from `VecDeque` to an AV-based structure. `advance_unit_pct` will be removed.
    *   `src/combat/turn_system/mod.rs`: `advance_turn_system` needs to be rewritten to manage AV. The `TurnAdvance` event is likely handled here or in `pipeline.rs`.
    *   `src/combat/unit.rs` (or new `av.rs`): To define the new `ActionValue` component.
    *   `src/combat/bootstrap.rs`: To initialize units with a starting `ActionValue`.
*   **Phase 2 (S06 Features):**
    *   `src/combat/resistance.rs` (new file): To define the `TempoResistance` component.
    *   `src/combat/turn_system/mod.rs`: The `TurnAdvance` handler needs to be modified to include resistance and threshold logic.
    *   `assets/data/units.ron`: To add the `TempoResistance` component to boss units.

### Build Order

The turn system refactoring (Phase 1) is a mandatory prerequisite and must be completed first. It fundamentally changes how turns are processed and unblocks the actual implementation of tempo resistance. Attempting to implement S06 on the current `VecDeque` system would result in a fragile, incorrect implementation that doesn't meet the design intent and would need to be thrown away later.

### Verification Approach

1.  **Phase 1:** Create a new integration test in `tests/turn_system.rs` that seeds units with different `Speed` values and verifies that higher speed units get more turns over a 10-round simulation. It should also test that `TurnAdvance` events correctly modify the `ActionValue` of units.
2.  **Phase 2:** Create a new integration test in `tests/tempo_resistance.rs`. This test will simulate applying a `Delay` skill multiple times to a boss unit.
    *   Assert that the first `Delay` applies 100% of its AV modification.
    *   Assert that the second `Delay` applies 50%.
    *   Assert that the third and subsequent `Delay`s apply 25%.
    *   Assert that a unit's `ActionValue` cannot be pushed above the `MIN_ACTION_THRESHOLD_AV` constant.

## Open Risks

*   **Scope Creep:** The turn system refactor is non-trivial. It will touch core logic and may have unforeseen knock-on effects on existing tests or systems that assume the `VecDeque` implementation (e.g., `Stunned` logic, AI action picking). This risk is high.
*   **AV Model Details:** The exact mechanics of the AV system (e.g., how AV is calculated from speed, what the max AV value is) are not specified in detail. The implementation will need to make reasonable assumptions based on common JRPG patterns (like Honkai: Star Rail's 10,000 AV model). This requires a design decision to be made and documented.
