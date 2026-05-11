---
estimated_steps: 16
estimated_files: 4
skills_used: []
---

# T03: Add real-binary shared-surface integration test

Lock the S05 proof into a targeted integration test that launches the actual CLI binary and asserts on the shared surfaces S05 owns.

Executor setup: load `tdd` and `verify-before-complete`; keep tests headless and deterministic per R100. Do not use ignored `.gsd/` fixtures or broad stale tests; this test should use only tracked files and the Cargo-provided binary path.

Steps:
1. Create `tests/combat_cli_shared_surface.rs` using `std::process::Command` and `env!("CARGO_BIN_EXE_combat_cli")`; set `BEVYROGUE_JSONL=1`, `BEVYROGUE_CLI_PROOF=1`, and any proof tick-limit env needed for deterministic completion.
2. Set `current_dir` to a non-root tracked/build directory such as `<manifest>/target` (creating it if necessary) so the test proves manifest-relative asset loading rather than cwd-relative luck.
3. Assert combined stdout/stderr contains `Action affordances`, canonical event output including `OnCombatBeat`, `OnKernelTransition`, and an action-resolved/damage/cast signal, plus `[CLI_PROOF] validation_snapshot` and `holy_support=grace=`.
4. Assert combined stdout/stderr does not contain `panicked`, `Message not initialized`, `[QUERY] Skill book unavailable`, `validation_snapshot_error`, or proof timeout/failure markers.
5. Add source-level guard assertions if useful to prove `src/bin/combat_cli.rs` names shared functions (`build_snapshot_from_ecs_with_sp`, `query_action_affordance`, `first_enabled_target_id`, `capture_validation_snapshot`) and does not mention `animation_sequence` or `qte`.

Must-haves:
- The test executes the real `combat_cli` binary, not a helper-only reconstruction of the app.
- The proof covers shared action query, event stream, beat event, kernel-observable event/state, and validation snapshot surfaces in one run.
- The test explicitly catches the known hidden-panic behavior where a worker thread panic can still produce exit 0.
- The test does not rely on full-suite fixture repair or ignored planning files.

Failure Modes (Q5): binary launch failure should include stdout/stderr in assertion output; cwd asset failure should fail before proof markers; event/snapshot marker absence should identify which shared surface disappeared.

Load Profile (Q6): shared resources are Cargo's test-built binary, local assets, stdout/stderr, and the target directory; each test run launches one short process; the 10x breakpoint is Cargo/build-process contention, not game-state scaling.

Negative Tests (Q7): negative assertions cover hidden panic text, missing message registration text, missing skill-book fallback text, snapshot errors, proof timeout markers, and source mentions of presentation metadata authority fields in CLI code.

## Inputs

- `src/bin/combat_cli.rs`
- `Cargo.toml`
- `assets/data/skills.ron`

## Expected Output

- `tests/combat_cli_shared_surface.rs`

## Verification

cargo test --test combat_cli_shared_surface -- --nocapture

## Observability Impact

Signals verified: stdout/JSONL event markers, proof snapshot marker, and failure-marker absence. Future agents inspect failures by running `cargo test --test combat_cli_shared_surface -- --nocapture`, which should print the CLI process output when assertions fail.
