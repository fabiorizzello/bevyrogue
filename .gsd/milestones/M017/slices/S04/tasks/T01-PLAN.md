---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Paralyzed skip-turn in process_turn_advanced_system

Add Paralyzed always-skip semantic to the per-TurnAdvanced handler in `src/combat/turn_system/mod.rs`. Mirror the existing Stunned short-circuit at L495–502: detect Paralyzed presence in the unit's StatusBag inside the mutable-borrow block (via `bag.has(&StatusEffectKind::Paralyzed)`), tick the bag normally so duration decrements, then `continue` to skip action dispatch. Extend the `Snap` struct with `is_paralyzed: bool` populated from the StatusBag at snapshot time so the enemy-AI dispatch gate at L547 can be widened to `&& !shock_cancelled && !snap.is_stunned && !snap.is_paralyzed`. Emit `CombatEventKind::OnActionFailed { reason: "paralyzed" }` (or the closest existing variant if `OnActionFailed` does not carry a reason — fall back to the generic skip-emission used by Stunned) so tests can count skips deterministically. Order: Heated DoT (existing) runs first, then Stunned check, then Paralyzed check + tick, then `continue`. Do NOT alter the Stunned branch.

## Inputs

- `src/combat/turn_system/mod.rs`
- `src/combat/status_effect.rs`
- `.gsd/milestones/M017/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/turn_system/mod.rs`

## Verification

cargo check && cargo test --lib turn_system
