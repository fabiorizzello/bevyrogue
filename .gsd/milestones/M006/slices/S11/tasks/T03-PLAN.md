---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Catalog discovery headless test

Add a headless test asserting the catalog discovers the current roster and that adding a fixture entry is picked up without code edits, locking the data-driven contract.

## Inputs

- `src/animation/plugin.rs`
- `assets/data/party.ron`

## Expected Output

- `Headless test proving roster discovery and pickup of an added fixture entry`

## Verification

cargo test --test assets_data (catalog discovery green)
