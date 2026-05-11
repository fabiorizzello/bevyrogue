# S04: Agumon/Gabumon Twin Core Refinement Blueprint — UAT

**Milestone:** M016
**Written:** 2026-05-10T22:06:39.187Z

# S04: Agumon/Gabumon Twin Core Refinement Blueprint — UAT

**Milestone:** M016
**Written:** 2026-05-10

## UAT Type

- UAT mode: artifact-driven | live-runtime
- Why this mode is sufficient: The integration tests prove the contract between skills.ron, the blueprint registry, and the kernel transition emission.

## Preconditions

- The project compiles without errors.
- `assets/data/skills.ron` contains custom signals for Agumon and Gabumon.

## Smoke Test

`cargo test --test twin_core_integration`

## Test Cases

### 1. Agumon Skill Resolution emits Heated Tag

1. Run the integration test `skill_resolution_emits_twin_core_signals_through_blueprints`.
2. Observe that Agumon's `pepper_breath` skill is resolved.
3. **Expected:** The resolution generates a `CombatKernelTransition::Tag` for `TwinCoreDesignTag::Heated`.

### 2. Blueprint Registry Routing

1. Run the integration test and verify no "Unknown Owner" or "Unknown Signal" errors are logged.
2. **Expected:** Signals for both Agumon and Gabumon are correctly routed to their respective `dispatch` functions.

## Edge Cases

### Missing Amount Payload

1. (Simulated) Remove the `amount` from a custom signal in `skills.ron`.
2. **Expected:** The blueprint `dispatch` should return a `MalformedPayload` error (verified via code inspection of `amount_payload`).

## Failure Signals

- `cargo test` failure in `twin_core_integration`.
- Logs showing `CustomSignalDispatchError`.

## Not Proven By This UAT

- Full balance of the Twin Core mechanics in a live match.
- Visual/UI feedback for the tags (deferred to UI/presentation slices).

## Notes for Tester

This slice focuses on the architectural migration to blueprints. The mechanics themselves should behave as before, but the authority has shifted to the per-Digimon modules.

