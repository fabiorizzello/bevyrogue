---
id: T03
parent: S10
milestone: M021
key_files:
  - src/combat/kernel.rs
  - src/combat/events.rs
  - src/combat/mod.rs
  - src/combat/blueprints/dorumon/identity.rs
  - src/combat/observability.rs
  - tests/battery_loop_kernel.rs
  - tests/dorumon_predator_runtime.rs
  - tests/event_stream.rs
  - tests/predator_loop_kernel.rs
key_decisions:
  - Shared combat kernel/event surfaces no longer expose digimon-specific BatteryLoop/PredatorLoop variants.
  - Owner-module observability formatting now imports mechanic step types directly from the owner modules.
  - Shared event-stream verification now focuses on generic blueprint transitions rather than digimon-named resolved events.
duration: 
verification_result: mixed
completed_at: 2026-05-17T06:29:09.530Z
blocker_discovered: true
---

# T03: Collapsed shared combat event/kernel surfaces to generic blueprint seams and updated owner-module routing, but Dorumon runtime verification still needs one follow-up pass

**Collapsed shared combat event/kernel surfaces to generic blueprint seams and updated owner-module routing, but Dorumon runtime verification still needs one follow-up pass**

## What Happened

Removed the digimon-specific raw transition exports from the shared kernel surface, removed the BatteryLoopResolved and PredatorLoopResolved event variants, and moved Dorumon predator transition decoding fully behind the blueprint owner module. Updated the battery-loop, Dorumon runtime, and shared event-stream regression tests to assert the generic OnKernelTransition(Blueprint) seam plus owner-owned state, and re-pointed observability formatting at the owner modules. The remaining issue is a Dorumon runtime assertion that still sees a rejected prey-lock transition instead of the expected applied transition, so the unit is not fully closed yet.

## Verification

Verified with cargo test --test battery_loop_kernel and cargo test --test event_stream passing. cargo test --test dorumon_predator_runtime currently fails on the predator runtime state assertion: the final transition is rejected with CapReached { cap: PreyLock } instead of the expected applied prey-lock transition.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test battery_loop_kernel --quiet` | 0 | ✅ pass | 0ms |
| 2 | `cargo test --test event_stream --quiet` | 0 | ✅ pass | 0ms |
| 3 | `cargo test --test dorumon_predator_runtime --quiet` | 101 | ❌ fail | 0ms |

## Deviations

Paused before fully resolving the Dorumon runtime test mismatch because the context budget warning required wrap-up.

## Known Issues

Dorumon runtime test still fails at the final last_transition assertion; it likely needs a small follow-up adjustment in the runtime test or transition application flow.

## Files Created/Modified

- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/mod.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/observability.rs`
- `tests/battery_loop_kernel.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/event_stream.rs`
- `tests/predator_loop_kernel.rs`
