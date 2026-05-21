---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Route Agumon Basic through Sharp Claws timeline data

## Inputs

- None specified.

## Expected Output

- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/agumon_sharp_claws_asset.rs`

## Verification

cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden
