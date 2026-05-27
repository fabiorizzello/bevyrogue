---
id: T02
parent: S06
milestone: M006
key_files:
  - src/animation/registry.rs
key_decisions:
  - Used continue; (not loop restructure) — minimal, semantics-preserving change that matches the task contract exactly
duration: 
verification_result: passed
completed_at: 2026-05-27T06:53:11.203Z
blocker_discovered: false
---

# T02: Replace two early `return;` with `continue;` in populate_graph_registries so all matching graph events in a batch are processed.

**Replace two early `return;` with `continue;` in populate_graph_registries so all matching graph events in a batch are processed.**

## What Happened

Read the full `populate_graph_registries` function in `src/animation/registry.rs` (lines 236–283). The for-loop over `events.read()` had two `return;` statements — one after inserting into `skill_reg` (line 275) and one after inserting into `stance_reg` (line 279). These caused the system to exit the entire function after the first matching event, starving all subsequent events in the same batch. The fix was minimal: replace both `return;` with `continue;` so each event is handled independently and the loop continues to the next event after each insert. Per-event error isolation (the earlier `continue` guards for missing asset, missing handle, etc.) is unaffected. The T01 starvation test (`populate_graph_registries_starves_second_event_when_first_matches`) immediately turned green, and all 120 animation tests plus the full headless suite passed.

## Verification

Ran `cargo test --test animation` — 120 tests including the T01 starvation case all pass (exit 0, ~3.2s). Ran `cargo test` — full headless suite passes (exit 0, ~5.0s).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | PASS — 120 tests including registry_starvation::populate_graph_registries_starves_second_event_when_first_matches | 3192ms |
| 2 | `cargo test` | 0 | PASS — full headless suite green | 4960ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/registry.rs`
