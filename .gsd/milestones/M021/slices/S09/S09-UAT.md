# S09: S09: Dorumon + Tentomon migrated (Predator Loop + Battery Loop) — UAT

**Milestone:** M021
**Written:** 2026-05-16T22:41:12.831Z

# S09: Dorumon + Tentomon migrated (Predator Loop + Battery Loop) — UAT

**Milestone:** M021
**Written:** 2026-05-16

## UAT Type

- UAT mode: live-runtime
- Why this mode is sufficient: The slice is about runtime transport and observability seams, so the acceptance proof must execute the real integration tests and build checks against the live code path.

## Preconditions

- The repository is in the current verified state.
- Rust toolchain and cargo are available.
- No local edits are required to replay the slice-level checks.

## Smoke Test

Run the targeted integration suite for the two migrated loops:

- `cargo test --test dorumon_blueprint`
- `cargo test --test tentomon_blueprint`

Expected: both suites pass and confirm Blueprint-envelope dispatch is still accepted by the owner modules.

## Test Cases

### 1. Dorumon Predator Loop transport

1. Run `cargo test --test dorumon_blueprint`.
2. Run `cargo test --test dorumon_predator_runtime`.
3. Confirm the runtime test still resolves the typed PredatorLoop seam.
4. **Expected:** raw Predator Loop writes use the generic Blueprint owner envelope and the owner-gated runtime ignores foreign or malformed events.

### 2. Tentomon Battery Loop determinism

1. Run `cargo test --test tentomon_blueprint`.
2. Run `cargo test --test battery_loop_kernel`.
3. Run `cargo test --test passive_reactive_canon`.
4. **Expected:** Battery Loop state still converges deterministically, wrapped-cycle reset behavior remains stable, and passive block reactions are unchanged.

### 3. Shared observability surfaces

1. Run `cargo test --test event_stream`.
2. Inspect the assertions around raw Blueprint owner/name/payload shape and typed resolved events.
3. **Expected:** the event stream still records the raw Blueprint owner boundary while preserving the downstream typed observability seams.

## Edge Cases

### Foreign Blueprint owner

1. Feed a Blueprint transition whose owner is not the loop owner.
2. **Expected:** the runtime applier ignores it and state does not mutate.

### Malformed payload

1. Feed a Blueprint transition with an invalid payload for the owner module.
2. **Expected:** the transition is ignored rather than partially applied.

## Failure Signals

- One or more targeted `cargo test` commands fail.
- Event-stream assertions stop seeing the raw Blueprint owner boundary or the typed resolved seam.
- Battery Loop or passive-reactive tests become nondeterministic or flaky.
- `cargo check` or `cargo check --features windowed` stops passing.

## Not Proven By This UAT

- The wider milestone goal of removing digimon-specific kernel coupling across the remaining roster.
- Full gameplay validation beyond the loop-specific integration tests.
- Future migration work in later slices.

## Notes for Tester

The important contract is not just that the tests pass, but that the raw transport envelope changed without changing the owner-gated runtime behavior or the typed resolved-event seam.
