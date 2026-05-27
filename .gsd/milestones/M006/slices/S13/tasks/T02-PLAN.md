---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Author data + windowed module for an aura/AoE Digimon

Add a second Digimon whose effect shape is an aura/AoE (different from projectile and melee) the same way, exercising keyed effect registration with a distinct effect topology.

## Inputs

- `src/windowed/digimon/renamon/mod.rs`
- `assets/data/party.ron`

## Expected Output

- `An aura/AoE Digimon registered through the seam, rendering and casting in windowed`

## Verification

cargo test (headless green); manual cargo winx shows the aura/AoE Digimon render and cast
