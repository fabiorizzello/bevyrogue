# S02: S02 — UAT

**Milestone:** M016
**Written:** 2026-05-09T16:03:14.164Z

# S02 UAT — Dorumon Predator Loop Blueprint Migration

## UAT Type
Headless contract + runtime integration proof.

This UAT covers:
- typed RON custom-signal decoding for the Dorumon/DORUgamon Predator Loop,
- owner-keyed registry dispatch into the Dorumon blueprint,
- generic kernel transition emission and application,
- canonical `PredatorLoopResolved` event observation,
- `ValidationSnapshot` / `format_validation_snapshot` predator-loop observability.

## Preconditions
- Canonical assets are present: `assets/data/skills.ron`.
- The headless combat runtime is available through the standard Bevy test harness.
- The target unit is tracked in `PredatorLoopState` before emitting Predator Loop transitions.

## Test Case 1 — Canonical Dorumon asset shape routes through blueprint envelopes
1. Load `assets/data/skills.ron`.
2. Resolve the `dorumon_ult` skill.
3. Inspect its custom signals.

Expected:
- The Dorumon entries use owner-keyed blueprint envelopes, not a bespoke top-level Digimon enum.
- The intent is expressed as Dorumon-owned `build_exploit` and `apply_prey_lock` signals with typed payloads.

## Test Case 2 — Blueprint dispatch emits generic predator-loop transitions
1. Resolve the canonical Dorumon skill through the action resolver.
2. Pass the resolved action through `blueprints::transitions_for_action_checked(...)`.
3. Inspect the returned transitions.

Expected:
- The Dorumon blueprint returns only generic `CombatKernelTransition::PredatorLoop(...)` values.
- The returned transitions map to `build_exploit(target, 2)` and `apply_prey_lock(target, 2)`.
- Unknown owners and malformed payloads are rejected instead of being interpreted.

## Test Case 3 — Headless runtime drains resolved predator events and mutates state
1. Start a headless Bevy app with the combat kernel runtime registered.
2. Insert `CombatState`, `SpPool`, and `ActionLog` resources.
3. Spawn an ally Dorumon source unit and an enemy target unit.
4. Track the target in `PredatorLoopState`.
5. Queue the blueprint-produced `OnKernelTransition` events into the message stream.
6. Run one update tick and drain the combat-event cursor.

Expected:
- The runtime emits canonical `PredatorLoopResolved` events after kernel application.
- The final predator-loop state contains the tracked target with exploit stacks and prey-lock state updated.
- The drained event stream does not depend on asserting against the transient input envelope.

## Test Case 4 — Validation snapshot includes predator-loop diagnostics
1. Capture a `ValidationSnapshot` from the same headless app after the runtime tick.
2. Format it with `format_validation_snapshot(...)`.
3. Inspect the snapshot string.

Expected:
- The formatted snapshot includes `predator_loop=exploit_cap=3 prey_lock_duration=2 berserk_threshold=50 ...`.
- The tracked target appears in the predator-loop target list as `7:e2:p2` for the canonical fixture.
- The current observability shape also includes the live `battery_loop` field.

## Edge Cases
- Unknown blueprint owner: rejected by the registry instead of being routed.
- Malformed payload: rejected by the Dorumon blueprint and never turned into a kernel transition.
- Untracked target: runtime kernel rejects the transition as `InvalidTarget`.
- Snapshot drift: the UAT should fail if `ValidationSnapshot` gains or loses required fields without test updates.

## Not Proven By This UAT
- Full playable CLI/windowed UX for the migrated Predator Loop.
- Balance tuning for Dorumon/DORUgamon outside the generic kernel contract.
- Remaining roster migrations (Renamon/Kyubimon, Agumon/Gabumon).
- Performance under load or long-run stability beyond the targeted headless proof.
