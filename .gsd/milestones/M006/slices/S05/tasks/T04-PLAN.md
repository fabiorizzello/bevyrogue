---
estimated_steps: 12
estimated_files: 12
skills_used: []
---

# T04: Run full slice verification gates

Expected executor skills/frontmatter: rust-development, cargo-nextest if desired, bevy, verify-before-complete.

Why: The slice touches binary-only windowed code, asset loading contracts, and headless dep-gating boundaries. A narrow source-contract pass is not enough; the final task reruns the full set of gates required by the milestone.

Do:
1. Run cargo test for the headless default suite.
2. Run cargo test --features windowed --test windowed_only for all windowed source-contract and presentation tests.
3. Run cargo test --test dependency_gating to prove bevy_enoki/windowed dependencies remain excluded from headless lib paths.
4. Run RUSTFLAGS='-D warnings' cargo build --features windowed to prove the windowed binary builds cleanly with warnings denied.
5. If any gate fails, fix the minimal owning task's files and rerun all gates in this task.

Done when: all automated gates pass on the current tree. Do not claim live visual sign-off; K001 manual cargo winx remains explicitly pending.

Failure Modes (Q5): A windowed feature build may expose binary-only type errors that source contracts miss; fix them in code, not by weakening tests. Dependency gating failure means a windowed symbol leaked into headless paths and must be removed.

Load Profile (Q6): Verification is local CPU/build workload only; no persistent services or external resources.

Negative Tests (Q7): dependency_gating is the negative test for headless leakage. Source contracts in the windowed_only suite are negative tests for engine Renamon hardcoding and single-entry regression.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/demo.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `assets/digimon/renamon/stance.ron`
- `assets/digimon/renamon/clip.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test
cargo test --features windowed --test windowed_only
cargo test --test dependency_gating
RUSTFLAGS='-D warnings' cargo build --features windowed

## Observability Impact

No new runtime signals; this task confirms the diagnostics and source-contract surfaces added earlier are green.
