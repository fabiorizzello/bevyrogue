---
id: T02
parent: S03
milestone: M021
key_files:
  - tests/timeline_mode_parity.rs
key_decisions:
  - Used `World::try_query::<&Unit>()` (takes `&self`) instead of `World::query` (`&mut self`) to read component data inside predicate closures that only have `&World` via `SkillCtx::world`.
  - Used `normalize` that strips `cast_id` from `DealDamage` format to enable structural cross-mode comparison (each mode run uses a distinct `CastId`).
  - Did not drain pending through `intent_applier` between mode runs, per D006 — world HP state stays unchanged so all three mode runs see the same predicate result.
  - Two separate test functions (finisher branch + normal branch) prove the predicate is live and routes both ways deterministically.
duration: 
verification_result: passed
completed_at: 2026-05-15T08:48:02.231Z
blocker_discovered: false
---

# T02: Added mode-parity integration test for a branched timeline: Execute ≡ DryRun ≡ Preview on both finisher and normal branches, with a live world-reading predicate gate.

**Added mode-parity integration test for a branched timeline: Execute ≡ DryRun ≡ Preview on both finisher and normal branches, with a live world-reading predicate gate.**

## What Happened

Created `tests/timeline_mode_parity.rs` modeled on the chain_bolt port test. The timeline has a single entry `Impact` beat (hook enqueues `DealDamage` at `ENTRY_DAMAGE=50`) with two outgoing edges: edge A gated by `target_is_low_hp` predicate (reads `Unit.hp_current` from `ctx.world` via `World::try_query::<&Unit>()` — the canonical immutable-world query in Bevy 0.18; `World::query` requires `&mut World` which is unavailable in predicate context) routing to a `finisher` beat (`FINISHER_DAMAGE=200`), and edge B unconditional fallback routing to a `normal` beat (`NORMAL_DAMAGE=100`). Two test cases cover both branches: test 1 spawns target with `hp_current=10 < LOW_HP_THRESHOLD=30` (finisher branch); test 2 spawns target with `hp_current=100` (normal branch). Each test runs `run_to_completion` for Execute, DryRun, and Preview modes on the same world, then asserts `normalize(exec) == normalize(dry) == normalize(prev)` (the `normalize` helper strips `cast_id` from `DealDamage` intents for mode-independent structural comparison). Branch-routing assertions confirm the predicate is live, not dead. No world mutations occur between mode runs (per D006: pending queue is not drained through `intent_applier`). `World::try_query` was the key API discovery — it initialises a `QueryState` from `&self` rather than `&mut self`, enabling read-only world access inside predicate closures.

## Verification

`cargo test --test timeline_mode_parity 2>&1 | tail -5` — both test cases pass. Test 1 confirms finisher branch taken (amounts [50, 200]) across all three modes. Test 2 confirms normal branch taken (amounts [50, 100]) across all three modes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_mode_parity 2>&1 | tail -5` | 0 | PASS — 2 passed; 0 failed | 520ms |

## Deviations

None — followed plan exactly. `World::try_query` API discovery was not anticipated but is the correct Bevy 0.18 solution for immutable world access in predicate context.

## Known Issues

None.

## Files Created/Modified

- `tests/timeline_mode_parity.rs`
