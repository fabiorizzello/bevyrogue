---
id: T02
parent: S08
milestone: M021
key_files:
  - src/combat/blueprints/gabumon/mod.rs
  - src/combat/blueprints/gabumon/signals.rs
key_decisions:
  - Placed OWNER const in mod.rs and referenced it as super::OWNER from signals.rs — matches the task plan's re-export intent and avoids duplicating the constant.
  - Used fully-qualified crate path for CustomSignalDispatchError/amount_payload in signals.rs rather than super::super:: chaining — clearer and less fragile under future module moves.
duration: 
verification_result: passed
completed_at: 2026-05-16T21:20:16.387Z
blocker_discovered: false
---

# T02: Converted Gabumon to directory module with twin_core imports — split signals.rs off from mod.rs and eliminated blueprints::agumon coupling.

**Converted Gabumon to directory module with twin_core imports — split signals.rs off from mod.rs and eliminated blueprints::agumon coupling.**

## What Happened

Gabumon lived in a single flat file `src/combat/blueprints/gabumon.rs` importing `TwinCoreDesignTag` and `twin_core_added_tag_transition` from `blueprints::agumon`. After T01 extracted TwinCore into its own mini-plugin, those types now live in `blueprints::twin_core`.

Steps taken:
1. Created `src/combat/blueprints/gabumon/signals.rs` — contains `GabumonSignal` enum, `parse()`, and `dispatch()`, importing from `crate::combat::blueprints::twin_core` (not agumon). Uses `super::OWNER` to reference the constant defined in mod.rs.
2. Created `src/combat/blueprints/gabumon/mod.rs` — contains `OWNER`, all passive runtime code, `mod signals; pub use signals::dispatch;`. Stripped `CompiledTimeline` from imports (not directly needed in mod.rs scope) and removed the `agumon` use.
3. Removed the old `src/combat/blueprints/gabumon.rs` flat file to avoid the Rust duplicate-module error.
4. `pub mod gabumon;` in `blueprints/mod.rs` required no change — Rust resolves directory modules transparently.

## Verification

cargo check completed with 0 errors (warnings only, pre-existing). cargo test passed all tests (0 failures across all integration suites). rg "blueprints::agumon" src/combat/blueprints/gabumon/ returned 0 matches.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass — 0 errors, warnings only | 5950ms |
| 2 | `cargo test` | 0 | pass — all tests ok | 45000ms |
| 3 | `rg "blueprints::agumon" src/combat/blueprints/gabumon/` | 1 | pass — 0 lines (no agumon coupling) | 100ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/gabumon/mod.rs`
- `src/combat/blueprints/gabumon/signals.rs`
