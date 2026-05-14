---
id: S03
parent: M018
milestone: M018
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/action_query.rs
  - src/combat/state.rs
  - tests/target_shape_bounce_chain.rs
key_decisions:
  - BounceSelector/RepeatPolicy kept Copy so TargetShape remains Copy — avoids pervasive refactor of pass-by-value call sites
  - DamageCurve stored on ResolvedAction at cast time, not re-read per hop — kernel stays zero-bias from skill data at execution time
  - TargetableSnapshot rebuilt each hop inside the Bounce loop so KOs shrink the candidate pool in real time
  - chain_bolt kept as inline test fixture (not added to skills.ron) to preserve the 74-skill catalog size assertion
  - Pool exhaustion breaks loop silently (no OnActionFailed) — deferred error signaling to a later slice
patterns_established:
  - select_bounce_hop() is a pure Rust fn (no ECS) taking a TargetableSnapshot slice — keeps multi-target resolution testable without a running Bevy App
  - Bounce hop loop rebuilds snapshot each hop so selector candidates always reflect current health/KO state
  - DamageCurve scaling is applied at the kernel level (pipeline.rs) by reading from ResolvedAction, not from the skill book
observability_surfaces:
  - Bounce hop loop: silent pool exhaustion break (no event emitted) — future diagnostic gap if hops complete fewer than expected
  - DamageCurve::PerHop length mismatch caught at skill-load time via validate_skill_def (logs parse error) — not at runtime
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T21:58:16.718Z
blocker_discovered: false
---

# S03: BounceSelector + RepeatPolicy DSL + generic Bounce kernel hop loop

**Full Bounce targeting pipeline shipped: DSL enums, struct variant migration, per-selector dispatcher, hop loop with DamageCurve scaling, and 4 integration tests — all 201 tests green.**

## What Happened

S03 delivered the complete Bounce targeting pipeline across three tasks.

**T01** introduced `BounceSelector` (NextSlot, LowestHp) and `RepeatPolicy` (NoRepeat, AllowRepeat) as Copy enums in `skills_ron.rs`, plus the pure `select_bounce_hop()` dispatcher that takes a `TargetableSnapshot` slice and `already_hit` set — no ECS, fully unit-testable.

**T02** migrated `TargetShape::Bounce` from a plain tuple to a named struct variant `{ hops: u8, selector: BounceSelector, repeat: RepeatPolicy }` and added `DamageCurve` (Constant, FalloffPct, PerHop) to `Effect::Damage`. All three validation gates (`validate_skill_def`, `target_shape_is_executable_now`, `target_status_for_unit`) were updated so Bounce-shaped skills with `hops >= 1` are treated as executable. T02 also kept `TargetShape` as `Copy` by ensuring all Bounce fields are Copy — avoiding a wide refactor of call sites. The `chain_bolt` fixture was created inline in tests (not in skills.ron) to preserve the 74-skill catalog size assertion.

**T03** wired the generic hop loop into `pipeline.rs`: it rebuilds the `TargetableSnapshot` each hop (so KOs are reflected in selector candidates), applies `DamageCurve` scaling per hop, enforces `NoRepeat` via `already_hit` insertion, and breaks silently on pool exhaustion (no `OnActionFailed` emitted). `DamageCurve` is stored on `ResolvedAction` at cast time — not re-read from the skill book at hop time — keeping the kernel zero-bias from skill data during execution. Four integration tests in `tests/target_shape_bounce_chain.rs` cover NextSlot no-repeat with falloff and mid-chain KO, LowestHp no-repeat with constant curve, LowestHp allow-repeat with per-hop curve, and silent pool exhaustion truncation.

## Verification

cargo check: 0 errors, 83 warnings (all pre-existing unused-fn warnings). cargo test --test target_shape_bounce_chain: 4/4 pass. cargo test --lib: 186/186 pass. Full cargo test suite: 201 tests, 0 failures across all integration and lib targets.

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

T01's verification was recorded as 'untested' in the task summary, but the code it produced (BounceSelector, RepeatPolicy, select_bounce_hop) was exercised by T03's integration tests (target_shape_bounce_chain). No source gaps were found; the 'untested' label reflected the state at T01 close before T03's tests existed.

## Known Limitations

Full per-hop damage application events (individual CombatEvent per hop) are deferred to a later slice. Pool exhaustion does not emit OnActionFailed — silent truncation only. DamageCurve::PerHop validation (length == hops) is enforced at skill-load time but not re-validated at runtime.

## Follow-ups

Per-hop CombatEvent emission for UI/log observability. OnActionFailed on pool exhaustion. Runtime DamageCurve::PerHop length guard in the kernel hop loop.

## Files Created/Modified

- `src/data/skills_ron.rs` — Added BounceSelector, RepeatPolicy enums; DamageCurve enum; select_bounce_hop dispatcher; Bounce struct variant in TargetShape; validator gates updated
- `src/combat/resolution.rs` — Updated Bounce arm in resolution fan-out to use struct fields; added DamageCurve to Effect::Damage
- `src/combat/turn_system/pipeline.rs` — Implemented generic Bounce hop loop with per-selector dispatch, repeat policy, and DamageCurve scaling
- `src/combat/action_query.rs` — Updated to carry DamageCurve on ResolvedAction
- `src/combat/state.rs` — Minor state type updates for ResolvedAction DamageCurve field
- `tests/target_shape_bounce_chain.rs` — 4 integration tests covering NextSlot, LowestHp, AllowRepeat, and pool exhaustion scenarios
