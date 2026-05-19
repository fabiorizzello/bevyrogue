---
estimated_steps: 10
estimated_files: 2
skills_used: []
---

# T02: Author Agumon clip.ron and geometry parity test

Expected executor skills: rust-testing, tdd, verify-before-complete.

Why: R003 requires proof that the new clip.ron geometry is lossless relative to the existing authoritative Agumon atlas JSON. This task creates the real asset and a drift-resistant parity test.

Do:
1. Create assets/digimon/agumon/clip.ron using the Clip schema from T01.
2. Preserve source atlas geometry exactly: frame_size w=557 h=561, columns=10, rows=10, total_frames=95.
3. Preserve exact clip names and inclusive ranges: attack 0-8, block 9-13, death 14-22, heavy_attack 23-46, hurt 47-53, idle 54-59, skill 60-77, victory 78-94.
4. Add tests/clip_geometry_parity.rs that deserializes assets/digimon/agumon/clip.ron as Clip and assets/digimon/agumon_atlas.json via serde_json into local test-only structs.
5. Assert exact frame geometry and every named inclusive range.
6. Assert every JSON count equals end_index - start_index + 1 so off-by-one or source-data drift is visible.

Done when: cargo test --test clip_geometry_parity proves the authored RON asset is geometry-equivalent to the source atlas and the source atlas count fields are internally consistent.

## Inputs

- `src/animation/clip.rs`
- `assets/digimon/agumon_atlas.json`

## Expected Output

- `assets/digimon/agumon/clip.ron`
- `tests/clip_geometry_parity.rs`

## Verification

cargo test --test clip_geometry_parity

## Observability Impact

Adds an executable drift-detection surface for future agents: failures identify whether source JSON counts, clip names, frame metadata, or inclusive ranges diverged.
