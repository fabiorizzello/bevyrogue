---
estimated_steps: 13
estimated_files: 5
skills_used: []
---

# T01: Move Dorumon Predator Loop onto Blueprint owner transitions

Skills: `bevy`, `rust-best-practices`, `tdd`.

Why: S09’s first proof is that Predator Loop raw writes stop using the kernel-local `CombatKernelTransition::PredatorLoop` envelope while Dorumon keeps the same exploit/prey-lock runtime semantics and typed `PredatorLoopResolved` events.

Do:
1. Replace Dorumon custom-signal dispatch and wrapped-cycle hook emission with `CombatKernelTransition::Blueprint { owner: "dorumon", name, payload }` values, following the S08 Twin Core pattern.
2. Update `apply_predator_loop_transitions_system` to filter `OnKernelTransition` for Blueprint transitions owned by Dorumon, decode `name/payload` back into the typed `PredatorLoopTransition`, and keep the existing `EnterBerserk` kernel-strain behavior plus `PredatorLoopResolved` emission.
3. Keep `PredatorLoopState`, target tracking, and snapshot semantics unchanged so only the raw transport envelope moves.
4. Rewrite Dorumon-focused tests to assert on Blueprint raw transitions first, then on the unchanged typed resolved events and state snapshot.

Done when: Dorumon dispatch/hook code no longer emits `CombatKernelTransition::PredatorLoop`, the runtime applier accepts only `owner == "dorumon"` Blueprint transitions, and Dorumon tests prove both raw Blueprint writes and unchanged resolved Predator Loop behavior.

Failure modes / negative checks:
- Unknown owner or unknown signal must still be rejected.
- Malformed payloads must not silently mutate state.
- Non-Dorumon Blueprint transitions must be ignored by the Dorumon applier.

Observability impact: Raw JSON/event-stream writes become generic Blueprint owner events for Dorumon, while `PredatorLoopResolved` remains the typed seam consumed by snapshots and diagnostics.

## Inputs

- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`

## Expected Output

- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`

## Verification

cargo test --test dorumon_blueprint
cargo test --test dorumon_predator_runtime

## Observability Impact

Pins the Dorumon raw transition owner/name/payload shape without changing the typed resolved-event seam.
