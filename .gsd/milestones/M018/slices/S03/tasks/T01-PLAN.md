---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: BounceSelector + RepeatPolicy DSL enums + selector dispatcher (refactor of prior next_bounce_hop)

Introduce two enums in src/data/skills_ron.rs: `BounceSelector` (variants: LowestHpPctAlive, NextSlotAlive, AdjLowest — extensible; serde-derived) and `RepeatPolicy` (NoRepeat, AllowRepeat). In src/combat/resolution.rs, replace the previously-landed hardcoded `next_bounce_hop` with a dispatcher `select_bounce_hop(selector: BounceSelector, snapshot: &TargetableSnapshot, already_hit: &HashSet<UnitId>, enemy_team: Team, last_target_slot: Option<u8>) -> Option<UnitId>` that pattern-matches the selector and calls a small pure fn per variant. LowestHpPctAlive logic moves into `select_lowest_hp_pct_alive` (existing integer per-mille math + slot_index asc tie-break preserved). Add `select_next_slot_alive` (lowest slot_index > last_target_slot among alive enemies not in already_hit) and `select_adj_lowest` (alive enemy with |slot - last_target_slot| <= 1 by lowest HP%, slot tie-break). All variants must honor already_hit when the in-effect RepeatPolicy is NoRepeat; the dispatcher receives the policy from the caller and skips the already_hit filter when AllowRepeat. Keep helpers total — no panics. Migrate existing table-driven tests for next_bounce_hop to the new dispatcher; add per-variant tests including AllowRepeat case (same target picked twice when policy allows). Do not yet change TargetShape::Bounce schema — that's T02.

## Inputs

- `existing next_bounce_hop fn from commit af09d40`
- `TargetableSnapshot from S02`

## Expected Output

- `BounceSelector + RepeatPolicy enums in skills_ron.rs`
- `select_bounce_hop dispatcher in resolution.rs`
- `select_lowest_hp_pct_alive / select_next_slot_alive / select_adj_lowest pure fns`
- `per-variant unit tests including AllowRepeat case`

## Verification

cargo test --lib resolution::tests && cargo check
