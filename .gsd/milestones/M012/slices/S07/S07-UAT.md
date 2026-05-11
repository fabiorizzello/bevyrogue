# S07: S07: CLI/windowed affordance integration — UAT

**Milestone:** M012
**Written:** 2026-05-01T14:14:49.923Z

# S07 UAT — CLI/windowed affordance integration

## Preconditions
- Use the current M012 worktree with S07 changes applied.
- The combat data files and skill book must load successfully.
- Run in headless or windowed mode as noted per test; no manual code edits.

## Test Case 1: CLI non-interactive Basic selection uses query output
1. Start a combat turn where the active actor has at least one enabled Basic target.
2. Run the CLI in non-interactive mode.
3. Observe the automatically chosen target.

**Expected:** The CLI selects the first target whose `TargetAffordance.status` is `Enabled`, using the shared query output rather than local KO/team rules. The chosen target should be an enemy enabled by the query surface, and the turn should proceed without inventing a fallback intent.

## Test Case 2: CLI interactive action menu shows truthful states
1. Start a turn where Basic, Skill, and Ultimate affordances include a mix of enabled, disabled, deferred, and hidden states.
2. Open the CLI action menu.
3. Inspect the labels and reason text for each option.
4. Attempt to choose a disabled or hidden option.

**Expected:** Only enabled actions can be emitted. Disabled/deferred/hidden entries are visible with query reason codes, and choosing them does not produce an intent. Canceling the prompt falls back to an enabled query-backed choice if one exists.

## Test Case 3: Revive-like target lists include KO allies without special-case UI logic
1. Load a revive-like skill whose legal targets include KO allies only.
2. Ensure the combat snapshot includes one KO ally, one live ally, and one enemy.
3. Open the target menu in CLI.
4. Try selecting each listed target.

**Expected:** The KO ally is shown as `Enabled`. The live ally and enemy remain visible but are disabled with the canonical query reason codes. Selecting a disabled target is blocked before any intent is emitted.

## Test Case 4: Windowed action and target affordances come from the same query surface
1. Launch the project with `--features "windowed"`.
2. Open the combat panel on an active unit.
3. Verify the action row for Basic/Skill/Ultimate.
4. Start a pending action and inspect ally/enemy target cards.
5. Click an enabled KO ally target for a revive-like action.

**Expected:** Button enablement comes from `ActionStatus::Enabled`, disabled/deferred/hidden actions are surfaced with query reasons, and target cards are clickable only when `TargetAffordance.status` is `Enabled`. Ally and enemy cards both participate in target selection, so revive-like KO ally targeting works without a UI branch for specific skill IDs.

## Test Case 5: Stale pending actions are cleared safely
1. Open the windowed combat panel and select a pending action.
2. Change the combat state so the selected action becomes illegal or unavailable.
3. Observe the panel after the state update.

**Expected:** The pending action is cleared or disabled before it can emit a stale intent. No locally guessed legality path should override the query result.

## Test Case 6: Snapshot fidelity and engine parity remain separate
1. Inspect the consumer tests or run them directly.
2. Confirm the UI/CLI snapshot uses real `SpPool.current` for affordance display.
3. Confirm the engine-facing snapshot path still uses the S06 SP-bypass behavior.

**Expected:** UI/CLI affordances reflect the real visible SP state, while the engine validation path remains distinct and continues to enforce legality authoritatively.

## Edge Cases
- If the skill book is missing, the UI/CLI should surface a query-derived unavailable reason instead of guessing an intent.
- If no enabled target exists for Basic, the CLI should use the query-backed safe fallback or end the turn consistently with existing behavior.
- The CLI/windowed source scan should remain green, proving no `patamon_revive`, `ko.is_none()`, `can_pick_target && !enemy.is_ko`, or other hardcoded legality branches were reintroduced.

