---
id: T04
parent: S03
milestone: M021
key_files:
  - tests/timeline_circuit_breaker.rs
key_decisions:
  - Exact pending count is MAX_HOPS (256): breaker fires at hop_index==256 BEFORE executing the 257th body, so body fires for hop_index 0..=255 = 256 times, one DealDamage per hop; documented inline in test comment.
  - Selector returns primary_target rather than hardcoding a second unit — the hook still needs a beat_targets entry to enqueue, but the full NoRepeat dedup used in chain_bolt is not needed here.
duration: 
verification_result: passed
completed_at: 2026-05-15T08:49:53.499Z
blocker_discovered: false
---

# T04: Added circuit-breaker integration test: Loop with never-true exit_when halts at MAX_HOPS=256 with bounded DealDamage stream

**Added circuit-breaker integration test: Loop with never-true exit_when halts at MAX_HOPS=256 with bounded DealDamage stream**

## What Happened

Created `tests/timeline_circuit_breaker.rs` as a realistic integration test for F4. The fixture spawns a real Bevy world with `intent_applier` wired (mirroring `timeline_chain_bolt_port`), registers a Loop timeline whose single Impact body beat enqueues one `DealDamage` per hop via `cb/one_damage_per_hop`, and an `exit_when` predicate `cb/never` that always returns false. Running `run_to_completion` with `max_steps=1000` (well above MAX_HOPS=256) exercises the circuit-breaker path. Assertions: outcome==StepOutcome::Halted; pending contains exactly 256 DealDamage intents (body fires for hop_index 0..=255, breaker trips at hop_index=256 before the 257th body execution); total pending is bounded (≤MAX_HOPS+1); draining through intent_applier yields exactly 256 OnDamageDealt events with no panic. The comment documents that T01's bevy::log::warn! is also emitted on Halt, but capturing log output is out of scope — StepOutcome::Halted is the integration contract.

## Verification

cargo test --test timeline_circuit_breaker: 1 passed, 0 failed, no warnings in the test file, finished in 0.00s

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_circuit_breaker 2>&1 | tail -10` | 0 | pass — loop_never_exit_halts_at_max_hops ok, MAX_HOPS=256 intents bounded, no panic | 550ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/timeline_circuit_breaker.rs`
