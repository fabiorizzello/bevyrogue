---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T01: Remove D026 engine re-entrancy guard (D046) and rewrite chain semantics test

Delete the `if event.follow_up_depth >= 1 { return Err(OneHopSuppressed) }` guard in `src/combat/follow_up.rs:161-163` and the `OneHopSuppressed` variant from `FollowUpSkipReason` (line ~66). This is the smallest surgical change in the slice and unblocks the test rewrite. Leave `event.follow_up_depth >= 1` in `src/combat/ultimate.rs:77` intact — that line is a DIFFERENT semantic (OnAllyFollowUp ult-charge gating: 'is this skill cast a follow-up?') and must be preserved. Update the doc comment on `CombatEvent.follow_up_depth` in `src/combat/events.rs:76-78` to reflect D046 (chains bounded by data, not engine). Remove the `OneHopSuppressed` test case from `src/combat/follow_up_tests.rs` (around lines 341-367 — search for the variant). Rewrite `tests/follow_up_reentrancy.rs` as `tests/follow_up_chains.rs` (delete old, create new) asserting the OPPOSITE of the old behavior: a follow-up emitted at depth 1 CAN trigger another follow-up at depth 2, and chains terminate naturally when no follower's preconditions re-trigger. Use the existing MVP roster — Greymon's follow-up does not re-emit OnEnemyBreak, so a chain Greymon→Greymon does not loop. Assert depth-2 events can appear and that the chain terminates after a finite number of steps. Trust MEM029: drain the event cursor between app.update() calls to avoid Messages ring-buffer pruning.

**Failure modes:** The grep target for the guard removal must be precise — mass-deleting all `follow_up_depth` references will break ultimate charging for Renamon (UltAccumulationTrigger::OnAllyFollowUp). Verify by re-running tests/follow_up_triggers.rs and tests/encounter_e2e.rs which exercise ultimate charging.

**Negative tests:** Add an assertion that a follow-up emitting OnBreak does NOT cause infinite recursion within a single update (chain terminates because the second follower's preconditions don't re-trigger).

## Inputs

- `src/combat/follow_up.rs`
- `src/combat/events.rs`
- `src/combat/follow_up_tests.rs`
- `src/combat/ultimate.rs`
- `tests/follow_up_reentrancy.rs`

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/events.rs`
- `src/combat/follow_up_tests.rs`
- `tests/follow_up_chains.rs`

## Verification

1) `cargo check` clean. 2) `! grep -n 'OneHopSuppressed' src/ tests/` returns zero matches. 3) `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs` still returns line 77 (preserved). 4) `cargo test --test follow_up_chains` passes — asserts chain progression at depth 2+ and natural termination. 5) `cargo test` (full suite) — all binaries green; counts: 28 → 28 (rename, no net add).

## Observability Impact

Chains at depth 2+ now emit normally through the standard CombatEvent bus. JSONL output will show OnDamageDealt and lifecycle events with follow_up_depth=2,3,... — previously truncated at depth 1. Document this in the test as a regression hedge: if an executor accidentally re-introduces a depth cap, JSONL stops showing depth>=2 events.
