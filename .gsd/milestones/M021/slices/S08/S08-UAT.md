# S08: S08: Agumon + Gabumon migrated (Twin Core paired) — UAT

**Milestone:** M021
**Written:** 2026-05-16T22:15:54.132Z

## UAT Type
Integration / regression

## Preconditions
- Milestone M021 is active and S08 tasks T01-T04 are complete.
- The project builds with the current blueprint and registry layout.
- Test data for Agumon/Gabumon and the Twin Core path is available in the existing fixtures.

## Steps
1. Run `cargo test --test bouncing_fire_off_baseline`.
2. Run `cargo test twin_core`.
3. Inspect the test output for the Bouncing Fire OFF path and confirm it produces a single primary hit with no bounce when talent rank is 0.
4. Confirm the rank-1 Bouncing Fire path produces the expected primary hit plus exactly one bounce hop.
5. Confirm Twin Core dispatches through the Blueprint owner path rather than a kernel-local TwinCore variant.

## Expected Outcomes
- `cargo test --test bouncing_fire_off_baseline` passes.
- `cargo test twin_core` passes.
- OFF / rank 0 behaves as the baseline intent stream with no loop-driven divergence.
- Rank 1 behavior is deterministic and produces the expected bounce branch.
- Gabumon resolves Twin Core wiring through `blueprints::twin_core`.

## Edge Cases
- Talent rank 0 should not emit any bounce transition.
- The test should remain stable even if unrelated suite failures exist elsewhere in the repository.
- Twin Core wiring should remain isolated from kernel-local logic.

## Not Proven By This UAT
- It does not prove the entire repository test suite is green.
- It does not prove unrelated failures in follow_up_triggers or combat_coherence are fixed.
- It does not validate later milestones S09-S12.
