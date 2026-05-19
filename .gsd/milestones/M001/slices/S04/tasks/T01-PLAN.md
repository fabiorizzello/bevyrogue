---
estimated_steps: 7
estimated_files: 2
skills_used: []
---

# T01: Author Renamon animation assets

Author Renamon's animation assets to prove the generic roster-ready path (R007).

Steps:
1. Create `assets/digimon/renamon/`.
2. Author `assets/digimon/renamon/clip.ron` using geometry and ranges from `assets/digimon/renamon_atlas.json`.
3. Author `assets/digimon/renamon/anim_graph.ron` with a basic skill sequence (Idle -> Skill -> Idle) referencing renamon skills.

Done when:
- Both files exist and contain valid RON according to the AnimGraph and Clip schemas.

## Inputs

- `assets/digimon/renamon_atlas.json`
- `assets/data/digimon/renamon/skills.ron`

## Expected Output

- `assets/digimon/renamon/clip.ron`
- `assets/digimon/renamon/anim_graph.ron`

## Verification

ls assets/digimon/renamon/clip.ron assets/digimon/renamon/anim_graph.ron
