---
estimated_steps: 5
estimated_files: 5
skills_used:
  - bevy
  - rust-best-practices
  - rust-testing
  - tdd
---

# T01: Add Dorumon custom signals and blueprint mapping

**Slice:** S02 — Dorumon/DORUgamon Predator Loop Blueprint
**Milestone:** M016

## Description

Add typed Dorumon/DORUgamon Predator Loop custom signals, wire generic blueprint dispatch, and prove direct mapping with integration-test fixtures.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| RON skill data parsing | Keep parser failure explicit through existing load/validation paths; do not silently ignore Dorumon signals. | Not applicable; local file parse only. | Serde `deny_unknown_fields` should reject malformed Dorumon signal payloads. |
| Blueprint dispatch | Return normal transition vectors for known signals; unknown variants should be impossible through the enum. | Not applicable. | Do not coerce bad values in the blueprint; let `PredatorLoopState` reject invalid amounts/targets. |

## Load Profile

- **Shared resources**: Local parsed skill data and small transition vectors.
- **Per-operation cost**: One custom-signal iteration per resolved action; O(number of custom signals), expected tiny.
- **10x breakpoint**: Trivial; excessive custom-signal lists would allocate larger vectors but remain bounded by authored RON content.

## Negative Tests

- **Malformed inputs**: Malformed RON custom-signal fields should be rejected by schema/serde, not ignored.
- **Error paths**: Zero or invalid amounts may be mapped by the blueprint but must be rejected by shared predator-loop state in kernel/runtime tests.
- **Boundary conditions**: Cover multiple Dorumon signals in one action and target-scoped signals using a non-default target ID.

## Steps

1. Extend `src/data/skills_ron.rs` with `DorumonCustomSignal` variants: `BuildExploit { amount: u16 }`, `ApplyPreyLock`, `ConsumePreyLockPayoff { amount: u16 }`, and `EnterBerserk`; add `SkillCustomSignal::Dorumon(DorumonCustomSignal)` while preserving `deny_unknown_fields`.
2. Create `src/combat/blueprints/dorumon.rs`; map each signal to `CombatKernelTransition::PredatorLoop(PredatorLoopTransition::...)`, using `ResolvedAction.target` for target-scoped signals and not inspecting skill IDs or mutating state.
3. Update `src/combat/blueprints/mod.rs` to route only `SkillCustomSignal::Dorumon` to the new module; do not add Dorumon branches to shared resolution, turn systems, or predator-loop state.
4. Add Dorumon signals to `assets/data/skills.ron`: build exploit on `draconic_edge`/`dorugamon_basic`, apply prey lock on `power_metal`, consume payoff on `cannonball`, and enter berserk on the selected ultimate predator burst while preserving existing damage/toughness effects.
5. Add `tests/dorumon_blueprint.rs` with `ResolvedAction` fixtures proving each signal maps to the expected generic `PredatorLoopTransition`, and that multiple Dorumon signals preserve order.

## Must-Haves

- [ ] RON remains declarative intent only; the blueprint emits generic transitions and shared `PredatorLoopState` enforces validity.
- [ ] Target-scoped transitions use the resolved action target, not hard-coded unit IDs.
- [ ] Existing Patamon/Tentomon dispatch and tests continue to compile.
- [ ] No Dorumon-specific branches are added to `src/combat/resolution.rs`, `src/combat/turn_system/`, or `src/combat/predator_loop.rs`.

## Verification

- `cargo test --test dorumon_blueprint --no-fail-fast`

## Observability Impact

- Signals added/changed: Dorumon-authored actions can now produce existing `CombatKernelTransition::PredatorLoop` values.
- How a future agent inspects this: `cargo test --test dorumon_blueprint -- --nocapture` and direct assertions in `tests/dorumon_blueprint.rs`.
- Failure state exposed: Runtime rejection remains exposed by shared predator-loop state in T02 rather than hidden in the blueprint.

## Inputs

- `src/data/skills_ron.rs` — existing custom signal schema and serde constraints.
- `src/combat/blueprints/mod.rs` — existing generic dispatch seam.
- `src/combat/blueprints/tentomon.rs` — current blueprint mapping pattern.
- `src/combat/kernel.rs` — `PredatorLoopTransition` constructors and `CombatKernelTransition` enum.
- `assets/data/skills.ron` — Dorumon/DORUgamon authored skill definitions.
- `tests/tentomon_blueprint.rs` — direct blueprint test fixture pattern.

## Expected Output

- `src/data/skills_ron.rs` — adds Dorumon custom signal schema and `SkillCustomSignal::Dorumon`.
- `src/combat/blueprints/mod.rs` — routes Dorumon signals to the new blueprint.
- `src/combat/blueprints/dorumon.rs` — maps Dorumon custom signals to generic predator-loop transitions.
- `assets/data/skills.ron` — declares predator-loop custom signals on Dorumon/DORUgamon skills.
- `tests/dorumon_blueprint.rs` — proves direct blueprint-to-transition mapping.
