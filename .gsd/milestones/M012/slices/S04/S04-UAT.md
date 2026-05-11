# S04: Pure legality and affordance query API — UAT

**Milestone:** M012
**Written:** 2026-05-01T07:29:28.794Z

# S04 UAT — Pure legality and affordance query API

## Preconditions
- Build the current branch in a headless test environment.
- Use the canonical skill book and snapshot fixtures from `tests/action_affordance_query.rs`.
- No Bevy UI is required; this slice is validated entirely through pure query calls and deterministic tests.

## Scenario 1 — Offensive skill preflight is truthful
1. Create a snapshot with an active attacker, one live enemy target, one KO enemy, one ally, and one commander unit.
2. Query an implemented offensive single-target skill.
3. Expect `ActionStatus::Enabled`.
4. Expect the live enemy target to be `TargetStatus::Enabled`.
5. Expect the ally to be rejected with a wrong-side reason.
6. Expect the KO enemy to be rejected with a life-state reason.
7. Expect commander and self targets to be rejected with stable commander/self reasons.
8. Expect the returned target list to include toughness visibility for enemies only.

## Scenario 2 — Revive and damaged-target legality stay data-driven
1. Build a snapshot with one KO ally, one live ally, and one enemy target.
2. Query an implemented revive skill.
3. Expect the KO ally to be the only legal ally target.
4. Expect the live ally to be rejected with a not-KO reason.
5. Expect the enemy to be rejected with a wrong-side reason.
6. Query a damaged-target healing skill using `TargetHpRule::Damaged`.
7. Expect a damaged ally to be legal.
8. Expect a full-HP ally to be rejected with `TargetFullHp`.

## Scenario 3 — Deferred and hidden affordances never masquerade as usable
1. Query a skill whose implementation is deferred for a non-single shape.
2. Expect the action and all candidate targets to report deferred status with the stable unsupported-shape reason.
3. Query a hidden self-only form-identity-like skill.
4. Expect hidden action/target status, not enabled/disabled.
5. Confirm toughness exposure remains hidden when the skill is deferred or hidden.

## Scenario 4 — Resource and phase preconditions are surfaced with reasons
1. Query a skill while the actor is not the active unit.
2. Expect `NotActiveUnit` and retain target affordances.
3. Query the same skill in the wrong combat phase.
4. Expect `WrongPhase`.
5. Query with insufficient SP.
6. Expect the action to be disabled with `SpShortfall` and the returned resource details to show current vs required SP.
7. Query with insufficient ultimate charge.
8. Expect `UltimateNotReady` and the returned resource details to show current vs required ultimate charge.

## Edge Cases
- Missing actor or missing skill book entry must return stable missing-skill style failures rather than panic.
- A snapshot with no legal targets must disable the action with `NoValidTargets`.
- Ally toughness must remain hidden even when the unit has a toughness component internally.
- Enemy toughness with zero/maxless values must not be presented as a breakable bar.

## Expected Result
- The query surface always returns reason-bearing affordances derived from DSL metadata and immutable snapshot state.
- No per-skill UI hardcoding is required to explain legality or targetability.
- The same vocabulary can be consumed by future CLI/windowed adapters and engine validation.
