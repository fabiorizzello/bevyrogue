---
estimated_steps: 5
estimated_files: 4
skills_used:
  - bevy
  - rust-best-practices
  - rust-testing
  - verify-before-complete
---

# T02: Prove Dorumon predator runtime state and event surfaces

**Slice:** S02 — Dorumon/DORUgamon Predator Loop Blueprint
**Milestone:** M016

## Description

Add a headless Bevy runtime test proving Dorumon blueprint transitions flow through the existing kernel runtime, `CombatEvent`, and `ValidationSnapshot` diagnostics.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| Bevy message runtime | Test should fail with missing `PredatorLoopResolved` event or unchanged state if systems are not registered or ordered correctly. | Not applicable; local deterministic update. | Malformed transitions should resolve to `PredatorLoopSignal::Rejected` with a concrete `PredatorLoopBlockedReason`. |
| Validation snapshot formatting | Test should fail if predator-loop state disappears from `format_validation_snapshot`. | Not applicable. | Snapshot should expose rejected transition/block reason rather than panic. |

## Load Profile

- **Shared resources**: Bevy `World`, `CombatEvent` messages, `PredatorLoopState`, `CombatKernelState`.
- **Per-operation cost**: A few message writes and one `app.update()` per scenario.
- **10x breakpoint**: Event volume is trivial in test; if expanded later, message-reader ordering would be the first fragility to inspect.

## Negative Tests

- **Malformed inputs**: Include a zero exploit amount or untracked target and assert `MalformedData` or `InvalidTarget` is exposed.
- **Error paths**: Assert attempting prey lock without exploit yields `MissingExploit` through `PredatorLoopResolved` and snapshot state.
- **Boundary conditions**: Assert exploit cap behavior remains covered by `tests/predator_loop_kernel.rs`; do not duplicate the full kernel suite here.

## Steps

1. Create `tests/dorumon_predator_runtime.rs` using `MinimalPlugins` and `register_combat_kernel_runtime(&mut app)`; create `ResolvedAction` fixtures carrying `SkillCustomSignal::Dorumon` from T01.
2. Seed `PredatorLoopState::track_target(action.target)` before target-scoped transitions so the test exercises the normal shared-kernel target contract.
3. Emit blueprint-produced `CombatKernelTransition::PredatorLoop` values as `CombatEventKind::OnKernelTransition`, call `app.update()`, and assert exploit stacks, prey-lock state, payoff consumption, `last_transition`, and `last_blocked_reason`.
4. Read `CombatEventKind::PredatorLoopResolved` messages and assert build exploit, apply prey lock, consume payoff, and at least one rejected diagnostic path appear with expected reasons.
5. Format a `ValidationSnapshot` and assert the rendered string exposes predator-loop state (`predator`, `exploit_cap=`, `targets=[...]`) and blocked-reason diagnostics when rejection is tested.

## Must-Haves

- [ ] The test uses real `register_combat_kernel_runtime` wiring and Bevy messages; no private pipeline calls.
- [ ] At least one failure path is asserted, such as missing exploit, invalid target, zero amount, cap overflow, or expired prey lock.
- [ ] `tests/predator_loop_kernel.rs` remains the authority for primitive rules; this test proves Dorumon seam integration only.
- [ ] The test demonstrates state, event, and snapshot observability for Dorumon-authored predator-loop transitions.

## Verification

- `cargo test --test dorumon_predator_runtime --no-fail-fast`
- `cargo test --test predator_loop_kernel --no-fail-fast`

## Observability Impact

- Signals added/changed: test coverage locks in `PredatorLoopResolved` and validation snapshot visibility for Dorumon-authored predator-loop transitions.
- How a future agent inspects this: `cargo test --test dorumon_predator_runtime -- --nocapture` and the `last_transition`/`last_blocked_reason` assertions.
- Failure state exposed: missing/invalid/malformed predator-loop requests remain visible as `PredatorLoopBlockedReason` instead of silent no-ops.

## Inputs

- `src/data/skills_ron.rs` — Dorumon signal enum from T01.
- `src/combat/blueprints/dorumon.rs` — Dorumon blueprint mapping from T01.
- `src/combat/blueprints/mod.rs` — dispatch seam from T01.
- `src/combat/kernel.rs` — kernel transition types and runtime registration.
- `src/combat/predator_loop.rs` — shared state and runtime transition application.
- `src/combat/events.rs` — `CombatEventKind::OnKernelTransition` and `PredatorLoopResolved` surfaces.
- `src/combat/observability.rs` — `ValidationSnapshot` and formatting surface.
- `tests/predator_loop_kernel.rs` — primitive behavior reference.

## Expected Output

- `tests/dorumon_predator_runtime.rs` — headless runtime integration test for Dorumon predator-loop events, state, and snapshots.
