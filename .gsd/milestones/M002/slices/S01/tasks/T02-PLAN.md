---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation

In src/animation/validation/types.rs add AnimationValidationCheck::GameplayCommandForbidden and AnimationValidationReason::GameplayCommandInAnimGraph. In validation/graph.rs add check that EmitDamage/EmitStatus/EmitHeal in node.on_enter OR node.cues produces an Error diagnostic. Remove EmitDamage block from both production anim_graph.ron files. Add executable anti-DRY test asserting the live loaded agumon graph contains zero gameplay commands.

## Inputs

- None specified.

## Expected Output

- `cargo test green`
- `Both production graphs gameplay-command-free`
- `Anti-DRY test enforces D001`

## Verification

cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation
