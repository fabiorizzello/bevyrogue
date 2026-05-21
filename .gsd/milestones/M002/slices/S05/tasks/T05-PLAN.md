---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T05: Baby Burner primary timeline + animation graph + thermal stack on impact

## Inputs

- None specified.

## Expected Output

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/agumon_baby_burner_primary.rs`

## Verification

cargo test --test agumon_baby_burner_primary --test agumon_baby_burner_reactive --test data_skills_ron_validation --test data_skills_ron_roundtrip --test anim_graph_asset --test anim_player_fsm --test anim_gameplay_command_forbidden --test clip_atlas_parity
