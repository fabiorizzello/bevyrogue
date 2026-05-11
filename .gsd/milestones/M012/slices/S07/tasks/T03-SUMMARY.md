---
id: T03
parent: S07
milestone: M012
key_files:
  - src/ui/combat_panel.rs
  - tests/action_affordance_consumers.rs
key_decisions:
  - Use `TurnOrder.active_unit` as the authoritative actor source for query-backed affordances; fall back to preview state only for display labels.
  - Render ally and enemy target cards from the query target affordances so KO allies can be selected for revive-like actions without hardcoded UI legality rules.
duration: 
verification_result: passed
completed_at: 2026-05-01T14:10:17.789Z
blocker_discovered: false
---

# T03: Routed windowed combat affordances through shared legality queries and target affordances.

**Routed windowed combat affordances through shared legality queries and target affordances.**

## What Happened

Updated `combat_panel()` to derive the windowed action row and per-unit target cards from the shared `query_action_affordance()` path used by CLI. The panel now builds a single ECS snapshot per frame with the missing inputs (`Commander`, `Energy`, `RoundEnergyTracker`, and real SP), uses `TurnOrder.active_unit` as the authoritative actor source, surfaces disabled/deferred/hidden statuses in button labels and hover text, and renders both ally and enemy cards as potential targets when an action is pending. If the selected pending action becomes invalid, the panel clears it before it can emit a stale intent. I also extended the consumer coverage to prove commander/energy/tracker/SP propagation and to keep revive-style KO-ally targeting covered without driving egui.

## Verification

Ran `cargo test-dev --test action_affordance_consumers` and `cargo check --features "dev windowed"` after the last code change. The test suite passed with 6/6 tests green, including the new commander/energy/tracker/SP snapshot coverage and the revive KO-ally target affordance checks. The windowed build check completed successfully with exit code 0, confirming the feature-gated UI/query signatures compile together.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_consumers` | 0 | ✅ pass | 5000ms |
| 2 | `cargo check --features "dev windowed"` | 0 | ✅ pass | 11800ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel.rs`
- `tests/action_affordance_consumers.rs`
