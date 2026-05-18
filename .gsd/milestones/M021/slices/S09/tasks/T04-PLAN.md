---
estimated_steps: 11
estimated_files: 15
skills_used: []
---

# T04: Run the slice verification sweep across headless and windowed builds

Skills: `bevy`, `tdd`, `verify-before-complete`.

Why: S09 is an architecture migration slice; completion claims are only credible after the narrow runtime tests and both build modes pass in the current codebase.

Do:
1. Run the slice-targeted Dorumon, Tentomon, passive, and event-stream tests after T01-T03 land.
2. Run headless and windowed `cargo check` to confirm the ownership move did not leak windowed dependencies into the combat runtime.
3. If a failure appears, classify it as slice regression vs. documented unrelated suite debt before completion.

Done when: all slice verification commands exit 0 in the current workspace and the results are ready to cite in task/slice completion evidence.

Negative checks:
- Windowed build must remain a compile-time feature gate only.
- Verification must use the current working tree, not stale prior-session output.

Observability impact: Produces fresh executable evidence that future agents can trust when deciding whether S09 is really done.

## Inputs

- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`
- `tests/passive_reactive_canon.rs`
- `tests/predator_loop_kernel.rs`
- `tests/event_stream.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test --test dorumon_blueprint
cargo test --test dorumon_predator_runtime
cargo test --test tentomon_blueprint
cargo test --test battery_loop_kernel
cargo test --test passive_reactive_canon
cargo test --test event_stream
cargo check
cargo check --features windowed

## Observability Impact

Fresh verification evidence across the slice’s runtime and build seams.
