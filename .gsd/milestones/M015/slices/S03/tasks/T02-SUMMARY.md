---
id: T02
parent: S03
milestone: M015
key_files:
  - src/combat/kernel.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
  - src/combat/predator_loop.rs
  - src/main.rs
  - src/bin/combat_cli.rs
  - tests/event_stream.rs
  - tests/validation_snapshot.rs
  - tests/battery_loop_kernel.rs
key_decisions:
  - Emitted beat-shaped kernel transitions directly from the deterministic action pipeline instead of adding an unordered bridge system.
  - Kept runtime registration as the canonical app-facing source for all existing kernel applier systems.
  - Used `ParamSet` for Predator Loop read/write access to the shared `CombatEvent` message resource to avoid Bevy B0002 conflicts.
duration: 
verification_result: passed
completed_at: 2026-05-08T15:56:52.818Z
blocker_discovered: false
---

# T02: Wired the combat kernel runtime into headless app paths and emitted live action lifecycle beat/kernel events.

**Wired the combat kernel runtime into headless app paths and emitted live action lifecycle beat/kernel events.**

## What Happened

Registered the complete kernel runtime in the primary app and combat CLI so headless paths initialize the registry plus Twin Core, Battery Loop, Holy Support, Predator Loop, and Precision Mind Game resources/appliers. Added direct deterministic action lifecycle beat emission to root and follow-up action resolution: Declared, PreApp, Impact, Damage, Applied, and Resolved now produce `OnCombatBeat` and matching canonical `OnKernelTransition::Beat` events, with registry dispatch surfacing hook-derived transitions such as Twin Core. Completed runtime applier wiring by adding Twin Core, Holy Support, and Precision Mind Game systems, and fixed the Predator Loop applier’s same-message read/write conflict with `ParamSet` so runtime registration can safely emit resolved Predator events. Extended regression coverage for live beat/kernel events, all-domain runtime appliers, runtime snapshot resources, and duplicate Battery Loop applier behavior.

## Verification

Fresh verification passed after the final code changes and formatting. The required scoped command passed 18 tests across `event_stream`, `battery_loop_kernel`, `predator_loop_kernel`, and `validation_snapshot`. `cargo check` exited 0 for the real app/CLI wiring, and the slice authority audit script reported success.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test event_stream --test battery_loop_kernel --test predator_loop_kernel --test validation_snapshot` | 0 | ✅ pass | 5438ms |
| 2 | `cargo check` | 0 | ✅ pass | 14907ms |
| 3 | `python3 scripts/verify_combat_authority_audit.py` | 0 | ✅ pass | 60ms |

## Deviations

Minor local adaptation: runtime registration exposed an existing Bevy B0002 conflict in the Predator Loop applier because it both read and wrote `CombatEvent`; this was fixed with `ParamSet` rather than replanning. I also ran `cargo check` and the slice audit script in addition to the task’s required test command.

## Known Issues

Existing compiler warning backlog remains outside this task; all verification commands exited 0.

## Files Created/Modified

- `src/combat/kernel.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `src/combat/predator_loop.rs`
- `src/main.rs`
- `src/bin/combat_cli.rs`
- `tests/event_stream.rs`
- `tests/validation_snapshot.rs`
- `tests/battery_loop_kernel.rs`
