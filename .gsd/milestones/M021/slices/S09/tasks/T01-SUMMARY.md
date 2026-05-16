---
id: T01
parent: S09
milestone: M021
key_files:
  - src/combat/blueprints/dorumon/signals.rs
  - src/combat/blueprints/dorumon/hooks.rs
  - src/combat/blueprints/dorumon/identity.rs
  - tests/dorumon_blueprint.rs
  - tests/dorumon_predator_runtime.rs
key_decisions:
  - Dorumon raw Predator Loop transport now uses CombatKernelTransition::Blueprint { owner: "dorumon", name, payload } instead of the kernel-local PredatorLoop envelope.
  - The Dorumon runtime applier ignores foreign or malformed Blueprint events and only mutates state from valid dorumon Blueprint transitions.
duration: 
verification_result: mixed
completed_at: 2026-05-16T22:27:00.118Z
blocker_discovered: false
---

# T01: Moved Dorumon Predator Loop raw transitions onto generic Blueprint owner envelopes without changing the typed PredatorLoopResolved runtime seam.

**Moved Dorumon Predator Loop raw transitions onto generic Blueprint owner envelopes without changing the typed PredatorLoopResolved runtime seam.**

## What Happened

Replaced Dorumon custom-signal dispatch so build_exploit, apply_prey_lock, consume_prey_lock_payoff, and enter_berserk now emit CombatKernelTransition::Blueprint values owned by dorumon, matching the shared owner/payload transport used by Twin Core. Updated the wrapped-cycle hook to emit a dorumon Blueprint tick envelope, then rewrote the PredatorLoop runtime applier to accept only dorumon Blueprint transitions, decode their names and payloads back into typed PredatorLoopTransition values, preserve the existing EnterBerserk strain override, and ignore foreign or malformed Blueprint events without mutating state. Refreshed the Dorumon blueprint tests to assert the raw Blueprint envelope shape first, and refreshed the runtime test to prove the resolved PredatorLoop events, state snapshot, and observability remain unchanged while non-Dorumon and malformed Blueprint writes are ignored.

## Verification

Ran the targeted test binaries for the migrated surfaces. The first combined cargo test run exposed a missing test import in tests/dorumon_predator_runtime.rs; after fixing that, the rerun passed. Final verification confirmed both the Blueprint-envelope dispatch tests and the runtime resolution tests succeed, including the negative checks for foreign owner and malformed payload handling.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test dorumon_blueprint -- --nocapture && cargo test --test dorumon_predator_runtime -- --nocapture` | 101 | ❌ fail | 5239ms |
| 2 | `cargo test --test dorumon_blueprint -- --nocapture && cargo test --test dorumon_predator_runtime -- --nocapture` | 0 | ✅ pass | 704ms |

## Deviations

Added an extra runtime negative test for non-Dorumon and malformed Blueprint events to pin the new ignore behavior.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`
