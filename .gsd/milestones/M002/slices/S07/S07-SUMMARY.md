---
id: S07
parent: M002
milestone: M002
provides:
  - A downstream-safe Agumon ult resource seam where UI/query/runtime all agree that full Energy enables Ultimate and casting Ultimate drains the same gauge.
  - A reusable snapshot/helper pattern for future Digimon migrations from legacy UltimateCharge to metadata-driven alternate gauges.
requires:
  []
affects:
  []
key_files:
  - src/combat/action_query/types.rs
  - src/combat/turn_system/resolve.rs
  - src/combat/action_query/legality/shared.rs
  - src/combat/action_query/legality/action.rs
  - src/combat/action_query/legality/resources.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/turn_system/pipeline/paths/bounce/finalize.rs
  - tests/digimon_kits/agumon_energy_gauge.rs
  - tests/digimon_kits/holy_support_roster_contract.rs
key_decisions:
  - Keep metadata-free Digimon on the legacy UltimateCharge path while only energy-backed Digimon read/write Energy for ult readiness and drain.
  - Avoid Bevy’s 15-tuple QueryData limit by stitching UltGaugeMetadata into UnitQuerySnapshot through a sibling read-only query rather than widening ResolveActorsQuery.
  - Use one shared snapshot helper for ult readiness and resource reporting so Action availability and displayed current/max values cannot drift.
patterns_established:
  - Energy-backed combat resources should be exposed in UnitQuerySnapshot as optional component data plus compatibility scalars, then interpreted through a shared legality/resource helper.
  - Runtime finalize seams that own per-cast resource effects must each honor UltEffect::Reset for energy-backed actors instead of assuming one central post-pass.
observability_surfaces:
  - None added; verification remains test-led through action_query, damage_resolution, digimon_kits, and full windowed suite coverage.
drill_down_paths:
  - .gsd/milestones/M002/slices/S07/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S07/tasks/T02-SUMMARY.md
  - .gsd/milestones/M002/slices/S07/tasks/T03-SUMMARY.md
  - .gsd/milestones/M002/slices/S07/tasks/T04-SUMMARY.md
  - .gsd/milestones/M002/slices/S07/tasks/T05-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-21T21:13:57.018Z
blocker_discovered: false
---

# S07: S07

**Migrated Agumon’s ult loop to the real energy gauge so readiness flips exactly at full energy, ultimate casts drain energy back to zero, and non-opted-in Digimon stay on the legacy UltimateCharge path.**

## What Happened

S07 closed the runtime gap between Agumon’s authored ult-gauge metadata and the combat/query surfaces that decide whether Ultimate is available. T01 extended UnitQuerySnapshot plumbing with optional gauge metadata and Energy data without widening ResolveActorsQuery past Bevy’s 15-tuple QueryData limit; the extra component is stitched in via a sibling read-only query so downstream snapshot consumers keep compiling. T02 then routed both legality and ResourceKind::Ultimate reporting through a shared snapshot helper so energy-backed actors read current/max/ready from Energy while legacy actors continue to use UltimateCharge. T03 completed the drain path by zeroing Energy.current alongside UltimateCharge.current on every UltEffect::Reset finalize seam for energy-backed attackers, including the bounce-path runtime seam. T04 added an end-to-end Agumon integration test that performs the real loop—basic actions fill Energy, Ultimate stays locked while energy is below max even if legacy ult charge is primed, and a successful ultimate drains both Energy and UltimateCharge to zero—and patched timeline_exec so timeline-backed basics actually apply energy_grant during finalize. T05 swept the remaining regression surface and corrected the stale Holy Support roster contract: Agumon is now explicitly expected to opt into owner-keyed ult_gauge=energy metadata, while Gabumon remains metadata-free as the legacy control. Result: Agumon’s ult bar is genuinely energy-backed, becomes ready exactly at Energy.max, drains to zero on cast, and metadata-free Digimon keep the old gauge behavior unchanged.

## Verification

Verified every slice-plan command through gsd_exec in this closeout pass: (1) `cargo check --features windowed` plus `cargo test --features windowed --test action_query --test windowed_only` passed; (2) `cargo test --features windowed --test action_query` passed; (3) `cargo test --features windowed --test damage_resolution --test windowed_only` passed; (4) `cargo test --features windowed --test digimon_kits agumon_energy_gauge` passed, including `agumon_energy_gauge_fills_locks_and_drains_end_to_end`; and (5) `cargo test --features windowed` passed for the full windowed suite. Evidence came from gsd_exec runs 7f0d9b32-8da8-4223-be3f-07110325cd08, 5f712c24-327a-4550-9ba3-70bf534a0534, c5579ca0-9cb4-4f4e-8af4-73dbf9728de3, 38fcca49-ce74-4e57-bc4f-99019eea56f1, and 0574bd92-0eeb-4433-b9d2-c9e15fab4dcd, all exit 0.

## Requirements Advanced

- R011 — Preserved the full-kit Agumon combat loop while moving ult readiness/drain to the real energy gauge, keeping the on-screen kit path aligned with runtime resources.

## Requirements Validated

- R011 — Full `cargo test --features windowed` passed after the Agumon energy-backed migration, and `agumon_energy_gauge_fills_locks_and_drains_end_to_end` proved basic->fill->ult ready->drain semantics through the real runtime.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T05 discovered the remaining regression was not a generic snapshot-fixture shape issue but a stale Holy Support roster contract that still expected Agumon metadata to be empty; the slice fixed that contract and preserved Gabumon as the explicit legacy control.

## Known Limitations

This slice proves the headless/runtime energy-backed loop and preserves legacy fallback, but it does not migrate the whole roster to Energy-backed ult gauges; metadata-free Digimon intentionally remain on UltimateCharge.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/action_query/types.rs` — Added snapshot plumbing for optional ult gauge metadata and energy data.
- `src/combat/turn_system/resolve.rs` — Stitched UltGaugeMetadata into snapshot building via a sibling read-only query to avoid the Bevy query tuple cap.
- `src/combat/action_query/legality/shared.rs` — Added shared snapshot-based ult readiness helper for energy-backed vs legacy gauge evaluation.
- `src/combat/action_query/legality/action.rs` — Routed ultimate legality through effective snapshot-based gauge readiness.
- `src/combat/action_query/legality/resources.rs` — Reported Ultimate resource current/max from the same energy-backed readiness seam.
- `src/combat/turn_system/pipeline/timeline_exec.rs` — Applied energy grants for timeline-backed basics and drained energy on ultimate reset.
- `src/combat/turn_system/pipeline/paths/bounce/finalize.rs` — Zeroed Energy.current on UltEffect::Reset for energy-backed bounce finalize flows.
- `tests/digimon_kits/agumon_energy_gauge.rs` — Added end-to-end Agumon fill/lock/ready/drain regression coverage.
- `tests/digimon_kits/holy_support_roster_contract.rs` — Updated roster contract to expect Agumon’s metadata opt-in while preserving a legacy metadata-free control.
