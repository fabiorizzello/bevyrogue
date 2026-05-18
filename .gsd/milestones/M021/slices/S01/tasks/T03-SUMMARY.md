---
id: T03
parent: S01
milestone: M021
key_files:
  - src/combat/api/intent.rs
  - src/combat/events.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
  - src/combat/api/applier.rs
  - src/main.rs
  - tests/cast_id_propagation.rs
key_decisions:
  - CastId::ROOT = CastId(NonZeroU32(1)) reserved for lifecycle events outside step_app
  - Option<ResMut<CastIdGen>> in system params for backward compat with tests missing the resource
  - OnCombatBeat excluded from in-cast filter in test since Declared/PreApp beats are emitted pre-step_app with ROOT
duration: 
verification_result: passed
completed_at: 2026-05-14T23:24:14.159Z
blocker_discovered: false
---

# T03: CastId propagation in CombatEvent + pipeline::step_app + emit sites — all 3 integration tests green

**CastId propagation in CombatEvent + pipeline::step_app + emit sites — all 3 integration tests green**

## What Happened

Added `cast_id: CastId` field to `CombatEvent`, implemented `CastIdGen` Resource (monotonic counter starting at 1, first issued id is 2), and propagated cast_id through all ~70+ emit sites in pipeline.rs, turn_system/mod.rs, follow_up.rs, and applier.rs.

Pre-cast lifecycle events (Declared, PreApp, Applied, Resolved beats) emit with `CastId::ROOT`. Within `step_app`, a single cast_id is allocated via `CastIdGen::next()` before entry and threaded through all emit calls. `Option<ResMut<CastIdGen>>` pattern used in system parameters to remain backward-compatible with tests that don't register the resource.

`serde::Serialize` added to `CastId` derives since `CombatEvent` derives Serialize for the JSONL logger. JSONL output now includes `cast_id: u32` per event (additive, non-breaking).

All 20+ test files and src emit sites updated to include `cast_id: CastId::ROOT` in direct `CombatEvent {}` literals. `CastIdGen` registered in `src/main.rs` app builder.

Integration test `tests/cast_id_propagation.rs` written with 3 assertions: (a) in-cast events share same non-ROOT cast_id, (b) lifecycle events use ROOT, (c) sequential casts have distinct non-ROOT cast_ids. Initial test had `OnCombatBeat { .. }` in the in-cast filter which also matched pre-cast Declared beat (ROOT) — fixed by removing `OnCombatBeat` and `OnKernelTransition` from the in-cast filter since those are also emitted outside step_app.

## Verification

cargo test --test cast_id_propagation → 3/3 pass; cargo test → full suite green; cargo check --features windowed → no errors

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test cast_id_propagation` | 0 | 3 tests passed: cast_events_share_nonroot_cast_id, lifecycle_events_use_root_cast_id, sequential_casts_have_distinct_cast_ids | 560ms |
| 2 | `cargo test` | 0 | Full suite green | 30000ms |
| 3 | `cargo check --features windowed` | 0 | No errors | 15000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/intent.rs`
- `src/combat/events.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `src/combat/api/applier.rs`
- `src/main.rs`
- `tests/cast_id_propagation.rs`
