---
id: T01
parent: S06
milestone: M011
key_files:
  - src/combat/av.rs
  - src/combat/turn_order.rs
  - src/combat/turn_system/mod.rs
  - src/combat/observability.rs
  - tests/turn_system_av.rs
  - tests/validation_snapshot.rs
key_decisions:
  - MAX_AV=10000 (HSR-inspired, allows granular speed differences and AV manipulation)
  - turn_preview derived from ECS ActionValue query (highest AV first, UnitId tiebreak, KO excluded)
  - seed() and insert_out_of_queue() kept as no-ops for backward compat with pre-AV tests
duration: 
verification_result: passed
completed_at: 2026-04-28T06:52:24.966Z
blocker_discovered: false
---

# T01: Refactored turn system to Action Value (AV) model with ActionValue ECS component, MAX_AV=10000, and restored turn_preview in validation snapshots

**Refactored turn system to Action Value (AV) model with ActionValue ECS component, MAX_AV=10000, and restored turn_preview in validation snapshots**

## What Happened

The existing VecDeque-based turn system was replaced with an Action Value model to support Delay effects and Tempo Resistance. Key changes:

1. **src/combat/av.rs** (new): Introduced `ActionValue` component (`i32` AV counter), `MAX_AV=10000`, `AV_PER_SPEED=100`, methods `advance()`, `delay()`, `self_advance()`, `reset()`, `is_ready()`, and `ActionValueUpdated` message for observability.

2. **src/combat/turn_order.rs**: Added `TurnAdvanced` with `av_at_turn` and `av_change` fields for traceability; `seed()` and `insert_out_of_queue()` converted to no-ops with compat shims for pre-AV tests; `order_from_speeds()` retained for speed-based initial seeding.

3. **src/combat/turn_system/mod.rs**: Rewired `advance_turn_system` and `resolve_action_system` to operate on `ActionValue` components; KO'd and Stunned units skip AV advancement; the active unit is determined by highest AV (≥ MAX_AV), with UnitId as tiebreak.

4. **src/combat/observability.rs**: Restored `turn_preview: Vec<UnitId>` on `ValidationSnapshot`, populated by querying all non-KO units' `ActionValue`, sorted descending by AV then ascending by UnitId. Always included in the formatted snapshot string.

5. **tests/turn_system_av.rs** (new): 4 integration tests covering basic AV advancement and turn selection, stunned-unit skip, KO-unit skip, and tie-breaking by UnitId.

6. **tests/validation_snapshot.rs**: Updated expected strings to include `turn_preview` field in formatted output.

7. Multiple existing integration tests updated to use new `TurnAdvanced::of()` constructor and remove references to the deprecated `future_preview`/`queue` fields.

The session was interrupted mid-execution; this resumption fixed the remaining failing test (`snapshot_defaults_empty_optional_surfaces`) by implementing AV-derived turn_preview in the snapshot formatter.

## Verification

Ran `cargo test --test turn_system_av` — 4/4 pass. Ran `cargo test --test validation_snapshot` — 3/3 pass. Ran full `cargo test` — 0 failures across all test crates.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test turn_system_av -- --nocapture` | 0 | ✅ pass | 180ms |
| 2 | `cargo test --test validation_snapshot -- --nocapture` | 0 | ✅ pass | 2150ms |
| 3 | `cargo test 2>&1 | grep FAILED | wc -l` | 0 | ✅ pass (0 failures) | 3200ms |

## Deviations

Restored turn_preview in ValidationSnapshot using AV-derived ECS query rather than TurnOrder.future_preview (which is now a no-op shim). First test expected string updated to include turn_preview=[1,2] (alive units, KO'd unit 4 excluded).

## Known Issues

None.

## Files Created/Modified

- `src/combat/av.rs`
- `src/combat/turn_order.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/observability.rs`
- `tests/turn_system_av.rs`
- `tests/validation_snapshot.rs`
