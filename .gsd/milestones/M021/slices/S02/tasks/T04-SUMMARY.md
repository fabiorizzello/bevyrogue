---
id: T04
parent: S02
milestone: M021
key_files:
  - tests/timeline_onturnstart_kills.rs
  - tests/timeline_chain_bolt_port.rs
key_decisions:
  - Selector in chain_bolt test is world-unaware (returns hard-coded CHAIN_ORDER slice) — real selectors will query world in S03+; documented with comment.
  - cast_hit_set NoRepeat enforced inside the hook by skipping targets already in ctx.cast_hit_set, matching the spike design.
  - BeatEvent.hop_index drives the 0.8^n falloff computation inside the hook fn (formula hoisting deferred to S05 compiler as documented).
duration: 
verification_result: passed
completed_at: 2026-05-15T08:13:41.172Z
blocker_discovered: false
---

# T04: Added two integration tests (Gate 1 + Gate 3): fixture OnTurnStart kills target via BeatRunner/intent_applier, and chain_bolt 3-hop Loop timeline produces correct NoRepeat + 80% falloff damage ladder.

**Added two integration tests (Gate 1 + Gate 3): fixture OnTurnStart kills target via BeatRunner/intent_applier, and chain_bolt 3-hop Loop timeline produces correct NoRepeat + 80% falloff damage ladder.**

## What Happened

Both test files were already written by the prior session but gsd_task_complete had failed to persist (E0786 stale artifact in the build cache). After cleaning stale artifacts with `cargo clean -p bevyrogue`, both tests compiled and passed immediately:

- `tests/timeline_onturnstart_kills.rs` — single-beat Impact timeline drives BeatRunner::run_to_completion, the ko_target hook enqueues Intent::DealDamage{amount:9999}, the pending queue is transferred to IntentQueue, app.update() triggers intent_applier, and the test asserts enemy hp ≤ 0 plus an OnDamageDealt event with the matching cast_id.

- `tests/timeline_chain_bolt_port.rs` — a Loop timeline with a single-level body fires chain_bolt_hop three times. The selector (lowest_hp_pct_alive_norepeat) returns targets in ascending hp_pct order; the hook skips already-hit targets via cast_hit_set; damage is base * 0.8^hop_index (integer floor). Assertions cover target order [UnitId(12), UnitId(11), UnitId(10)], the 100→80→64 damage ladder, NoRepeat invariant, and 3 OnDamageDealt events post-update.

Full suite: 0 failures across all integration tests.

## Verification

cargo test --test timeline_onturnstart_kills: 1 passed. cargo test --test timeline_chain_bolt_port: 1 passed. cargo test (full suite): 0 failures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_onturnstart_kills` | 0 | pass | 8070ms |
| 2 | `cargo test --test timeline_chain_bolt_port` | 0 | pass | 460ms |
| 3 | `cargo test` | 0 | pass | 12000ms |

## Deviations

none — test files were fully authored by the prior session; this session only re-ran verification after cleaning a stale build artifact.

## Known Issues

none

## Files Created/Modified

- `tests/timeline_onturnstart_kills.rs`
- `tests/timeline_chain_bolt_port.rs`
