---
estimated_steps: 12
estimated_files: 1
skills_used: []
---

# T03: Visual Validation Status and Hot-Reload Proof

Add a visual validation status to the windowed UI and perform the manual hot-reload proof required by R006.

Steps:
1. Update `src/windowed.rs` to display the current `AnimationValidationState` in the "Roster" side panel.
2. Use colored labels (Green for Ready, Red for Failed) and show error counts.
3. Perform the manual UAT:
   - Run `cargo run --features windowed`.
   - Edit a RON asset to introduce a typo.
   - Verify the UI reflects "FAILED".
   - Fix the typo.
   - Verify the UI reflects "READY".

Done when:
- The windowed UI shows the validation status and correctly reflects live edits.

## Inputs

- `src/windowed.rs`
- `src/animation/plugin.rs`

## Expected Output

- `src/windowed.rs`

## Verification

grep -q "AnimationValidationState" src/windowed.rs
