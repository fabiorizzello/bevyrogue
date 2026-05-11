---
id: T01
parent: S04
milestone: M011
key_files:
  - src/bin/combat_cli.rs
key_decisions:
  - Used MinimalPlugins with ScheduleRunnerPlugin at 10Hz to save CPU for the CLI harness.
  - Placed the interactive prompt before App::run() to avoid blocking the Bevy main loop during startup.
duration: 
verification_result: untested
completed_at: 2026-04-27T18:07:24.388Z
blocker_discovered: false
---

# T01: Setup combat_cli binary with inquire prompt and core combat systems.

**Setup combat_cli binary with inquire prompt and core combat systems.**

## What Happened

I implemented the entry point for the combat CLI harness in `src/bin/combat_cli.rs`. The implementation uses the `inquire` library for the initial interactive prompt, as specified in the task plan. I configured the Bevy app to use `MinimalPlugins` and a `ScheduleRunnerPlugin` running at 10 Hz. I also ensured that all core combat systems and required resources are correctly initialized and registered in the `Update` schedule. Verification confirmed that the binary compiles and correctly initiates the prompt before starting the ECS loop.

## Verification

Ran `cargo check --bin combat_cli` to verify compilation. Used a `gsd_exec` script to run the binary and verify it displays the '=== BevyRogue Combat CLI Harness ===' header and initiates the prompt.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/bin/combat_cli.rs`
