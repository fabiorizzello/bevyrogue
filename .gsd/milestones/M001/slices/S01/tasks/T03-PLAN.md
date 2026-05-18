---
estimated_steps: 9
estimated_files: 7
skills_used: []
---

# T03: Run full headless regression and tighten integration edges

Skills expected in executor frontmatter: bevy, rust-testing, verify-before-complete.

Why: The first two tasks can pass in isolation while still leaving broken public exports, accidental feature coupling, or regressions in existing headless tests. This task is the slice-level integration closure and should only make small fixes required by the full verification pass.

Do:
1. Run the dedicated S01 contract checks first: `cargo test --test anim_graph_parse` and `cargo test --test anim_graph_asset`.
2. Run the repository headless regression suite with `cargo test`.
3. If failures are caused by public exports, feature gates, derive bounds, Bevy plugin ordering, or test assumptions introduced in T01/T02, fix them in the relevant S01 files.
4. Do not broaden scope into S02 clip loading, S03 semantic validator checks, or S04 windowed hot-reload proof.
5. Ensure no S01 tests read `.gsd/`, `.planning/`, `.audits/`, or other gitignored planning artifacts.

Done when: All S01-specific tests and full `cargo test` pass headlessly, and any integration fixes remain inside the animation module, its asset fixture, or its tests.

## Inputs

- `src/lib.rs`
- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `src/animation/plugin.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_parse.rs`
- `tests/anim_graph_asset.rs`
- `Cargo.toml`

## Expected Output

- `src/lib.rs`
- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `src/animation/plugin.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_parse.rs`
- `tests/anim_graph_asset.rs`

## Verification

cargo test

## Observability Impact

Integration observability: full regression proves the new animation plugin/schema does not perturb existing headless data/combat tests and keeps asset diagnostics agent-inspectable through normal cargo test output.
