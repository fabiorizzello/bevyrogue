---
id: T01
parent: S10
milestone: M021
provides:
  - Patamon Holy Support transport fully on Blueprint owner envelope; shared HolySupport kernel variant no longer emitted by any Patamon signal path
key_files:
  - src/combat/blueprints/patamon/identity.rs
  - src/combat/blueprints/patamon/signals.rs
  - src/combat/blueprints/patamon/mod.rs
  - tests/patamon_blueprint_seam.rs
  - tests/holy_support_resolution.rs
key_decisions:
  - Restored pub use for HolySupportRejectReason/HolySupportStep/HolySupportTransition in identity.rs so mod.rs can re-export them at the patamon module boundary
  - Removed HolySupport fallback arm from apply_holy_support_transitions_system; applier now only decodes Blueprint { owner: "patamon", ... } envelopes
  - Tests assert CombatKernelTransition::Blueprint { owner: "patamon", name, payload } instead of HolySupport variant
patterns_established:
  - Blueprint owner envelope is the sole dispatch surface for Patamon; typed HolySupportTransition remains internal to state mutation only
observability_surfaces:
  - HolySupportState resource (grace, martyr_light_marked/consumed, last_signal) remains inspectable via capture_validation_snapshot
duration: ~20m
verification_result: passed
completed_at: 2026-05-17T05:25:00Z
blocker_discovered: false
---

# T01: Migrate Patamon Holy Support transport onto the Blueprint owner envelope

**All Patamon signal paths now emit `CombatKernelTransition::Blueprint { owner: "patamon", name, payload }` exclusively; the shared `HolySupport` kernel variant is no longer produced by any Patamon dispatch path.**

## What Happened

The uncommitted work-in-progress already had `signals.rs` and the `HolySupportHook` fan-out migrated to `Blueprint`. Two issues remained:

1. **Broken re-exports**: The in-progress changes converted `pub use crate::combat::kernel::{HolySupportRejectReason, HolySupportStep, HolySupportTransition}` in `identity.rs` to a plain `use`, breaking the `mod.rs` re-export chain and causing three E0603 compile errors.

2. **Stale fallback arm**: `apply_holy_support_transitions_system` still had a `CombatKernelTransition::HolySupport(...)` match arm alongside the new `Blueprint { owner }` arm.

3. **Tests still asserting `HolySupport` variant**: Both test files compared `transitions_for_action` and `emit_transitions` output against the old `HolySupport(...)` variant.

Fixed by:
- Restoring `pub use crate::combat::kernel::{HolySupportRejectReason, HolySupportStep, HolySupportTransition}` in `identity.rs`
- Removing the `HolySupport` fallback arm from `apply_holy_support_transitions_system`
- Updating `tests/patamon_blueprint_seam.rs` and `tests/holy_support_resolution.rs` to assert `Blueprint { owner: "patamon", name: "build_holy_support_grace", payload: Amount(1) }` instead of `HolySupport(build_grace(1))`

The `HolySupportState.last_signal: Option<HolySupportTransition>` field remains unchanged — the decoder converts Blueprint payloads into typed `HolySupportTransition` values for state mutation, which tests still assert on.

## Verification

```
cargo test --test patamon_blueprint_seam
cargo test --test holy_support_resolution
```

Both test suites pass. Pre-existing failure in `combat_coherence::s_m008_s06_break_follow_up_and_ult_timing_trace` confirmed independent of these changes (verified by git stash).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test patamon_blueprint_seam` | 0 | pass (7/7) | ~5s |
| 2 | `cargo test --test holy_support_resolution` | 0 | pass (2/2) | ~5s |
| 3 | `cargo check` | 0 | clean compile | ~10s |

## Diagnostics

Patamon Holy Support state remains inspectable via:
- `app.world().resource::<HolySupportState>()` → grace, markers, last_signal
- `capture_validation_snapshot` / `format_validation_snapshot` → "holy_support=grace=N/3 last=build(N)"

## Deviations

None — all planned changes executed as described. The implementation was 90% done in the stashed WIP; this task completed the remaining re-export fix, fallback removal, and test updates.

## Known Issues

`s_m008_s06_break_follow_up_and_ult_timing_trace` in `combat_coherence.rs` fails, but is pre-existing and unrelated to Patamon.

## Files Created/Modified

- `src/combat/blueprints/patamon/identity.rs` — Restored `pub use` for kernel types; removed `HolySupport` fallback arm from applier
- `tests/patamon_blueprint_seam.rs` — Updated two tests to assert Blueprint envelope instead of HolySupport variant
- `tests/holy_support_resolution.rs` — Updated transition and emitted assertions to Blueprint envelope
