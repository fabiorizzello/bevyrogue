# S15: Final milestone closeout evidence

**Goal:** Re-establish final milestone-closeout evidence on the current tree after the remediation proofs land, so M021 can be revalidated without depending on stale or intermediate slice claims.
**Demo:** After this: full cargo test is green again, final grep and closeout UAT evidence are recorded, and M021 is ready for validation rerun.

## Must-Haves

- Fresh final `cargo test` passes on the integrated tree.
- Final grep gates and both cargo check modes pass and are recorded in the slice artifacts.
- Slice artifacts tie the final evidence back to the milestone success criteria and validation gaps.
- M021 is ready for a clean validation rerun without new remediation needs.

## Proof Level

- This slice proves: Full-suite and final-closeout verification on the integrated tree, with summaries tied back to the milestone success criteria.

## Integration Closure

This slice converts the earlier remediation work into final milestone-closeout evidence by rerunning the full suite, the grep gates, and the closeout checks on the final integrated tree.

## Verification

- Produces the final closeout artifacts needed by milestone validation: fresh full-suite evidence, grep evidence, and a truthful final UAT trail.

## Tasks

- [x] **T01: Run and stabilize the final verification battery** `est:0.75d`
  Run the final milestone verification battery on the integrated tree, including full `cargo test`, headless and windowed checks, and the milestone grep gates. Fix any regressions exposed by the combined remediation work until the final verification battery is green.
  - Files: `src`, `tests`, `tools`
  - Verify: cargo test
cargo check
cargo check --features windowed
rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'
rg "enum Effect" src/data/skills_ron.rs
rg "use bevy" src/combat/blueprints/

- [x] **T02: Capture final closeout evidence for validation rerun** `est:0.25d`
  Write the closeout narrative that maps the fresh final evidence back to the milestone success criteria, requirement gaps, and cross-slice integration proof so the next validation pass can succeed from artifacts alone.
  - Files: `.gsd/milestones/M021/slices/S15`, `.gsd/milestones/M021`
  - Verify: test -f .gsd/milestones/M021/slices/S15/S15-PLAN.md

## Files Likely Touched

- src
- tests
- tools
- .gsd/milestones/M021/slices/S15
- .gsd/milestones/M021
