# S05: S05 — UAT

**Milestone:** M012
**Written:** 2026-05-01T08:36:16.010Z

# UAT — S05 Resource Truth and Energy Caps

## Preconditions
- Use the current headless test harness and canonical skill book.
- Spawn fixtures with `Energy`, `RoundEnergyTracker`, and the standard combat components used by the resource-cap tests.
- For query-only checks, use the pure snapshot/query helpers from `src/combat/action_query.rs`; do not inspect ECS directly.

## Test Case 1 — Energy cap query is truthful when budget remains
1. Build a `UnitQuerySnapshot` with a partially used Energy budget (for example, secondary gain already below the 10-point cap).
2. Query the Energy-cap affordance for a secondary `GrantEnergy` request that still fits the remaining budget.
3. Repeat the query for an external gain request that still fits the remaining external budget.

**Expected:**
- The Energy-cap resource detail is `Enabled`.
- The reason code is non-failing / non-deferred.
- The query reports the remaining budget, not the requested amount.

## Test Case 2 — Energy cap query disables exhausted or over-budget gains
1. Reuse the snapshot from Test Case 1 and raise the used secondary budget to the full cap.
2. Query the same secondary `GrantEnergy` affordance again.
3. Repeat with an external budget at its cap.
4. Query a request larger than the remaining budget.

**Expected:**
- The Energy-cap resource detail is `Disabled`.
- The reason code is `EnergyCapReached`.
- No query path panics when the snapshot is incomplete or already capped.

## Test Case 3 — Live pipeline enforces the cap and reports actual applied energy
1. Spawn a combat app with a unit that has `Energy.current = 0`, a finite `Energy.max`, and a `RoundEnergyTracker`.
2. Resolve two same-round `GrantEnergy` effects through the live action/follow-up pipeline.
3. Read back the unit's `Energy` and the emitted `CombatEventKind::EnergyGained` events.

**Expected:**
- Total applied energy never exceeds the per-round budget.
- `Energy.current` increases only by the actually applied amount.
- `EnergyGained` events never overreport the requested amount after a cap or max clamp.

## Test Case 4 — Energy max clipping remains truthful
1. Spawn a unit whose `Energy.current` is already close to `Energy.max`.
2. Apply a `GrantEnergy` amount larger than the remaining headroom.
3. Inspect the resulting `Energy` and event stream.

**Expected:**
- Energy stops exactly at `Energy.max`.
- The event stream reports only the delta that was really applied.
- No over-reporting of the unclipped request.

## Test Case 5 — Tracker resets on turn start
1. Use a unit that has already consumed part of its round budget.
2. Advance to the next turn using the real turn-start path.
3. Query the tracker state and apply another `GrantEnergy` effect.

**Expected:**
- `RoundEnergyTracker` resets at turn start alongside the round flags lifecycle.
- The next round can gain energy again up to the cap.

## Test Case 6 — Deferred Tamer and Child affordances stay non-executable
1. Query the affordance surface for Tamer Gauge, Data Scan, Emergency Guard, Retreat, and the Child Tamer Gauge boost dependency.
2. Inspect their resource details and implementation status.

**Expected:**
- Tamer Gauge / Tamer Commands / Child boost are surfaced as deferred or hidden affordances.
- They use stable machine-readable reason codes such as `TamerGaugeDeferred` and `TamerCommandDeferred`.
- They are not represented as executable player actions.

## Test Case 7 — Canonical hidden Form Identity still works through the runtime path
1. Trigger the canonical hidden Form Identity energy follow-up in the live pipeline.
2. Repeat within the same round to confirm the cap still applies.
3. Advance the turn and trigger again.

**Expected:**
- The hidden skill remains absent from player-facing affordances.
- The internal follow-up can still execute for the acting unit.
- The same-round cap is enforced, and the next-turn reset restores budget.

## Edge Cases
- Missing or incomplete snapshot budget fields should default conservatively rather than panic.
- Zero remaining budget must return `Disabled`, not `Enabled` with a misleading reason.
- Deferred resource declarations must remain queryable even when the runtime implementation is intentionally deferred.

## Acceptance
S05 is accepted when the query surface can explain Energy caps, the live pipeline enforces them, hidden/internal Form Identity behavior remains cap-aware, and deferred Tamer/Child resource truth is exposed without user-facing hardcoding.
