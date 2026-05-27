---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Repoint combat panel to canonical SkillBookHandle

Replace skill_books.iter().next() with a lookup against the canonical aggregated SkillBookHandle resource (the same one preview_cache.rs and combat_cli use). Preserve current rendering for the legitimately-missing case.

## Inputs

- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/preview_cache.rs`

## Expected Output

- `render.rs resolves skills via canonical SkillBookHandle; T01 passes`

## Verification

cargo test --test action_query (T01 green); cargo test (headless suite green)
