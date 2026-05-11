---
id: S07
parent: M012
milestone: M012
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/action_query.rs
  - src/bin/combat_cli.rs
  - src/ui/combat_panel.rs
  - tests/action_affordance_consumers.rs
  - .gsd/PROJECT.md
key_decisions:
  - Keep the engine-facing snapshot builder on the SP-bypass path and expose a separate explicit-SP helper for UI/CLI affordance snapshots.
  - Keep consumer selection helpers pure by filtering query output only; do not re-encode KO/team legality rules outside `query_action_affordance()`.
  - Use `TurnOrder.active_unit` as the authoritative actor source for windowed affordances, and let revive-like KO ally targeting come from query-backed target affordances rather than a special UI branch.
  - Use exact-literal source scans as a regression to prevent hardcoded KO/team/skill-ID legality paths from returning in CLI or windowed adapters.
patterns_established:
  - One snapshot per turn/frame, then reuse the shared legality query for both action and target affordances.
  - Surface disabled/deferred/hidden states with machine-readable reason codes instead of local legality heuristics.
  - Keep UI/CLI adapters thin: they may display and filter affordances, but they should not decide legality.
  - Protect query-backed consumer code with both behavioral tests and static source scans.
observability_surfaces:
  - CLI action/target labels with reason codes
  - Windowed action buttons and hover text
  - Windowed ally/enemy target-card enablement
  - Consumer regression tests for snapshot fidelity and hardcoding scans
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-01T14:14:49.923Z
blocker_discovered: false
---

# S07: S07: CLI/windowed affordance integration

**CLI and windowed combat affordances now consume the shared DSL-backed legality query, so enable/disable/hide decisions and revive-like KO-ally targeting are truthful before an intent is emitted.**

## What Happened

S07 wired the player-facing combat consumers onto the shared legality/query surface introduced earlier in M012. The CLI now builds a single affordance snapshot per turn with the real `SpPool.current` plus the missing commander/energy/tracker inputs, uses `query_action_affordance()` for Basic/Skill/Ultimate presentation, and selects the first enabled target from query output in non-interactive mode instead of applying local KO/team heuristics. The windowed combat panel follows the same path: it derives action enablement from `ActionStatus`, renders ally and enemy target cards from `TargetAffordance`, uses `TurnOrder.active_unit` as the authoritative actor source, and clears stale pending actions if state changes invalidate them.

Across the slice, the consumer layer was kept intentionally thin: selection helpers stay pure, legality decisions stay in the DSL/query surface, and the adapters only filter or display the query results. A regression test suite now covers explicit-SP snapshot behavior, Basic target selection from query output, revive-like KO-ally targeting with disabled live targets, snapshot propagation of commander/energy/tracker/SP inputs, and a static no-hardcoding scan that protects both CLI and windowed adapters from reintroducing local legality branches. This closes the S07 contract that UI-facing code must not invent its own action/target legality rules while keeping the engine-facing SP-bypass snapshot path intact for S06 parity.

## Verification

Fresh final verification passed with `cargo test-dev --test action_affordance_consumers && cargo test-dev --test action_affordance_query && cargo test-dev --test engine_legality_integration && cargo test-dev && cargo check --features "dev windowed"`. The targeted consumer suite passed 7/7, the shared affordance-query suite passed 23/23, the engine legality integration suite passed 7/7, the full Rust test suite passed (including CLI/windowed and combat regression coverage), and the windowed feature check compiled successfully. The build emitted pre-existing warnings, but no verification command failed.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
