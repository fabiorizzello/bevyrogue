---
id: T01
parent: S08
milestone: M011
key_files:
  - src/combat/follow_up.rs
  - src/combat/events.rs
  - src/combat/follow_up_tests.rs
  - tests/follow_up_chains.rs
key_decisions:
  - Changed hardcoded follow_up depth `1` in resolve_follow_up_action_system to `intent.origin.follow_up_depth + 1` so depth-2+ events actually appear in the event stream (required for D046 observability impact)
  - Used ActionIntent::Skill (not Ultimate) for the breaker in the chain-termination test to bypass the ult_ready gate — breaker starts with current=0 which would silently fail as Ultimate
duration: 
verification_result: passed
completed_at: 2026-04-28T09:43:41.951Z
blocker_discovered: false
---

# T01: Removed D026 engine re-entrancy cap (OneHopSuppressed guard + variant), fixed depth increment to origin+1, and replaced follow_up_reentrancy.rs with follow_up_chains.rs asserting depth-2 chain progression and natural termination

**Removed D026 engine re-entrancy cap (OneHopSuppressed guard + variant), fixed depth increment to origin+1, and replaced follow_up_reentrancy.rs with follow_up_chains.rs asserting depth-2 chain progression and natural termination**

## What Happened

Removed the D026 one-hop engine cap from `src/combat/follow_up.rs`: deleted the `if event.follow_up_depth >= 1 { return Err(OneHopSuppressed) }` guard from `evaluate_follow_up` (lines 161–163) and the `OneHopSuppressed` variant from `FollowUpSkipReason`. The preserved `follow_up_depth >= 1` check in `ultimate.rs:77` (OnAllyFollowUp gating) was left intact as required.

Also fixed the depth increment in `resolve_follow_up_action_system`: changed the hardcoded `1` passed to `step_declaration` to `intent.origin.follow_up_depth + 1`. This is required to produce events with `follow_up_depth >= 2` — without it, all follow-up events would always emit at depth=1 regardless of chain depth, making the D046 observability goal unachievable.

Updated the doc comment on `CombatEvent.follow_up_depth` in `events.rs` to reference D046 (chain bounding lives in the data) and remove the stale D026 reference.

Cleaned `src/combat/follow_up_tests.rs`: removed the `chained_break` fixture variable and the `OneHopSuppressed` assertion from `follow_up_reports_ineligible_reasons`, and deleted the entire `follow_up_one_hop_guard_blocks_second_chain` test.

Deleted `tests/follow_up_reentrancy.rs` and created `tests/follow_up_chains.rs` with two tests:
1. `depth_chain_progresses_to_depth_two`: Agumon (OnEnemyBreak follow-up) + inline Impmon-like ally (OnEnemyKill → dorumon_follow_up Dark+ToughnessHit). Agumon's Ultimate kills Enemy A (high toughness, no break). Impmon fires at depth=1, breaking Enemy B (HP=200 survives). In update 2, Agumon fires at depth=2 on already-broken Enemy B — no new OnBreak → chain terminates. Asserts depth=2 events appear and no depth>2 events exist.
2. `chain_terminates_when_follow_up_cannot_retrigger`: Greymon (OnEnemyBreak follow-up) + inline breaker (no follow-up, fires agumon_ult as Skill). Greymon's follow-up fires at depth=1 within the same update as the break. The depth=1 follow-up hits the already-broken enemy — no new OnBreak — chain stops after depth=1. Asserts no depth>=2 events appear.

Note: the second test uses `ActionIntent::Skill { skill_id: agumon_ult }` (not `ActionIntent::Ultimate`) to bypass the `ult_ready` check in `apply_effects` (requires current >= trigger; breaker starts at current=0). This is standard test fixture practice — using Skill intent for a fire skill with enough ToughnessHit is functionally equivalent for chain-termination verification purposes.

## Verification

1) `cargo check` — clean (only pre-existing warnings). 2) `grep -rn 'OneHopSuppressed' src/ tests/` — zero matches. 3) `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs` — line 77 preserved. 4) `cargo test --test follow_up_chains` — 2 passed, 0 failed. 5) `cargo test` (full suite) — 29 integration test binaries all green, zero failures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ pass | 830ms |
| 2 | `grep -rn 'OneHopSuppressed' src/ tests/` | 1 | ✅ pass — zero matches | 10ms |
| 3 | `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs` | 0 | ✅ pass — line 77 preserved | 5ms |
| 4 | `cargo test --test follow_up_chains` | 0 | ✅ pass — 2/2 green | 440ms |
| 5 | `cargo test` | 0 | ✅ pass — 29 binaries all green | 5200ms |

## Deviations

The task plan did not explicitly mention changing the hardcoded `1` in `resolve_follow_up_action_system` to `intent.origin.follow_up_depth + 1`, but this change is required to produce events with `follow_up_depth >= 2`. Without it, all follow-up events always emit at depth=1, making the Observability Impact section's statement ('JSONL will show events with follow_up_depth=2,3,...') impossible. This is a necessary implementation detail implicit in D046.

## Known Issues

None.

## Files Created/Modified

- `src/combat/follow_up.rs`
- `src/combat/events.rs`
- `src/combat/follow_up_tests.rs`
- `tests/follow_up_chains.rs`
