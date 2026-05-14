---
id: S03
parent: M017
milestone: M017
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - status_amp_pct returns i32 multiplier (100 or 115) not a float — consistent with existing integer arithmetic in the damage pipeline
  - DamageBreakdown.status_amp_pct kept with underscore prefix in apply_effects destructuring — unused by OnDamageDealt for now, retained for log/snapshot symmetry
  - Self-targeting path in pipeline.rs passes None for defender_status — attacker == defender branch, no separate StatusBag ref without query restructuring
  - chilled_speed_delta uses integer division (base_speed / 5) matching −20% rounded toward zero
  - StatusBag already present at index 7 in the AV-gain query tuple — no query schema change needed, just renamed the wildcard
  - as_deref() used on Option<Mut<StatusBag>> to get &StatusBag without consuming the mutable ref
  - Heated DoT restructured to fire before stun-skip early-return — canon §H.1 requires DoT to bypass stun
  - StatusBag::apply used directly in test setup to pre-seed Heated/Chilled — simpler than routing through skill pipeline, still exercises the amp path
  - add_message::<ActionValueUpdated>() must be registered in test apps using advance_turn_system — its av_event_writer MessageWriter fails validation otherwise
patterns_established:
  - Derived-read status modifier pattern: compute AV/speed delta at the gain site from StatusBag without mutating SpeedModifier — avoids stale delta after status expiry mid-round
  - status_amp_pct pure lookup pattern: isolate per-status amp lookup in status_effect.rs as a pure function, wire into calculate_damage as a fourth multiplicative factor
  - DoT-before-stun pattern: unconditional per-status DoT emission before the stun-skip early-return in the turn block ensures bypass semantics per canon §H.1
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T09:19:20.688Z
blocker_discovered: false
---

# S03: Heated + Chilled — damage amp% pipeline + DoT + speed mod

**Wired Heated/Chilled per-status semantics: +15% damage amp on matching tag, Heated DoT (4 HP Fire at turn-end bypassing stun), Chilled −20% AV-gain via derived-read helper; 4-case integration test all green.**

## What Happened

S03 delivered the full Heated and Chilled damage pipeline across five tasks.

**T01** added a pure `status_amp_pct(bag, tag) -> i32` lookup in `src/combat/status_effect.rs` returning 115 when (Heated && Fire) or (Chilled && Ice), 100 otherwise. Four unit tests confirmed all four branches. Using an integer multiplier (100/115) kept arithmetic consistent with the rest of the i32 damage pipeline.

**T02** extended `DamageBreakdown` with `status_amp_pct: i32` and rewired `calculate_damage` to accept `defender_status: Option<&StatusBag>`, applying the amp as a fourth multiplicative factor after tag_mod/tri_mod/break_mod. Both `apply_effects` call sites in `turn_system/pipeline.rs` now forward the defender's `&StatusBag`. All existing callers in `resolution_tests.rs` and the undocumented `tests/holy_support_resolution.rs` were updated to pass `None` (regression-safe). The self-targeting path passes `None` as well — attacker == defender without a separate StatusBag ref.

**T03** restructured the per-turn block in `turn_system/mod.rs` so Heated DoT fires unconditionally before the stun-skip early-return. For each `StatusEffectKind::Heated` instance the unit loses 4 HP (clamped, skipped if already KO), an `OnDamageDealt { amount:4, damage_tag:Fire, kind:Normal }` event is pushed via `emit_combat_event`, and `OnKO` follows if HP drops to zero. A Heated+Stunned unit still burns. Confirmed `follow_up.rs` listeners require no preceding `OnSkillCast` for `OnDamageDealt`.

**T04** added `chilled_speed_delta(bag, base_speed) -> i32` returning `-(base_speed / 5)` (integer division, rounded toward zero) when Chilled is present. In the AV-gain loop the pre-existing StatusBag slot (index 7) was renamed from `_` to `status_bag_opt`; the delta is derived via `as_deref().map(...)` and added to the speed sum before multiplication by `AV_PER_SPEED`. No `SpeedModifier` mutation — purely derived read so expiry is stale-free.

**T05** created `tests/status_amp_pipeline.rs` with four deterministic headless tests covering all S03 DoD scenarios: (A) Fire base=100 on non-Heated → 100; (B) Fire base=100 on Heated → 115; (C) Ice base=100 on Chilled → 115; (D) Heated unit takes a turn → `OnDamageDealt{amount:4, damage_tag:Fire}` in event stream and HP drops by 4. A gotcha: `advance_turn_system` requires `add_message::<ActionValueUpdated>()` registered in test apps or the `av_event_writer` MessageWriter fails validation. `StatusBag::apply` used directly in test setup to pre-seed Heated/Chilled without routing through the skill pipeline.

## Verification

cargo check: clean (warnings only, no errors). cargo test --test status_amp_pipeline: 4/4 passed (fire_base100_non_heated_deals_100, fire_base100_heated_defender_deals_115, ice_base100_chilled_defender_deals_115, heated_unit_turn_emits_dot_4_fire). cargo test (full suite): 0 failures across all integration test files including combat_coherence, follow_up_chains, form_identity, status_*, ultimate_meter, validation_snapshot.

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

Undocumented call site tests/holy_support_resolution.rs also required None for defender_status — not listed in task plan but trivially fixed. T05 discovered that advance_turn_system requires add_message::<ActionValueUpdated>() registration in headless test apps.

## Known Limitations

Chilled AV-delta test coverage is unit-test only (chilled_speed_delta helper); no dedicated integration test asserting observable turn-order shift for Chilled units (marked as optional in slice plan, deferred to S04/S06 observability pass).

## Follow-ups

None.

## Files Created/Modified

- `src/combat/status_effect.rs` — 
- `src/combat/damage.rs` — 
- `src/combat/resolution.rs` — 
- `src/combat/turn_system/pipeline.rs` — 
- `src/combat/resolution_tests.rs` — 
- `src/combat/damage_tests.rs` — 
- `src/combat/turn_system/mod.rs` — 
- `src/combat/follow_up.rs` — 
- `tests/status_amp_pipeline.rs` — 
- `tests/holy_support_resolution.rs` — 
