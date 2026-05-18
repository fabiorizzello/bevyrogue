# S03: Validator §L completo (contract and cross-asset)

**Goal:** Validator §L completo come contract test (tests/anim_fsm_validation.rs): entry exists, reachability (warn), exit reachable, dangling edges, priority unique, frame range in-bounds, command params reference exist (cross-asset vs skills.ron), StartQTE has headless_default, cancel coverage (warn).
**Demo:** tests/anim_fsm_validation.rs verde — Agumon valido passa l'intero §L; un fixture rotto per ogni check fa fallire le boot con DataError che nomina file+check; reachability/cancel solo warning.

## Must-Haves

- Validator §L complete; cross-asset checks pass on Agumon; broken fixtures rejected at boot.

## Proof Level

- This slice proves: Contract + Integration

## Integration Closure

Validator joins Clip and AnimGraph; param-ref cross-checks skills.ron.

## Verification

- DataError on boot failure

## Tasks

- [ ] **T01: Implement AnimGraph validator logic** `est:4h`
  Implement the validation logic in a new module src/combat/blueprints/anim_graph/validation.rs. Implement all checks listed in draft 02-02b §L.
  - Files: `src/combat/blueprints/anim_graph/validation.rs`
  - Verify: cargo check

- [ ] **T02: Add contract tests with broken fixtures** `est:2h`
  Create tests/anim_fsm_validation.rs with test cases for each validator check. Use broken fixture strings to verify that errors are reported correctly.
  - Files: `tests/anim_fsm_validation.rs`
  - Verify: cargo test --test anim_fsm_validation

- [ ] **T03: Wire validator to boot-time finish()** `est:1h`
  Update CombatPlugin or DataPlugin to run the validator during finish() or after load, emitting DataError if validation fails.
  - Files: `src/combat/plugin.rs`, `src/data/mod.rs`
  - Verify: cargo test --test anim_fsm_validation

## Files Likely Touched

- src/combat/blueprints/anim_graph/validation.rs
- tests/anim_fsm_validation.rs
- src/combat/plugin.rs
- src/data/mod.rs
