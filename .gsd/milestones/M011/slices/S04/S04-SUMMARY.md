---
id: S04
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["src/bin/combat_cli.rs"]
key_decisions:
  - ["MinimalPlugins + ScheduleRunnerPlugin at 10Hz to save CPU for the CLI harness (T01)", "Interactive inquire prompt placed before App::run() to avoid blocking the Bevy main loop (T01)", "combat_dashboard_system must run before event_logger_system in .chain() because MessageReader is consuming — whichever runs first drains the queue (T03)", "bootstrap_system fires the first TurnAdvanced after apply_composition so advance_turn_system loop can start (T03)", "PlayerActed(bool) resource as a two-state machine (prompt vs. advance-turn) instead of a tick counter or separate system (T04)", "units.ron loaded synchronously via std::fs::read_to_string + ron::from_str before App::run() so roster data is available without the Bevy asset system (T05)", "bootstrap_system consumes SelectedAllies resource directly, eliminating party.ron dependency for CLI runs (T05)"]
patterns_established:
  - ["TTY check pattern: check atty::is(atty::Stream::Stdin) once before App::run() and store as IsInteractive(bool) resource — all interactive systems read from this resource", "Pre-App synchronous data load: load RON assets via std::fs + ron::from_str before App::run() when the data must be available before the first ECS tick", "Consuming MessageReader ordering: systems that read CombatEvent via MessageReader must be ordered explicitly in .chain() — earlier in chain = first to drain the queue"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T19:01:40.625Z
blocker_discovered: false
---

# S04: Headless interactive CLI (combat playtest harness)

**Interactive combat_cli binary with inquire-driven roster selection and turn-by-turn action menus, streaming CombatEvent bus to stdout as a playtest harness**

## What Happened

S04 built the `combat_cli` binary end-to-end across five tasks.

**T01** established the binary entry point with `MinimalPlugins` + `ScheduleRunnerPlugin` at 10 Hz and placed the interactive `inquire` prompt before `App::run()` to avoid blocking the Bevy main loop during startup.

**T02** integrated the headless engine: registered `DataPlugin` and core combat systems, added a `bootstrap_system` that spawns an encounter once data loads, and implemented `event_logger_system` to stream `CombatEvent` kinds to stdout. A TTY check enables non-interactive CI runs without blocking on `inquire` prompts.

**T03** fixed three compile errors in the dashboard scaffolding (string concat syntax, `SpPool` named fields, struct variant matching) and two runtime ordering bugs: `combat_dashboard_system` must run before `event_logger_system` in the `.chain()` because `MessageReader` is consuming, and `bootstrap_system` must fire the first `TurnAdvanced` after `apply_composition` seeds the turn queue.

**T04** added `player_action_system` with a `PlayerActed(bool)` two-state machine. When `WaitingAction` and flag is false, it presents `inquire::Select` prompts for action type then target, emits `ActionIntent`, and sets the flag. On the next tick it calls `order.advance()` and fires `TurnAdvanced` to rotate the queue. Ultimate option is conditionally shown based on `UltimateCharge::ready()`.

**T05** replaced the placeholder "Press Enter" prompt with a full pre-combat roster selection phase. `load_ally_roster()` reads `assets/data/units.ron` synchronously via `ron::from_str` before `App::run()`, filters to ally units, and presents them via `inquire::MultiSelect` enforcing exactly 4 selections. `bootstrap_system` now consumes a `SelectedAllies` resource directly, eliminating the `party.ron` dependency for CLI runs.

## Verification

1. `cargo check --bin combat_cli` — clean, no errors or warnings.
2. `echo "" | timeout 12 cargo run --bin combat_cli 2>/dev/null` — non-interactive mode selects first 4 allies, bootstraps combat, renders dashboard with SP/TurnOrder/HP/ULT/TGH per unit, streams CombatEvent bus (PartySelected, TurnOrderSeeded, OnDamageDealt, UltGain, etc.), and cycles through multiple turns.
3. `cargo test` — all integration tests pass.

## Requirements Advanced

None.

## Requirements Validated

- R082 — cargo run --bin combat_cli launches the harness, presents inquire::MultiSelect for 4-ally selection, shows turn-by-turn action menus (Basic/Skill/Ultimate), and streams CombatEvent bus to stdout. Non-interactive CI path confirmed via echo pipe. No TUI framework required.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

["No enemies in the bootstrap encounter — all attacks resolve ally-on-ally, producing 'Target is a Commander' log entries for Taichi. Enemy encounter wiring is deferred to S09.", "The 60-tick (6s) timeout exits the non-interactive path unconditionally — long encounters in CI may be cut short if future slices add more processing per tick."]

## Follow-ups

None.

## Files Created/Modified

- `src/bin/combat_cli.rs` — New binary: full interactive combat playtest harness with roster selection, dashboard rendering, turn-by-turn action menus, and CombatEvent streaming
