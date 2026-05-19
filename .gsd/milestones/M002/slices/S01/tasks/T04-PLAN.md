---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path

Add a whole-sheet named range to assets/digimon/agumon/clip.ron: 'all': (start: 0, end: 92). Author assets/digimon/agumon/stance.ron: id 'agumon_stance', clip 'all', entry 'idle', nodes idle(frames 53-58, modifier Loop), hurt(46-52), death(14-22), victory(76-92). Add DEFAULT_ANIM_STANCE_PATHS and AnimationStancePaths resource in plugin.rs; load + validate stance graphs through the existing AnimationAssetPlugin pipeline and feed them to StanceGraphRegistry.

## Inputs

- None specified.

## Expected Output

- `Test asserts stance.ron parses, validates with zero Error diagnostics`
- `clip_geometry_parity still green`
- `StanceGraphRegistry resolves 'agumon_stance'`

## Verification

cargo test --test anim_stance_asset --test clip_geometry_parity
