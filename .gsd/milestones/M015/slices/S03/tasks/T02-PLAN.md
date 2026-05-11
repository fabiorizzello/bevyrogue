---
estimated_steps: 14
estimated_files: 8
skills_used: []
---

# T02: Wire complete kernel runtime and live beat events

Make the existing kernel registry and beat vocabulary participate in real headless action resolution before adding per-Digimon blueprint behavior.

Skills expected: `tdd`, `verify-before-complete`.

Steps:
1. Update `register_combat_kernel_runtime` so it installs all existing applier systems: Battery Loop, Predator Loop, Twin Core, Holy Support, and Precision Mind Game.
2. Call kernel runtime registration from real app builders, including the default app path and `src/bin/combat_cli.rs`; keep all wiring headless-safe and feature-gate-free except existing windowed UI gates.
3. Add deterministic action lifecycle beat emission in `resolve_action_system` / `pipeline::step_app`, writing both `OnCombatBeat` and canonical beat `OnKernelTransition` events where appropriate.
4. Remove or adjust duplicate applier additions in tests that would double-apply transitions after runtime registration.

Must-haves:
- Runtime registration creates the resources required by `ValidationSnapshot` in headless app paths.
- Existing Battery Loop and Predator Loop kernel tests still pass, proving registration changes did not regress established domains.
- Beat events are observable through `CombatEvent`, but presentation metadata still cannot decide gameplay.

Failure Modes (Q5): duplicate applier registration can double mutate state; missing app wiring causes snapshots to omit kernel resources; unordered event bridge systems can introduce one-frame lag, so prefer direct deterministic pipeline emission.
Load Profile (Q6): per action cost should remain linear in emitted beats/transitions and existing registry hooks; no unbounded per-frame accumulation.
Negative Tests (Q7): tests should catch missing beat events, missing runtime resources, and duplicate transition application.

## Inputs

- ``src/combat/kernel.rs` — existing `CombatKernelRegistry`, `CombatKernelTransition`, beat IDs, hooks, and appliers.`
- ``src/combat/turn_system/mod.rs` — live Bevy action resolution system.`
- ``src/combat/turn_system/pipeline.rs` — deterministic action pipeline helper used by tests/runtime.`
- ``src/main.rs` — primary app composition path.`
- ``src/bin/combat_cli.rs` — CLI app composition path that must share kernel wiring.`
- ``tests/event_stream.rs` — event bus expectations to extend for beat/kernel events.`

## Expected Output

- ``src/combat/kernel.rs` — all existing mechanic appliers registered by `register_combat_kernel_runtime`.`
- ``src/combat/turn_system/mod.rs` — live action resolution emits or forwards canonical beat/kernel events.`
- ``src/combat/turn_system/pipeline.rs` — test/runtime pipeline emits deterministic beat/kernel events from shared action flow.`
- ``src/main.rs` — primary app setup registers kernel runtime.`
- ``src/bin/combat_cli.rs` — CLI app setup registers kernel runtime without adding CLI-only combat logic.`
- ``tests/event_stream.rs` — regression coverage for live `OnCombatBeat` / beat transition emission.`
- ``tests/validation_snapshot.rs` — coverage still passes with runtime-registered kernel resources.`

## Verification

cargo test --test event_stream --test battery_loop_kernel --test predator_loop_kernel --test validation_snapshot

## Observability Impact

Signals added/changed: `OnCombatBeat` and beat-shaped `OnKernelTransition` events are emitted by the live action pipeline. How a future agent inspects this: run `cargo test --test event_stream` or inspect `CombatEvent` logs/snapshots. Failure state exposed: missing lifecycle beats, missing kernel resources, or double-applied transitions become test failures.
