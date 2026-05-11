# S02: Dorumon/DORUgamon Predator Loop Blueprint

**Goal:** Migrate Dorumon/DORUgamon Predator Loop ownership into typed RON custom signals and a dedicated Rust blueprint, proving that real predator-loop transitions still flow through the generic kernel, event, and validation snapshot surfaces without adding character-specific shared-system branches.
**Demo:** Dorumon/DORUgamon skills declare Predator Loop intent in `assets/data/skills.ron`; `src/combat/blueprints/dorumon.rs` maps that intent to generic `PredatorLoopTransition`s; headless tests observe `PredatorLoopResolved` events and `ValidationSnapshot.predator_loop` state.

## Must-Haves

- `assets/data/skills.ron` declares Dorumon/DORUgamon Predator Loop intent with typed `custom_signals`; presentation metadata and effect numbers remain non-authoritative.
- `src/data/skills_ron.rs` has a Dorumon-specific custom signal enum and `SkillCustomSignal::Dorumon` variant with `deny_unknown_fields` like existing blueprint signals.
- `src/combat/blueprints/dorumon.rs` maps Dorumon/DORUgamon signals to generic `CombatKernelTransition::PredatorLoop` values, and `src/combat/blueprints/mod.rs` only performs generic dispatch.
- New direct blueprint tests prove each signal maps to the expected `PredatorLoopTransition` without touching shared `resolution.rs` or turn-system character branches.
- New headless runtime integration tests emit the blueprint-produced kernel transitions, update `PredatorLoopState`, observe `PredatorLoopResolved` events, and verify `ValidationSnapshot` formatting contains predator-loop diagnostic state.
- Documentation updates name Dorumon/DORUgamon as the next migrated blueprint seam while preserving M015 authority boundaries.

## Threat Surface

- **Abuse**: No auth or user-account surface. The relevant abuse scenario is malformed or oversized RON custom signal values causing impossible predator-loop transitions; schema/tests must keep malformed data rejected by the shared kernel rather than interpreted in the blueprint as authority.
- **Data exposure**: None; combat state, event JSON, and snapshots expose gameplay diagnostics only.
- **Input trust**: `assets/data/skills.ron` is trusted project data at runtime but still parsed as external content; tests must prove typed signal decoding and shared-kernel rejection paths are authoritative.

## Requirement Impact

- **Requirements touched**: No Active requirements. This slice advances the validated M015 baseline around RON custom signals, per-Digimon blueprint ownership, generic kernel transitions, `CombatEvent`, and `ValidationSnapshot` surfaces.
- **Re-verify**: Predator-loop kernel tests, new Dorumon blueprint tests, new Dorumon runtime seam tests, and the combat authority audit script.
- **Decisions revisited**: CD001, CD004, CD005, CD006, CD007 must be honored, not changed.

## Proof Level

- This slice proves: contract + headless integration
- Real runtime required: yes, headless Bevy runtime only
- Human/UAT required: no

## Verification

- `cargo test --test dorumon_blueprint --no-fail-fast`
- `cargo test --test dorumon_predator_runtime --no-fail-fast`
- `cargo test --test predator_loop_kernel --no-fail-fast`
- `python3 scripts/verify_combat_authority_audit.py`

## Observability / Diagnostics

- Runtime signals: existing `OnKernelTransition` and `PredatorLoopResolved` events become observable for Dorumon/DORUgamon actions.
- Inspection surfaces: `ValidationSnapshot.predator_loop`, `format_validation_snapshot`, and existing JSON-serializable `CombatEvent` output.
- Failure visibility: malformed data, invalid targets, missing exploit/prey lock, cap reached, expired prey lock, and strain-blocked berserk remain exposed via `PredatorLoopBlockedReason` and snapshot `last_blocked_reason`.
- Redaction constraints: none beyond avoiding secrets/PII, which are not present in combat data.

## Integration Closure

- Upstream surfaces consumed: `assets/data/skills.ron`, `src/data/skills_ron.rs`, `src/combat/state.rs`, `src/combat/blueprints/mod.rs`, `src/combat/kernel.rs`, `src/combat/predator_loop.rs`, `src/combat/events.rs`, and `src/combat/observability.rs`.
- New wiring introduced in this slice: `SkillCustomSignal::Dorumon` dispatches through a dedicated Dorumon blueprint into existing generic `PredatorLoopTransition` handling.
- What remains before the milestone is truly usable end-to-end: later slices must migrate Renamon/Kyubimon and Agumon/Gabumon identities, and final milestone closure may still need broader real-binary proof.

## Tasks

- [ ] **T01: Add Dorumon custom signals and blueprint mapping** `est:1h30m`
  - Why: This closes the core authority migration: RON declares Dorumon/DORUgamon predator intent, a per-Digimon Rust blueprint owns interpretation, and shared predator-loop modules remain generic.
  - Files: `src/data/skills_ron.rs`, `src/combat/blueprints/mod.rs`, `src/combat/blueprints/dorumon.rs`, `assets/data/skills.ron`, `tests/dorumon_blueprint.rs`
  - Do: Add `DorumonCustomSignal`, route `SkillCustomSignal::Dorumon`, author the Dorumon blueprint, attach signals to Dorumon/DORUgamon skills, and add direct mapping tests for each transition and multi-signal order.
  - Verify: `cargo test --test dorumon_blueprint --no-fail-fast`
  - Done when: every authored Dorumon signal maps to the expected generic `PredatorLoopTransition`, no shared character branches are introduced, and existing Patamon/Tentomon dispatch still compiles.
- [ ] **T02: Prove Dorumon predator runtime state and event surfaces** `est:1h30m`
  - Why: Direct mapping is not enough; this task proves the migrated blueprint reaches the real headless Bevy kernel runtime, event stream, and validation snapshot diagnostics.
  - Files: `tests/dorumon_predator_runtime.rs`, `src/combat/predator_loop.rs`, `src/combat/events.rs`, `src/combat/observability.rs`
  - Do: Use `MinimalPlugins` and `register_combat_kernel_runtime`, seed `PredatorLoopState::track_target`, emit blueprint-produced `OnKernelTransition` events, assert `PredatorLoopState` mutation, read `PredatorLoopResolved` events, and assert `format_validation_snapshot` exposes predator diagnostics including a rejected path.
  - Verify: `cargo test --test dorumon_predator_runtime --no-fail-fast && cargo test --test predator_loop_kernel --no-fail-fast`
  - Done when: build/apply/payoff transitions and at least one rejection are visible through state, events, and snapshot output.
- [ ] **T03: Update combat authority docs for the migrated predator blueprint** `est:45m`
  - Why: The M015 authority map is the project contract; after code proof, docs and audit markers must truthfully record Dorumon/DORUgamon as migrated without overclaiming full roster or UI/CLI completion.
  - Files: `docs/combat_current.md`, `docs/contracts/combat_authority_map.md`, `docs/contracts/combat_mixed_pattern_drift_ledger.md`, `scripts/verify_combat_authority_audit.py`
  - Do: Update current combat docs and authority map evidence, adjust the mixed-pattern drift ledger if it still classifies Dorumon as future-only work, and update the audit script only if its explicit marker set needs the new Dorumon evidence.
  - Verify: `python3 scripts/verify_combat_authority_audit.py && cargo test --test dorumon_blueprint --test dorumon_predator_runtime --test predator_loop_kernel --no-fail-fast`
  - Done when: docs match executable proof, the audit passes, and later roster migrations remain clearly future work.

## Files Likely Touched

- `src/data/skills_ron.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/dorumon.rs`
- `assets/data/skills.ron`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`
- `docs/combat_current.md`
- `docs/contracts/combat_authority_map.md`
- `docs/contracts/combat_mixed_pattern_drift_ledger.md`
- `scripts/verify_combat_authority_audit.py`
