# S13: Close deferred foundation captures and boot invariants

**Goal:** Recover the deferred foundation proofs that were not captured when early M021 slices were administratively closed, and turn those invariants into explicit fresh evidence for milestone validation.
**Demo:** After this: cast_id, UltInstant, and 5-step turn pipeline are covered by fresh proof; DryRun equals Execute is green; invalid timeline ids fail at boot with explicit evidence.

## Must-Haves

- Fresh proof covers `cast_id` propagation on emitted combat and beat surfaces.
- Fresh proof covers `UltInstant` bypass behavior and the explicit 5-step turn pipeline ordering.
- Fresh proof covers `DryRun ≡ Execute` intent-stream parity.
- Fresh proof covers strict boot validation for bad timeline ids at `App::finish()`.
- Slice artifacts explicitly map the evidence back to P2, P3, P4, I2, and I5.

## Proof Level

- This slice proves: Focused runtime and boot-validation proofs on the current tree, plus closeout artifact evidence suitable for milestone validation.

## Integration Closure

This slice closes the validation gap around foundation-only invariants by proving them on the live tree and recording the evidence in normal slice artifacts instead of relying on historical intent or roadmap text.

## Verification

- Fresh targeted tests and UAT commands make cast_id propagation, UltInstant routing, turn-phase ordering, DryRun parity, and boot-validation failures observable from current artifacts.

## Tasks

- [x] **T01: Capture cast_id, UltInstant, and turn-pipeline proof** `est:0.75d`
  Audit existing tests and runtime paths for `cast_id`, `UltInstant`, and turn-phase ordering. Add or update focused integration tests so the current tree proves these deferred M021 contracts explicitly instead of relying on old slice roadmap claims. Capture exactly which context requirements each test discharges in the task summary.
  - Files: `src/combat/events.rs`, `src/combat/api/timeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `tests`
  - Verify: cargo test -- --nocapture cast_id || true
cargo test -- --nocapture ult_instant || true
cargo test -- --nocapture turn_phase || true

- [x] **T02: Prove DryRun equals Execute parity** `est:0.5d`
  Add or tighten parity coverage so the same compiled timeline run produces the same pending intent stream in `DryRun` and `Execute` modes. Reuse the shared preview/timeline path where possible and record the exact invariant proven by the test.
  - Files: `src/combat/api/skill_ctx.rs`, `src/combat/preview.rs`, `src/combat/api/runner.rs`, `tests`
  - Verify: cargo test --test skill_preview -- --nocapture
cargo test -- --nocapture dry_run || true

- [ ] **T03: Prove strict boot validation for invalid timeline ids** `est:0.5d`
  Exercise boot-time timeline validation directly: prove a bad registry reference fails at `App::finish()` with a deterministic test or focused harness, and align the resulting evidence with the M021 strict-boot-validation criterion.
  - Files: `src/combat/plugin.rs`, `src/combat/api/timeline.rs`, `tests`
  - Verify: cargo test -- --nocapture timeline_refs || true
cargo test -- --nocapture boot_validation || true

## Files Likely Touched

- src/combat/events.rs
- src/combat/api/timeline.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- tests
- src/combat/api/skill_ctx.rs
- src/combat/preview.rs
- src/combat/api/runner.rs
- src/combat/plugin.rs
