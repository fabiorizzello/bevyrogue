---
id: T01
parent: S02
milestone: M018
key_files:
  - src/combat/unit.rs
  - src/combat/bootstrap.rs
  - tests/slot_index_tiebreak.rs
key_decisions:
  - SlotIndex inserted post-spawn via commands.entity().insert() in apply_composition, keeping spawn_unit_from_def signature stable — avoids breaking 6+ existing test callers that spawn without slot context
  - SlotIndex derives Ord+Hash to enable sorted query consumers (per task plan warning about non-deterministic Bevy query order)
duration: 
verification_result: passed
completed_at: 2026-05-13T16:07:21.791Z
blocker_discovered: false
---

# T01: Added SlotIndex(u8) Component and wired per-team insertion-order assignment into apply_composition

**Added SlotIndex(u8) Component and wired per-team insertion-order assignment into apply_composition**

## What Happened

Declared `SlotIndex(u8)` as a new Bevy Component in `src/combat/unit.rs` (derives Component, Copy, Ord, Hash — stable, never mutated). Updated `apply_composition` in `src/combat/bootstrap.rs` to enumerate both the allies and enemies loops independently, inserting `SlotIndex(idx as u8)` on each spawned entity via `commands.entity(entity).insert(...)`. This approach keeps `spawn_unit_from_def` signature unchanged (avoiding breakage across 6+ existing test callers that don't need slot context) — minor deviation from the literal task wording but cleaner composition boundary. Integration test `tests/slot_index_tiebreak.rs` asserts two properties: (1) after a 3v3 apply_composition each team's slot set equals {0,1,2}; (2) slots are unique per team. Both pass. `cargo check` clean (81 pre-existing warnings, zero new errors).

## Verification

cargo test --test slot_index_tiebreak: 2 passed. cargo check: Finished dev profile, 0 errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test slot_index_tiebreak 2>&1 | grep -E '(test result|FAILED)'` | 0 | 2 passed, 0 failed | 4200ms |
| 2 | `cargo check 2>&1 | tail -3` | 0 | Finished dev profile, 0 errors | 2260ms |

## Deviations

spawn_unit_from_def signature not changed (task plan said 'pass SlotIndex into spawn_unit_from_def'). SlotIndex inserted post-spawn by apply_composition caller instead. Semantically equivalent; avoids cascading signature changes across 6+ test files.

## Known Issues

None.

## Files Created/Modified

- `src/combat/unit.rs`
- `src/combat/bootstrap.rs`
- `tests/slot_index_tiebreak.rs`
