---
estimated_steps: 15
estimated_files: 3
skills_used: []
---

# T02: Add env-gated CLI proof snapshot and deterministic exit

Add the minimal proof-mode contract the real binary needs: after shared combat action resolution has produced observable events/state, emit a validation snapshot from the live world and exit deterministically.

Executor setup: load `tdd` and `verify-before-complete`; keep `api-design` in mind for stable machine-readable marker names. Do not make proof mode a gameplay path — it is only an observation/exit control surface for tests.

Steps:
1. Add a `BEVYROGUE_CLI_PROOF=1` gate (and optionally a bounded tick limit such as `BEVYROGUE_CLI_TICK_LIMIT`) to insert proof configuration/state resources in `src/bin/combat_cli.rs`.
2. Reuse the headless exclusive-system pattern from `src/headless.rs`: once data is ready, units are spawned, at least one shared action has resolved or the `ActionLog` is non-empty, call `capture_validation_snapshot(world)` and print a stable line like `[CLI_PROOF] validation_snapshot: {format_validation_snapshot(&snapshot)}`.
3. Order the proof system after shared bootstrap/action/kernel systems and before/alongside loggers so JSONL/event output for the same proof run remains visible; write `AppExit::Success` only after the snapshot marker is emitted.
4. On proof-mode failure to reach readiness before the bounded tick limit, surface a clear non-success failure signal instead of relying on the old 6s timeout.

Must-haves:
- Proof mode observes the live ECS world and existing `capture_validation_snapshot` / `format_validation_snapshot`; it must not construct a fake snapshot or bypass kernel resources.
- Snapshot output includes initialized kernel-observable fields such as `holy_support=grace=` when runtime wiring is correct.
- Proof mode exits fast and deterministically after proof emission, without requiring UI/windowed features or wall-clock-sensitive assertions.
- The CLI remains interactive by default when proof mode is not enabled.

Failure Modes (Q5): if data/skills never load or no action resolves, proof mode should expose a bounded failure rather than hanging; if snapshot capture fails, print a `[CLI_PROOF] validation_snapshot_error:` marker and fail/exit non-success.

Load Profile (Q6): shared resources are the single CLI Bevy world, message queues, asset handles, and stdout; per proof run cost is a small fixed number of update ticks; 10x parallel runs may contend on Cargo build artifacts/stdout but not on game state.

Negative Tests (Q7): proof verification must fail on absent snapshot marker, `validation_snapshot_error`, hidden panic text, missing `holy_support=grace=`, or old missing-skill fallback output.

## Inputs

- `src/bin/combat_cli.rs`
- `src/combat/observability.rs`
- `src/headless.rs`

## Expected Output

- `src/bin/combat_cli.rs`

## Verification

BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli

## Observability Impact

Signals added: `[CLI_PROOF] validation_snapshot:` and bounded proof failure markers. Future agents inspect this with `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` and can localize failure to asset readiness, action resolution, snapshot capture, or message/runtime registration.
