---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path

Why: R005/D004 require a data-authored per-Digimon Stance FSM (not hardcoded); the player ticks it. Resolves the highest research risk (stance vs single-named-clip-range) via D039. Do: Add a whole-sheet named range to assets/digimon/agumon/clip.ron: "all": (start: 0, end: 92) (total_frames=93, max index 92; leaves the existing 8 ranges untouched so clip_geometry_parity stays green). Author assets/digimon/agumon/stance.ron: id "agumon_stance", clip "all", entry "idle", nodes idle(frames 53-58, modifier Loop), hurt(46-52), death(14-22), victory(76-92); transitions minimal and S01-supported only — idle self-cycles (Loop modifier; no transition needed to loop, or an Always self-edge), other nodes present but inert (no KernelEvent/UserInput predicates required in S01, those are S02+). No gameplay commands (T02 check will reject any). Add DEFAULT_ANIM_STANCE_PATHS (Agumon only — D042) and an AnimationStancePaths resource in src/animation/plugin.rs; load + validate stance graphs through the existing AnimationAssetPlugin pipeline (RonAssetPlugin<AnimGraph>, validate_anim_graph) and feed them to StanceGraphRegistry (T03 provenance). Done when: a test asserts stance.ron parses, validates with zero Error diagnostics (clip "all" makes idle/hurt/death/victory all in-range — D039), clip_geometry_parity still green, and StanceGraphRegistry resolves "agumon_stance". Decisions: D039, D042. Q7: a stance node outside [0,92] still fails FrameOutsideClipTotal (guard intact).

## Inputs

- `assets/digimon/agumon/clip.ron`
- `src/animation/anim_graph.rs`
- `src/animation/plugin.rs`
- `src/animation/registry.rs`
- `src/animation/validation/graph.rs`
- `tests/clip_geometry_parity.rs`

## Expected Output

- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon/stance.ron`
- `src/animation/plugin.rs`
- `tests/anim_stance_asset.rs`

## Verification

cargo test --test anim_stance_asset --test clip_geometry_parity
