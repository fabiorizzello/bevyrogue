---
estimated_steps: 4
estimated_files: 4
skills_used: []
---

# T01: Run and stabilize the final verification battery

Run the final milestone verification battery on the integrated tree, including full cargo test, headless and windowed checks, and the milestone grep audits. Fix regressions exposed by the integrated tree until runtime verification is green, then record any remaining architecture-boundary grep hits truthfully instead of claiming those gates already pass.

This task now separates two outcomes:
1. runtime closeout must be green (`cargo test`, `cargo check`, `cargo check --features windowed`);
2. architecture grep audits are evidence to record, and any remaining hits must be called out explicitly in the task and slice summaries.

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
rg -n -e "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**' || true
rg -n "enum Effect" src/data/skills_ron.rs || true
rg -n "use bevy" src/combat/blueprints/ || true

## Observability Impact

Provides truthful final closeout evidence while separating green runtime verification from still-open architecture boundary audits.
