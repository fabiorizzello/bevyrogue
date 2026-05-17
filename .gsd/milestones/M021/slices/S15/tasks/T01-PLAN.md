---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Run and stabilize the final verification battery

Run the final milestone verification battery on the integrated tree, including full `cargo test`, headless and windowed checks, and the milestone grep gates. Fix any regressions exposed by the combined remediation work until the final verification battery is green.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S13/S13-PLAN.md`
- `.gsd/milestones/M021/slices/S14/S14-PLAN.md`

## Expected Output

- `.gsd/milestones/M021/slices/S15/tasks/T01-SUMMARY.md`

## Verification

cargo test
cargo check
cargo check --features windowed
rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'
rg "enum Effect" src/data/skills_ron.rs
rg "use bevy" src/combat/blueprints/

## Observability Impact

Provides current final evidence instead of relying on mid-milestone green runs.
