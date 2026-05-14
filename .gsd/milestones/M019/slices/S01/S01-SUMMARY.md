---
id: S01
parent: M019
milestone: M019
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/buffs.rs
  - src/combat/mod.rs
  - src/combat/bootstrap.rs
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
  - tests/dr_pipeline.rs
key_decisions:
  - DR sum is unclamped at the bag level — values >1.0 are legal; the (1.0-sum).max(0.0) factor in calculate_damage provides the effective clamp.
  - DR applied as final multiplicative step in calculate_damage, not as additive subtraction in resolution.rs.
  - follow_up.rs has its own local ResolveActorsQuery that must be kept in sync with resolution.rs's query shape.
  - Integration tests use apply_effects direct-call pattern (no Bevy world) for determinism and speed.
patterns_established:
  - DrBag accumulation: unclamped sum, factor applied at calculate_damage boundary.
  - Integration tests avoid Bevy world spin-up — use apply_effects directly for lightweight deterministic coverage.
observability_surfaces:
  - CombatEvent::Damage with amount=0 is emitted when DR clamps the final hit to zero — confirmed by dr_100pct_clamps_to_zero_and_event_emitted test.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-14T08:28:14.395Z
blocker_discovered: false
---

# S01: DR pipeline — BuffKind::DR multiplicative damage reduction primitive

**Added DrBag component with unclamped summation and integrated it as a multiplicative damage-reduction step in calculate_damage, verified by 6 integration tests in tests/dr_pipeline.rs covering single DR, stacked DR, DR+resist, DR during Break, 100% clamp, and >100% no-panic.**

## What Happened

Work proceeded across four tasks, all of which completed successfully.

**T01** confirmed that `DrBag`, the `sum_dr` helper, and bootstrap insertion were already present from a prior commit (2c09b85). The bag's accumulation policy was established: unclamped summation with no merging at the bag level — clamping is the caller's responsibility. `tick_all` returns `usize` (count dropped), mirroring `StatusBag::tick_all`.

**T02** integrated DR into `calculate_damage` as a final multiplicative step: `factor = (1.0 - sum_dr).max(0.0)`, then `final_damage.max(0)`. A `dr_reduction_pct` field was added to `DamageBreakdown` (capped at 100 for display; `sum_dr` remains unclamped internally). A key discovery: `follow_up.rs` maintains its own local `ResolveActorsQuery` that needed `DrBag` added separately, causing tuple-arity compile errors that had to be resolved. `pipeline.rs` was also updated for the new query shape.

**T03** wired `DrBag` through `resolution.rs` call sites and added a per-turn tick in `advance_turn_system` alongside `StatusBag::tick_all`. No expiry events are emitted for DR instances in this milestone — that is deferred to a later slice. The T03 summary notes a historical deviation: DR was initially applied as a post-`calculate_damage` subtraction (before T02 finalised the multiplicative approach), which was superseded.

**T04** wrote `tests/dr_pipeline.rs` with 6 integration tests using the `apply_effects` direct-call pattern (no Bevy world spin-up): single DR reduces damage by 30%, stacked DR sums unclamped, DR+resist stacks multiplicatively, DR applies when toughness already broken, 100% DR clamps damage to 0 with event emitted, and >100% DR produces no panic with damage also 0. All 6 passed, and the full suite (all test binaries) shows 0 failures.

## Verification

- `cargo test --test dr_pipeline`: 6/6 passed (single DR, stacked DR, DR+resist, DR during Break, 100% clamp, >100% no-panic).
- `cargo test` (full suite): all test binaries green, 0 failures across all integration and lib tests. Lib tests include 192+ passing tests covering damage matrix, buffs, status effects, follow-up, SP, ultimate, bootstrap, and more.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T03 notes that during its execution DR was applied as a post-calculate_damage subtraction (since T02 had not yet landed its damage.rs changes). This was superseded by the final multiplicative approach in T02. The final code is correct and consistent; the deviation only affected intermediate commit state.

## Known Limitations

No expiry events are emitted when a DrBag entry ticks down to zero — deferred to a later milestone alongside the general buff-expiry event system. DR is plumbed at the component/formula level only; there is no RON `Effect::DR` variant in this slice (that is S01's explicit scope boundary per the slice goal).

## Follow-ups

S02 (Heal primitive) and S03 (Cleanse primitive) can now proceed. When adding new components to the main ResolveActorsQuery in resolution.rs, remember to update follow_up.rs's local query in the same change to avoid tuple-arity errors.

## Files Created/Modified

- `src/combat/buffs.rs` — DrBag component, DrEntry, sum_dr helper, tick_all — already present from prior commit, confirmed in T01
- `src/combat/mod.rs` — DrBag re-export added
- `src/combat/bootstrap.rs` — DrBag::default() inserted on spawned units
- `src/combat/damage.rs` — DR multiplicative step + dr_reduction_pct field in DamageBreakdown
- `src/combat/resolution.rs` — DrBag plumbed through ResolveActorsQuery and passed to pipeline
- `src/combat/turn_system/mod.rs` — Per-turn DrBag tick added alongside StatusBag tick
- `src/combat/turn_system/pipeline.rs` — DrBag added to query, defender_dr forwarded to calculate_damage
- `src/combat/follow_up.rs` — DrBag added to local ResolveActorsQuery to fix tuple-arity errors
- `tests/dr_pipeline.rs` — 6 integration tests: single DR, stacked DR, DR+resist, DR during Break, 100% clamp, >100% no-panic
