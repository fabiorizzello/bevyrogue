---
id: T01
parent: S03
milestone: M018
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
key_decisions:
  - RepeatPolicy filter is applied in the shared candidate-pool construction step (before dispatching to per-variant helpers), keeping all three helpers policy-agnostic and total.
  - AdjLowest falls back to select_lowest_hp_pct_alive across all candidates when last_slot is None, matching the spec's 'degrade gracefully' requirement.
  - Dispatcher parameter order puts policy before enemy_team to mirror the conceptual layering: policy is a chain-level concern, enemy_team is a snapshot-filter concern.
duration: 
verification_result: passed
completed_at: 2026-05-13T21:38:18.896Z
blocker_discovered: false
---

# T01: BounceSelector + RepeatPolicy DSL enums + select_bounce_hop dispatcher fully implemented and tested in resolution.rs

**BounceSelector + RepeatPolicy DSL enums + select_bounce_hop dispatcher fully implemented and tested in resolution.rs**

## What Happened

T01 was already fully implemented as part of T02's earlier work (T02 touched both skills_ron.rs and resolution.rs). On inspection, all three deliverables were in place:

1. **BounceSelector enum** (LowestHpPctAlive, NextSlotAlive, AdjLowest) — declared in src/data/skills_ron.rs with Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize.

2. **RepeatPolicy enum** (NoRepeat, AllowRepeat) — declared in the same file with the same derives.

3. **select_bounce_hop dispatcher** — in src/combat/resolution.rs, with signature `select_bounce_hop(selector: BounceSelector, snapshot: &TargetableSnapshot, already_hit: &HashSet<UnitId>, policy: RepeatPolicy, enemy_team: Team, last_target_slot: Option<u8>) -> Option<UnitId>`. Pattern-matches the selector and dispatches to three pure helper fns:
   - `select_lowest_hp_pct_alive`: integer per-mille math with slot_index asc tie-break
   - `select_next_slot_alive`: lowest slot_index strictly > last_slot (degrades to lowest slot when None)
   - `select_adj_lowest`: candidates with |slot - last| <= 1, picking lowest HP%; falls back to global lowest when last_slot is None

4. **RepeatPolicy filter** applied in the dispatcher's candidate-pool construction: AllowRepeat skips the already_hit filter entirely.

5. **Tests** — 13 table-driven dispatcher tests covering all variants, tiebreaks, empty-pool, NoRepeat exclusion, and AllowRepeat repick-same-target scenarios. All 39 resolution::tests pass.

## Verification

Ran `cargo test --lib resolution::tests` — 39 tests pass, 0 failures. Ran `cargo check` — compiles cleanly with only pre-existing unrelated warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib resolution::tests` | 0 | 39 passed, 0 failed | 4200ms |
| 2 | `cargo check` | 0 | Finished dev profile, no errors | 1440ms |

## Deviations

None. Implementation was complete before this task execution started (landed as part of T02 cross-file work). Verified correctness and test coverage match the plan exactly.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
