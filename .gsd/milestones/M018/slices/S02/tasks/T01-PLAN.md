---
estimated_steps: 6
estimated_files: 3
skills_used: []
---

# T01: Introduce SlotIndex Component and wire it into spawn_unit_from_def

Declare a new SlotIndex(u8) Bevy Component (per-team scoped, paired with the existing Team component for global uniqueness) and assign it at spawn time. apply_composition iterates per-team allies/enemies; convert those loops to enumerate() and pass SlotIndex(idx as u8) into spawn_unit_from_def. SlotIndex must be stable across the encounter — never mutated, survives Revive. Add an integration test asserting that for a 3-ally / 3-enemy encounter each team's slot range is exactly {0,1,2}.

**Locked decisions (from research, autonomous mode):**
- Slot representation: new SlotIndex(u8) Component (NOT a UnitId field, NOT a HashMap lookup) — durable across encounter, decoupled from UnitId.
- Scope: per-team — (Team, SlotIndex) together give global uniqueness.
- Source: insertion order in apply_composition (not RON-declared, not stat-based).

**Do not hand-roll:** Bevy Query iteration order is not deterministic; never read SlotIndex by relying on query order. Always sort consumers by (Team, SlotIndex) explicitly.

## Inputs

- `src/combat/bootstrap.rs`
- `src/combat/unit.rs`
- `src/combat/team.rs`

## Expected Output

- `src/combat/unit.rs`
- `src/combat/bootstrap.rs`
- `tests/slot_index_tiebreak.rs`

## Verification

cargo test --test slot_index_tiebreak 2>&1 | grep -E '(test result|FAILED)' && cargo check 2>&1 | tail -5
