---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Author data + windowed module for a ranged Digimon

Pick a ranged attacker from the existing roster data (e.g. one with a projectile skill), author its anim_graph/clip/stance/vfx assets and a src/windowed/digimon/<name>/mod.rs register() that populates only its own entries. No render core edits.

## Inputs

- `src/windowed/digimon/renamon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `assets/data/party.ron`

## Expected Output

- `A ranged Digimon registered through the seam, rendering and casting in windowed`

## Verification

cargo test (headless green); manual cargo winx shows the ranged Digimon render and cast
