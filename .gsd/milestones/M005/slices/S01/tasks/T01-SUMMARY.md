---
id: T01
parent: S01
milestone: M005
key_files:
  - src/animation/reaction.rs
  - src/animation/mod.rs
  - tests/animation/stance_reaction_mapping.rs
  - tests/animation.rs
key_decisions:
  - stance_reaction_for uses a fully-enumerated explicit match over CombatEventKind (no _ catch-all) so future event variants must be classified deliberately
  - resolve_stance_reaction encodes death-precedence with short-circuit return on first Death, satisfying the slice requirement that a unit that died in the same window it was struck plays death not hurt
  - StanceReaction is an independent closed enum (only Hurt/Death) rather than mirroring the open event taxonomy, keeping the reaction vocabulary minimal for S02 to consume the Death role
duration: 
verification_result: passed
completed_at: 2026-05-26T08:11:24.976Z
blocker_discovered: false
---

# T01: Added pure event-to-stance-reaction lib mapping (Hurt/Death + death-precedence) with four headless tests

**Added pure event-to-stance-reaction lib mapping (Hurt/Death + death-precedence) with four headless tests**

## What Happened

Created `src/animation/reaction.rs` defining a closed `StanceReaction` enum (`Hurt`, `Death`) and three pure functions: `stance_reaction_for(&CombatEventKind) -> Option<StanceReaction>` maps `OnHitTaken` → Hurt, `UnitDied` → Death, and every other variant → None via an explicit fully-enumerated match (no catch-all, so a new event variant forces a compile error here); `resolve_stance_reaction<'a>(impl IntoIterator<Item = &'a CombatEventKind>)` encodes death-precedence by returning Death on first Death hit, else Hurt if any Hurt was seen, else None; and `impl StanceReaction::stance_node(self) -> NodeId` returning `NodeId("hurt")` / `NodeId("death")` matching the node names in assets/digimon/agumon/stance.ron. Imports `CombatEventKind` from `crate::combat::observability::events` and `NodeId` from `crate::animation::anim_graph`. Registered the module in `src/animation/mod.rs` (`pub mod reaction;` + `pub use reaction::*;`). Added `tests/animation/stance_reaction_mapping.rs` with the four required cases (hit→Hurt+node, death→Death+node, mixed batch→Death precedence, non-reaction kind + empty batch→None) linking only against the `bevyrogue` lib crate, and registered it in `tests/animation.rs`. Mapping has no windowed/bevy-render dependency, so S02 can consume the Death role without touching the lib mapping.

## Verification

Ran `cargo test --test animation stance_reaction_mapping` — all 4 new cases green. Ran full `cargo test --test animation` — 119 passed, 0 failed. `cargo clippy --lib` produced no findings on reaction.rs. Compilation succeeds headless (default profile, no windowed feature), confirming the mapping lives entirely in the lib with no bevy-render dependency.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation stance_reaction_mapping` | 0 | pass | 8000ms |
| 2 | `cargo test --test animation` | 0 | pass | 2000ms |
| 3 | `cargo clippy --lib` | 0 | pass | 3000ms |

## Deviations

For the no-op test case the plan suggested OnSkillCast or OnActionResolved; used OnActionResolved (a unit variant) to avoid constructing a SkillId fixture. Functionally equivalent and still exercises a representative non-reaction kind.

## Known Issues

none

## Files Created/Modified

- `src/animation/reaction.rs`
- `src/animation/mod.rs`
- `tests/animation/stance_reaction_mapping.rs`
- `tests/animation.rs`
