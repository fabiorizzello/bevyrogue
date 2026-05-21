---
id: T02
parent: S07
milestone: M002
key_files:
  - src/combat/action_query/legality/shared.rs
  - src/combat/action_query/legality/action.rs
  - src/combat/action_query/legality/resources.rs
key_decisions:
  - Introduced ult_readiness_from_snapshot in shared.rs (scalar-input mirror of effective_ult_gauge) instead of constructing a synthetic UltimateCharge component — keeps the snapshot the single source of truth and avoids fabricating trigger_type/charge_per_event values.
  - Both the action-status gate and the ResourceKind::Ultimate detail call the same helper so readiness and reported (current,required) cannot drift apart.
duration: 
verification_result: passed
completed_at: 2026-05-21T18:12:16.599Z
blocker_discovered: false
---

# T02: Route ult readiness in action_query legality through effective_ult_gauge so Agumon's energy-backed gauge gates ultimates while legacy Digimon stay on UltimateCharge.

**Route ult readiness in action_query legality through effective_ult_gauge so Agumon's energy-backed gauge gates ultimates while legacy Digimon stay on UltimateCharge.**

## What Happened

Added ult_readiness_from_snapshot(&UnitQuerySnapshot) -> (current, trigger, ready) to src/combat/action_query/legality/shared.rs. The helper mirrors effective_ult_gauge semantics but operates on the snapshot's scalar fields (no UltimateCharge reconstruction needed): when is_energy_backed(gauge_meta) is true and energy_data is Some, it returns (energy.current, energy.max, energy.current >= energy.max); otherwise it falls back to the legacy (ultimate_current, ultimate_trigger, ultimate_ready && current >= trigger) path. Replaced the inline !actor.ultimate_ready || actor.ultimate_current < actor.ultimate_trigger gate in action.rs::action_and_resource_status_for_snapshot (Ultimate kind) with a call to the helper, preserving the same UltimateNotReady disabled status. Updated resources.rs::build_resource_details so the ResourceKind::Ultimate detail also derives current/required via the same helper — UI/CLI consumers now see Energy values for Agumon and UltimateCharge values for legacy units through a single seam.

## Verification

Ran cargo test --features windowed --test action_query — all 41 existing tests pass, including the legacy ultimate_not_ready_disables_action_and_reports_current_and_required_ult_charge case which validates the fallback path. Additionally ran cargo check --features windowed to confirm the full crate type-checks.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test action_query` | 0 | pass | 20ms |
| 2 | `cargo check --features windowed` | 0 | pass | 1360ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query/legality/shared.rs`
- `src/combat/action_query/legality/action.rs`
- `src/combat/action_query/legality/resources.rs`
