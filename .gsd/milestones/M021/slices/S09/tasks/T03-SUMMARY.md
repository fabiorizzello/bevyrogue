---
id: T03
parent: S09
milestone: M021
key_files:
  - src/combat/kernel.rs
  - src/combat/battery_loop.rs
  - src/combat/blueprints/dorumon/identity.rs
  - src/combat/events.rs
  - src/combat/observability.rs
  - src/combat/mod.rs
  - src/combat/blueprints/dorumon/mod.rs
  - tests/predator_loop_kernel.rs
  - tests/event_stream.rs
key_decisions:
  - Battery Loop and Predator Loop transition payload types now live in their owner runtime modules and are re-exported for stable combat imports.
  - Shared event and observability code now imports typed payloads from the owner modules instead of `kernel.rs`.
  - Event-stream coverage now pins raw Blueprint owners for `twin_core`, `dorumon`, and `tentomon` alongside the typed resolved-event surfaces.
duration: 
verification_result: passed
completed_at: 2026-05-16T22:39:42.251Z
blocker_discovered: false
---

# T03: Moved battery and predator transition payload ownership into owner modules and updated shared event/observability imports.

**Moved battery and predator transition payload ownership into owner modules and updated shared event/observability imports.**

## What Happened

Relocated the Battery Loop and Predator Loop transition/signal/blocking types out of `src/combat/kernel.rs` into their owning runtime modules (`src/combat/battery_loop.rs` and `src/combat/blueprints/dorumon/identity.rs`), then re-exported them from the combat root and Dorumon namespace for stable callers. Updated `src/combat/events.rs` and `src/combat/observability.rs` to import the typed resolved-event payloads from the owner modules instead of `kernel.rs`, preserving the typed `BatteryLoopResolved` / `PredatorLoopResolved` surfaces. Refreshed the targeted tests so `tests/predator_loop_kernel.rs` imports from Dorumon and `tests/event_stream.rs` now asserts raw Blueprint owner shapes for `twin_core`, `dorumon`, and `tentomon` while keeping the typed resolved-event variant coverage intact.

## Verification

`cargo test --test event_stream`, `cargo test --test predator_loop_kernel`, and `cargo check` all passed. The first two confirm the event-stream and predator-loop surfaces still serialize and resolve correctly after the ownership move; `cargo check` completed successfully with only pre-existing warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test event_stream` | 0 | ✅ pass | 8426ms |
| 2 | `cargo test --test predator_loop_kernel` | 0 | ✅ pass | 8832ms |
| 3 | `cargo check` | 0 | ✅ pass | 4758ms |

## Deviations

None.

## Known Issues

`cargo check` emits existing warnings unrelated to this refactor.

## Files Created/Modified

- `src/combat/kernel.rs`
- `src/combat/battery_loop.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- `src/combat/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `tests/predator_loop_kernel.rs`
- `tests/event_stream.rs`
