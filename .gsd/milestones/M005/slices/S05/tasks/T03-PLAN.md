---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Full-migration regression sweep + dep-isolation guard

Why: the migration touches the windowed render path and adds an optional-dep-backed handle map; the slice is only done when the headless dep-isolation invariant (R005/R016) still holds and both build flavors stay green. Do: run the four-command regression sweep and confirm each is green — (1) cargo test (full headless suite, which includes the standalone dependency_gating binary asserting bevy_enoki is absent from the dev graph and present in the windowed graph), (2) cargo build --features windowed (the full enoki render stack compiles windowed-gated), (3) cargo test --features windowed --test windowed_only (parse tests for all three assets + the generalized-seam source-contract test, all green), (4) cargo test --test dependency_gating (explicit dep-isolation guard). No new source files. Then perform the K001 manual sign-off step: this is a human/UAT gate — auto-mode cannot launch the windowed binary, so record in the slice summary that the user must run cargo winx and confirm Sharp Claws, Baby Flame, and Baby Burner contact bursts render through enoki and look better than the flat-quad placeholder. Done when: all four commands exit 0 and the K001 sign-off requirement is documented for the user.

## Inputs

- `src/windowed/render.rs`
- `tests/dependency_gating.rs`
- `tests/windowed_only.rs`

## Expected Output

- `src/windowed/render.rs`
- `tests/dependency_gating.rs`
- `tests/windowed_only.rs`

## Verification

cargo test
