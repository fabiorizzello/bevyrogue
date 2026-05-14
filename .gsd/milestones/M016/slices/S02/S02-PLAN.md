# S02: S02

**Goal:** Migrate Dorumon/DORUgamon Predator Loop ownership into typed RON custom signals and a dedicated Rust blueprint, proving that real predator-loop transitions still flow through the generic kernel, event, and validation snapshot surfaces without adding character-specific shared-system branches.
**Demo:** After this: Predator loop logic operates through the Dorumon blueprint.

## Must-Haves

- ## Must-Haves
- `assets/data/skills.ron` declares Dorumon/DORUgamon Predator Loop intent with typed `custom_signals`; presentation metadata and effect numbers remain non-authoritative.
- `src/data/skills_ron.rs` has a Dorumon-specific custom signal enum and `SkillCustomSignal::Dorumon` variant with `deny_unknown_fields` like existing blueprint signals.
- `src/combat/blueprints/dorumon.rs` maps Dorumon/DORUgamon signals to generic `CombatKernelTransition::PredatorLoop` values, and `src/combat/blueprints/mod.rs` only performs generic dispatch.
- New direct blueprint tests prove each signal maps to the expected `PredatorLoopTransition` without touching shared `resolution.rs` or turn-system character branches.
- New headless runtime integration tests emit the blueprint-produced kernel transitions, update `PredatorLoopState`, observe `PredatorLoopResolved` events, and verify `ValidationSnapshot` formatting contains predator-loop diagnostic state.
- Documentation updates name Dorumon/DORUgamon as the next migrated blueprint seam while preserving M015 authority boundaries.
- ## Threat Surface
- **Abuse**: No auth or user-account surface. The relevant abuse scenario is malformed or oversized RON custom signal values causing impossible predator-loop transitions; schema/tests must keep malformed data rejected by the shared kernel rather than interpreted in the blueprint as authority.
- **Data exposure**: None; combat state, event JSON, and snapshots expose gameplay diagnostics only.
- **Input trust**: `assets/data/skills.ron` is trusted project data at runtime but still parsed as external content; tests must prove typed signal decoding and shared-kernel rejection paths are authoritative.
- ## Requirement Impact
- **Requirements touched**: No Active requirements. This slice advances the validated M015 baseline around RON custom signals, per-Digimon blueprint ownership, generic kernel transitions, `CombatEvent`, and `ValidationSnapshot` surfaces.
- **Re-verify**: Predator-loop kernel tests, new Dorumon blueprint tests, new Dorumon runtime seam tests, and the combat authority audit script.
- **Decisions revisited**: CD001, CD004, CD005, CD006, CD007 must be honored, not changed.
- ## Verification
- `cargo test --test dorumon_blueprint --no-fail-fast`
- `cargo test --test dorumon_predator_runtime --no-fail-fast`
- `cargo test --test predator_loop_kernel --no-fail-fast`
- `python3 scripts/verify_combat_authority_audit.py`

## Proof Level

- This slice proves: Contract + headless integration proof. The slice proves typed RON signal parsing, direct blueprint-to-kernel transition mapping, and headless Bevy event/state/snapshot wiring for the Dorumon/DORUgamon Predator Loop. It does not claim full playable CLI/windowed UX validation for the migrated identity.

## Integration Closure

Upstream surfaces consumed: `assets/data/skills.ron`, `src/data/skills_ron.rs`, `src/combat/state.rs`, `src/combat/blueprints/mod.rs`, `src/combat/kernel.rs`, `src/combat/predator_loop.rs`, `src/combat/events.rs`, and `src/combat/observability.rs`. New wiring introduced: `SkillCustomSignal::Dorumon` dispatches through a dedicated Dorumon blueprint into existing generic `PredatorLoopTransition` handling. Remaining milestone work: later slices must migrate Renamon/Kyubimon and Agumon/Gabumon identities, and final milestone closure may still need broader real-binary proof.

## Verification

- Runtime signals: existing `OnKernelTransition` and `PredatorLoopResolved` events become observable for Dorumon/DORUgamon actions. Inspection surfaces: `ValidationSnapshot.predator_loop`, `format_validation_snapshot`, and existing JSON-serializable `CombatEvent` output. Failure visibility: malformed data, invalid targets, missing exploit/prey lock, cap reached, expired prey lock, and strain-blocked berserk remain exposed via `PredatorLoopBlockedReason` and snapshot `last_blocked_reason`. Redaction constraints: none beyond avoiding secrets/PII, which are not present in combat data.

## Tasks

- [x] **T01: Added owner-keyed custom signal envelopes and registry dispatch for blueprint routing.** `est:2h`
  Replace the planned per-Digimon `SkillCustomSignal` enum extension with a generic, typed custom-signal envelope and registry seam. `assets/data/skills.ron` should express Dorumon/DORUgamon intent as data addressed to a blueprint/plugin owner (for example owner/id plus signal name and typed payload), while `src/data/skills_ron.rs` validates only the shared envelope shape and leaves Digimon-specific payload interpretation to the registered blueprint. `src/combat/blueprints/mod.rs` should dispatch by registered blueprint owner instead of matching on hard-coded Digimon enum variants, and tests must prove unknown owners/signals or malformed envelopes are rejected rather than silently interpreted.
  - Files: `src/data/skills_ron.rs`, `src/combat/blueprints/mod.rs`, `src/combat/blueprints/dorumon.rs`, `assets/data/skills.ron`, `tests/digimon_signal_registry.rs`
  - Verify: cargo test --test digimon_signal_registry --no-fail-fast

- [x] **T02: Moved Dorumon Predator Loop decoding into a dedicated blueprint plugin.** `est:2h`
  Implement the Dorumon/DORUgamon Predator Loop as Dorumon-owned blueprint behavior rather than as a new shared character mechanic branch. The Dorumon blueprint module may keep private typed signal/payload/domain structs internally, but public combat wiring should see only generic registry dispatch and generic kernel/state/event transition surfaces. Direct blueprint tests must prove each Dorumon signal maps to the expected generic transition or rejection reason, and code should avoid adding Dorumon-specific cases to shared `resolution.rs`, turn-system branches, or shared non-blueprint mechanic modules.
  - Files: `src/combat/blueprints/dorumon.rs`, `src/combat/blueprints/mod.rs`, `src/combat/kernel.rs`, `src/combat/state.rs`, `src/combat/events.rs`, `tests/dorumon_blueprint.rs`
  - Verify: cargo test --test dorumon_blueprint --no-fail-fast

- [x] **T03: Added Dorumon predator runtime proof scaffolding and updated combat authority docs/audit markers.** `est:1h30m`
  Add headless runtime coverage showing Dorumon plugin-produced transitions update combat state through the generic kernel, emit canonical events, and appear in validation snapshots. Refresh combat authority docs and audit markers to describe Dorumon/DORUgamon as the first migrated plugin-owned Predator Loop seam, explicitly noting that `skills_ron.rs` does not enumerate each Digimon and that removing a Digimon should primarily remove its blueprint/plugin registration plus data. The audit should catch regressions that reintroduce Dorumon-specific shared-system branches or static signal enum variants.
  - Files: `tests/dorumon_predator_runtime.rs`, `tests/predator_loop_kernel.rs`, `src/combat/observability.rs`, `docs/combat_current.md`, `docs/contracts/combat_authority_map.md`, `docs/contracts/combat_mixed_pattern_drift_ledger.md`, `scripts/verify_combat_authority_audit.py`
  - Verify: cargo test --test dorumon_predator_runtime --no-fail-fast && cargo test --test dorumon_blueprint --no-fail-fast && cargo test --test digimon_signal_registry --no-fail-fast && python3 scripts/verify_combat_authority_audit.py

- [x] **T04: Confirmed that S02's static enum/shared-mechanic plan is blocked by the captured plugin-boundary feedback.** `est:15m`
  Assess the captured design feedback for S02 and determine whether the existing enum-based/shared-mechanic plan is still executable. This planning-only task records the blocker that Dorumon signals must not be added as static `SkillCustomSignal::Dorumon` variants and Predator Loop authority should be blueprint/plugin-owned rather than implemented as shared character mechanic branches.
  - Files: `.gsd/milestones/M016/slices/S02/S02-PLAN.md`
  - Verify: Planning-only verification: compare current S02 task requirements against CAP-749a38e2 and CAP-92aab67d and confirm the enum/shared-mechanic plan is invalid.

- [x] **T05: Repair Dorumon runtime proof and registry target** `est:1h`
  Update the Dorumon runtime proof so it asserts only the canonical drained `PredatorLoopResolved` event stream after kernel updates, not the transient `OnKernelTransition` envelope. Add the missing `tests/digimon_signal_registry.rs` integration target to prove Dorumon envelope parsing, registry routing, unknown-owner rejection, and malformed payload rejection, and wire it into Cargo so the verification command can run.
  - Files: `tests/dorumon_predator_runtime.rs`, `tests/digimon_signal_registry.rs`, `Cargo.toml`
  - Verify: cargo test --test dorumon_predator_runtime --no-fail-fast && cargo test --test digimon_signal_registry --no-fail-fast

- [x] **T06: Refresh predator-loop kernel snapshot fixture** `est:30m`
  Update `tests/predator_loop_kernel.rs` so its `ValidationSnapshot` fixture matches the current observability struct shape, including the battery-loop field now present on the snapshot. Preserve the existing intent of proving predator-loop event serialization and snapshot readability.
  - Files: `tests/predator_loop_kernel.rs`
  - Verify: cargo test --test predator_loop_kernel --no-fail-fast

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/blueprints/mod.rs
- src/combat/blueprints/dorumon.rs
- assets/data/skills.ron
- tests/digimon_signal_registry.rs
- src/combat/kernel.rs
- src/combat/state.rs
- src/combat/events.rs
- tests/dorumon_blueprint.rs
- tests/dorumon_predator_runtime.rs
- tests/predator_loop_kernel.rs
- src/combat/observability.rs
- docs/combat_current.md
- docs/contracts/combat_authority_map.md
- docs/contracts/combat_mixed_pattern_drift_ledger.md
- scripts/verify_combat_authority_audit.py
- .gsd/milestones/M016/slices/S02/S02-PLAN.md
- Cargo.toml
