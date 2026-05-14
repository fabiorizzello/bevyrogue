---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T01: Add Effect::Heal variant, TargetShape::AllAllies, OnHealed event, and validator

Introduce the data-model surface area for Heal. Add `Effect::Heal { amount_pct_max_hp: u32, target: TargetShape }` to the skill DSL, extend `TargetShape` with an `AllAllies` arm, add `CombatEventKind::OnHealed { amount: i32, hp_after: i32 }`, and extend `validate_skill_def` so Heal rejects enemy-side target shapes (Bounce/AllEnemies/Blast). Also add `AllAllies` to the executable-now whitelist used by legality checks. No behaviour wiring in this task — only the type/enum surface and exhaustiveness fallout (match arms in resolution.rs / follow_up.rs that today inspect `Effect` must compile without behavioural change).

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`
- `.gsd/milestones/M019/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`

## Verification

cargo check passes (proves enum + match exhaustiveness across all sites). cargo test runs and remains green — no behavioural change yet, only new variants.

## Observability Impact

Adds OnHealed variant to CombatEventKind. Serde derive on CombatEventKind already serializes new variants — no jsonl_logger.rs changes required.
