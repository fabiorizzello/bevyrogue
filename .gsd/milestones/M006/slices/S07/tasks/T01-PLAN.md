---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Reproduce false MissingSkill in a headless test

Add a headless test that loads the full roster and asserts Renamon's skills resolve through the combat panel path without MissingSkill. Confirm it fails today because render.rs picks an arbitrary SkillBook.

## Inputs

- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/preview_cache.rs`

## Expected Output

- `A red headless test asserting Renamon skills resolve with no false MissingSkill`

## Verification

cargo test --test action_query (new case red, reproducing false MissingSkill)
