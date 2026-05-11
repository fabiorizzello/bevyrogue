# S06: S06 — UAT

**Milestone:** M012
**Written:** 2026-05-01T13:19:41.975Z

# S06 UAT — Engine validation integration

## Purpose
Verify that the live Bevy combat pipeline uses the same legality decision as the pure query surface, and that illegal intents fail before declaration, mutation, or lifecycle events.

## Preconditions
- Milestone M012 is on the S06 code state.
- `cargo test-dev` is available in the workspace.
- The combat test fixtures can construct both a pure `CombatQuerySnapshot` and a minimal Bevy `App` using the same skill-book data.
- The test harness can drain `CombatEvent` with `MessageCursor`.

## Scenario 1 — Revive on a live ally is rejected with the same reason in query and engine
1. Build a fixture where the selected skill is revive-only and the target ally is alive.
2. Call `query_intent_legality()` with the snapshot and selected target.
   - Expected: `Err(TargetNotKo)`.
3. Inject the same `ActionIntent` into the Bevy message bus and run one `app.update()`.
4. Drain `CombatEvent`.
   - Expected: exactly one `OnActionFailed` with reason string `"TargetNotKo"`.
   - Expected: no `OnActionDeclared`, `OnActionPreApp`, `OnActionApplied`, or `OnActionResolved` events.
   - Expected: target HP and KO state are unchanged.

## Scenario 2 — Offensive skill on an ally is rejected before any lifecycle event
1. Build a fixture where an enemy-only offensive skill is targeted at a live ally.
2. Query legality.
   - Expected: `Err(WrongSide)`.
3. Submit the same intent through Bevy and update once.
4. Drain events and inspect state.
   - Expected: one `OnActionFailed("WrongSide")` only.
   - Expected: no lifecycle events.
   - Expected: no HP/KO mutation.

## Scenario 3 — KO enemy target remains a specific target failure, not a generic aggregate failure
1. Build a fixture where an offensive skill targets a KO enemy.
2. Query legality.
   - Expected: `Err(TargetKo)`.
3. Inject the intent into Bevy and update once.
4. Drain events.
   - Expected: one `OnActionFailed("TargetKo")`.
   - Expected: no lifecycle events and no mutation.

## Scenario 4 — Commander targeting is rejected explicitly
1. Build a fixture where the target is a commander unit and the skill does not allow commander targeting.
2. Query legality.
   - Expected: `Err(TargetIsCommander)`.
3. Inject the same intent into Bevy and update once.
4. Drain events.
   - Expected: one `OnActionFailed("TargetIsCommander")`.
   - Expected: no lifecycle events and no mutation.

## Scenario 5 — KO attacker and stunned attacker fail on attacker state before target checks
1. Build one fixture with the attacker KO and another with the attacker stunned.
2. Query legality for both.
   - Expected: `Err(AttackerKo)` for the KO attacker.
   - Expected: `Err(AttackerStunned)` for the stunned attacker.
3. Inject both intents through Bevy separately and update once per case.
4. Drain events.
   - Expected: each case emits exactly one matching `OnActionFailed`.
   - Expected: no lifecycle events and no mutation in either case.

## Scenario 6 — Non-active actor is rejected when turn order names a different active unit
1. Build a fixture where `TurnOrder.active_unit = Some(other_id)` and the submitted attacker is not `other_id`.
2. Query legality.
   - Expected: `Err(NotActiveUnit)`.
3. Inject the intent into Bevy and update once.
4. Drain events.
   - Expected: one `OnActionFailed("NotActiveUnit")`.
   - Expected: no lifecycle events and no mutation.

## Scenario 7 — Compatibility mode when `TurnOrder.active_unit` is `None`
1. Build a fixture with `TurnOrder.active_unit = None`.
2. Query legality for a valid intent.
   - Expected: the attacker is treated as active and the query succeeds.
3. Inject the same valid intent through Bevy and update once.
4. Drain events.
   - Expected: normal combat lifecycle events occur and the action resolves successfully.

## Scenario 8 — SP shortfall still follows the legacy lifecycle contract
1. Build a fixture where the attacker has insufficient SP for the selected skill.
2. Submit the intent through Bevy and update once.
3. Drain events.
   - Expected: the failure still occurs in the existing pipeline path, not the early legality guard.
   - Expected: lifecycle behavior remains consistent with the pre-S06 contract.

## Edge Cases
- Missing skill-book entry should reject as `MissingSkill` in the pure query path.
- Nonexistent target IDs should reject as `TargetNotFound` in the pure query path.
- A valid intent should still resolve normally, proving the new guard does not block legal actions.

## Pass Criteria
- Pure query and engine rejection reason match exactly for illegal intents.
- Illegal intents produce one `OnActionFailed` and no lifecycle events.
- No combat state mutation occurs on rejected intents.
- Existing SP lifecycle behavior is unchanged.
- `cargo test-dev --test engine_legality_integration` and `cargo test-dev` pass.
