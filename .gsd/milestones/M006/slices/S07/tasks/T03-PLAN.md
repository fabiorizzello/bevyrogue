---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Diagnose true MissingSkill with context

On the legitimately-missing branch, log the skill id and the book handle consulted (deduplicated) so a real miss is debuggable and distinct from the old arbitrary-book defect.

## Inputs

- `src/ui/combat_panel/render.rs`

## Expected Output

- `MissingSkill branch logs skill id + book handle, deduplicated`

## Verification

cargo test (headless green)
