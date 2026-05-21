---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation

## Inputs

- None specified.

## Expected Output

- `src/animation/validation/types.rs`
- `src/animation/validation/command.rs`
- `src/animation/validation/graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_validation.rs`
- `tests/anim_gameplay_command_forbidden.rs`

## Verification

cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation
