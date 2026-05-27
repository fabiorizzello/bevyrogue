# S15: Prune windowed VFX test churn per spike 4 recommendation

**Goal:** Prune the windowed VFX test churn per spike 4's recommendation: cut the redundant Tier-1 contract test, thin the Tier-2 source-token tests to absence-guards only, and append an anti-churn rule to DECISIONS.md so future refactors don't re-introduce brittle source-shape assertions.
**Demo:** Tier-1 cut vfx_windowed_contracts.rs; Tier-2 thin source-token tests to absence-guards; anti-churn rule appended to DECISIONS.md; windowed tests green before and after

## Must-Haves

- vfx_windowed_contracts.rs removed (its real coverage subsumed by behavior tests); the remaining source-token tests assert only the durable invariant (engine code must not reference per-species ids) rather than exact file shape; an anti-churn rule is recorded in DECISIONS.md; cargo test --features windowed is green both before the prune (baseline) and after.

## Proof Level

- This slice proves: windowed suite green before and after the prune

## Verification

- None; this slice reduces test surface. Confirm the kept absence-guard still fails loudly if engine code reintroduces a hardcoded species id.

## Tasks

- [ ] **T01: Baseline-green, then cut the Tier-1 contract test** `est:S`
  Record cargo test --features windowed green as a baseline, then delete tests/windowed_only/vfx_windowed_contracts.rs whose coverage is subsumed by behavior tests (S08/S12 cast and effect proofs). Confirm no unique assertion is lost.
  - Files: `tests/windowed_only/vfx_windowed_contracts.rs`
  - Verify: cargo test --features windowed --test windowed_only (green after removal)

- [ ] **T02: Thin source-token tests to absence-guards** `est:M`
  Reduce the remaining source-token/source-contract tests to assert only the durable invariant (engine render core contains no per-species id) and drop assertions tied to exact file layout, so the S09/S10 module split does not break them gratuitously.
  - Files: `tests/windowed_only/renamon_extension_contract.rs`, `tests/windowed_only/agumon_module_extraction.rs`
  - Verify: cargo test --features windowed --test windowed_only (green); deliberately reintroduce a species id locally and confirm the guard fails, then revert

- [ ] **T03: Record the anti-churn rule** `est:S`
  Append a DECISIONS.md rule (via gsd_save_decision) stating that windowed presentation correctness is proven by behavior tests, and source-shape assertions are limited to the engine-no-species-id absence guard, to prevent future brittle-test churn.
  - Files: `.gsd/DECISIONS.md`
  - Verify: cargo test --features windowed --test windowed_only (green); DECISIONS.md contains the anti-churn rule

## Files Likely Touched

- tests/windowed_only/vfx_windowed_contracts.rs
- tests/windowed_only/renamon_extension_contract.rs
- tests/windowed_only/agumon_module_extraction.rs
- .gsd/DECISIONS.md
