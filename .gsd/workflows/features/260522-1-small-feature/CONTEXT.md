# CONTEXT — Out-of-Turn Ultimate (HSR-style Burst)

**Workflow:** small-feature `260522-1-small-feature`
**Capture:** CAP-3a1dfcbc — "Ult out-of-turn (HSR-style burst)"
**Date:** 2026-05-22
**Branch:** master (isolation: none — see Constraints)

## Feature description

Today an Ultimate can only be fired by the unit that currently holds the turn
(`TurnOrder.active_unit`). This feature lets a ready Ultimate be fired
**out of turn** — Honkai: Star Rail "burst" model: the moment a unit's gauge is
ready, the player may interject the Ult between turns without it being that
unit's scheduled turn, and **without consuming/advancing that unit's AV**.

User-facing behavior:
- A unit whose ult gauge is ready can fire its Ultimate even when another unit
  is the active unit.
- Firing the burst does **not** cost the burst unit its normal turn (no AV
  reset, no turn skip).
- All other Ult rules still hold: SP cost, gauge consumption on use, targeting.

## Current mechanics (researched — file:line grounded)

- **Turn scheduler:** AV-based. `advance_turn_system`
  (`src/combat/turn_system/advance.rs`) runs only in phase `WaitingForTurn`;
  the highest-AV ready unit becomes `TurnOrder.active_unit`, phase → `WaitingAction`.
- **`is_active`** is computed in `build_snapshot_from_ecs`
  (`src/combat/action_query/types.rs:135-139`) as `id == turn_order.active_unit`.
- **The single hard block:** `action_and_resource_status_for_snapshot`
  (`src/combat/action_query/legality/action.rs:46-55`) returns
  `Disabled{NotActiveUnit}` for any actor with `!is_active`. This fires for
  Ultimate too. **This is the seam the feature must open.**
- **Intent already exists:** `ActionIntent::Ultimate { attacker, target }`
  (`src/combat/turn_system/types.rs:26-29`). The resolution pipeline
  (`resolve_action_system` → `step_declaration` → `step_app` →
  `paths/single_target::run`) already resolves it and handles
  `UltEffect::Reset` (gauge drain + `UltimateUsed` event).
- **No interrupt/insert lane exists today.** Intents are read once per cycle by
  `resolve_action_system`; follow-ups run via a separate reactive system.
- **Input:** player intents written by `player_action_system`
  (`src/bin/combat_cli/player.rs`); headless tests write `ActionIntent`
  directly via `MessageWriter<ActionIntent>`.

## Key decisions (to confirm at scope gate)

| # | Decision | Recommendation | Rationale |
|---|----------|----------------|-----------|
| D1 | How to bypass the `NotActiveUnit` block for a burst | Add a typed **burst path** that resolves an out-of-turn Ult intent through a dedicated legality kind (e.g. `ActionQueryKind::Ultimate` + a `burst: bool` context), keeping legality as the single source of truth — **not** bypassing the legality layer | Bypassing legality scatters the rules and breaks the snapshot/affordance consumers (tests rely on `query_action_affordance`) |
| D2 | Does the burst consume the unit's turn / change AV? | **No** — burst is a free interjection; the burst unit's AV is untouched and its scheduled turn still comes | This is the defining property of the HSR model; without it, it's just "act early" |
| D3 | When may a burst fire (phase)? | Only in `WaitingAction` between turns (not mid-`Resolving`) | Mirrors HSR (ult between actions, not mid-animation); avoids re-entrancy in the resolution pipeline |
| D4 | Trigger surface | New explicit signal: a `UltBurstRequest { attacker, target }` message that a new system (ordered before `resolve_action_system`) validates via legality and converts to an `ActionIntent::Ultimate` resolved out-of-turn. Headless tests write `UltBurstRequest`; combat_cli offers it when gauge is ready | Keeps the normal turn intent path untouched; a separate message is testable headless and avoids overloading `ActionIntent` semantics |

## Scope boundaries

**In scope**
- Out-of-turn Ult activation for **player-controlled** units, headless-first.
- Legality exception for the burst path (D1), AV-preserving (D2), phase-gated (D3).
- A dedicated trigger message + resolution wiring (D4).
- Headless integration tests under `tests/` proving: burst fires when not active,
  burst is rejected when gauge not ready / wrong phase / KO / stunned, and burst
  does **not** alter the burst unit's AV or the active unit's turn.
- combat_cli surfacing of the burst option when ready.

**Out of scope**
- Enemy AI initiating out-of-turn bursts.
- Multi-ult ordering / burst chaining / burst priority queues.
- Any animation/VFX/render concerns (that is M003's domain — explicitly avoided
  to prevent collision; this feature touches no `windowed/` or `animation/` files).
- Changing the AV scheduler's core math.

## Constraints / risks

- **M003 auto-mode is live on this same checkout (`isolation: none`).**
  PID 103950 holds `auto.lock` (`plan-slice M003/S01`). Implement-phase commits
  must wait for that lock to free, otherwise GSD's snapshot/auto-commit can sweep
  uncommitted burst edits into an M003 commit. Scope + plan are filesystem-only
  and safe to do now.
- **No file overlap with M003** (M003 = `windowed/render.rs`, `animation/`;
  this = `turn_system/`, `action_query/legality/`, `combat_cli/player.rs`).
  The risk is the shared git index, not the source files.
- **Determinism (R004):** tests seeded, no wall-clock. **Headless-first (R002):**
  every system runs without `windowed`.
- This capture was originally classified as "cross-cutting, dedicated milestone".
  Running it as a small-feature is the user's explicit choice; the AV-preservation
  and legality-seam decisions above are what keep it bounded.
