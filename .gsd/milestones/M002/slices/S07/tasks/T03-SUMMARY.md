---
id: T03
parent: S07
milestone: M002
key_files:
  - src/combat/turn_system/pipeline/paths/bounce/mod.rs
  - src/combat/turn_system/pipeline/paths/bounce/finalize.rs
  - tests/damage_resolution/combat_resolution_apply.rs
  - tests/common/apply.rs
  - tests/damage_resolution/dr_pipeline.rs
key_decisions:
  - Apply the energy-backed ult drain at each finalize seam rather than trying to centralize it behind a single post-pass, because the target-shape pipelines already own their per-cast resource side effects.
duration: 
verification_result: passed
completed_at: 2026-05-21T21:00:08.054Z
blocker_discovered: false
---

# T03: Finished energy-backed ult drain on every runtime finalize path and added direct reset regressions for energy-backed vs legacy units.

**Finished energy-backed ult drain on every runtime finalize path and added direct reset regressions for energy-backed vs legacy units.**

## What Happened

Completed the remaining runtime drain seam by threading `UltGaugeMetadata` through the bounce pipeline finalize path, matching the existing single-target, self-target, multi-target, and timeline-backed reset behavior so `UltEffect::Reset` now zeroes `Energy.current` only for energy-backed attackers while still zeroing legacy `UltimateCharge.current` for compatibility. Added direct `apply_legacy_ops` regressions in `tests/damage_resolution/combat_resolution_apply.rs` covering both the energy-backed drain case and the legacy no-drain case, and updated the affected legacy test helpers/callsites to pass the new optional energy/metadata arguments introduced by the runtime seam.

## Verification

Ran `cargo test --features windowed --test damage_resolution --test windowed_only`; both targets passed, including the new damage-resolution regression coverage and the existing windowed-only surface.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test damage_resolution --test windowed_only` | 0 | ✅ pass | 6375ms |

## Deviations

Besides the planned drain logic, I had to finish a partially-wired bounce-path plumbing change (`application.rs` was already passing `gauge_meta_q` into bounce while the bounce path had not yet accepted/used it) and update legacy `apply_legacy_ops` test callsites to the expanded signature so the targeted verification could compile.

## Known Issues

None.

## Files Created/Modified

- `src/combat/turn_system/pipeline/paths/bounce/mod.rs`
- `src/combat/turn_system/pipeline/paths/bounce/finalize.rs`
- `tests/damage_resolution/combat_resolution_apply.rs`
- `tests/common/apply.rs`
- `tests/damage_resolution/dr_pipeline.rs`
