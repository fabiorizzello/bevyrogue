# SUMMARY — Out-of-Turn Ultimate (HSR-style Burst)

**Workflow:** small-feature `260522-1-small-feature` · **Status:** complete
**Date:** 2026-05-22 · **Branch:** master (isolation:none, in parallel with M003)

## What was built

A unit can now fire its Ultimate **out of turn** — a free "burst" that does not
consume the active unit's turn. The burst is validated through the *existing*
legality gate; the only thing lifted is the active-unit check. Gauge-readiness,
SP cost, KO, stun, phase, and targeting all still apply.

**Queueing (added after initial T1–T4).** Pressing ult when it cannot launch
*right now* no longer drops the request — it is parked in the new
`PendingBurstQueue` and fires automatically the moment it becomes launchable.
The only non-launchable moment is **the enemy's turn** (the active unit is an
enemy, or combat is mid-`Resolving`, or the AV-ticking `WaitingForTurn` gap).
A queued burst fires on the first `WaitingAction` frame with a non-enemy active
unit — i.e. the instant control returns to the player. A burst pressed during
the player's own action window still fires immediately and never lingers.

Flow per frame:

1. `burst_action_system` enqueues new `UltBurstRequest`s into `PendingBurstQueue`.
   If the current frame is launchable (`WaitingAction`, active unit not an enemy),
   it pops the front request, validates it (active-unit check lifted via
   `mark_unit_active`), and on success sets `OutOfTurnBurst = Some(attacker)` +
   writes `ActionIntent::Ultimate`. Otherwise the queue is held untouched. At most
   one burst per frame (resolve consumes one `ActionIntent`/frame).
2. `resolve_action_system` (same frame, immediately after) honors the same lift
   for that one cycle and resolves the ult normally — gauge reset, SP spent,
   `UltimateUsed` emitted.
3. `OutOfTurnBurst` is cleared at the top of the next `burst_action_system` run,
   so it is `Some` only across the single resolving frame.

`TurnOrder.active_unit` and every unit's `ActionValue` are **never touched** — a
burst costs no turn (verified byte-identical before/after).

**Why legality keeps the phase gate.** `legality/action.rs` enforces
`phase == WaitingAction` for every action. The burst system does *not* lift that
(only the active-unit gate), so it cannot fire during the AV gap; it defers
instead. This keeps legality the single source of truth and matches the player's
experience — input only ever surfaces in `WaitingAction` windows anyway.

## Deviation from PLAN (one)

The plan said to widen the `is_active` derivation inside
`build_snapshot_from_ecs`. That function has **4 callers**, two in
`src/ui/combat_panel/` (adjacent to M003's live render work). Threading a new
parameter through all four would have maximized merge-overlap risk with M003 for
no benefit (the UI preview callers don't need burst awareness).

Instead, the shared builder signature is untouched. A small, testable helper
`mark_unit_active(&mut snapshot, unit_id)` forces `is_active` locally at exactly
the two sites that need it — the engine legality path (`resolve.rs`) and the CLI
menu (`player.rs`), bridged by the `OutOfTurnBurst` resource. Net effect is
identical; the change is confined to `turn_system/` + `combat_cli/`, with **zero
edits to `src/ui/` or `src/windowed/render.rs`**.

Also: `resolve_action_system`'s `OutOfTurnBurst` access is `Option<Res<>>` so the
many test-harness apps that schedule that system without the resource don't panic.

## Files changed

| Task | Files | Commit |
|------|-------|--------|
| T1 | `turn_system/types.rs`, `turn_system/mod.rs`, `action_query/types.rs`, `action_query/mod.rs`, `combat/plugin.rs`, `main.rs`, `bin/combat_cli.rs`, `tests/action_query.rs` + `tests/action_query/out_of_turn_burst_seam.rs` | `0a821b1` |
| T2 | `turn_system/burst.rs` (new), `turn_system/mod.rs`, `turn_system/resolve.rs`, `headless.rs`, `bin/combat_cli.rs`, `windowed/mod.rs` | `5f860ac` |
| T4 | `turn_system/resolve.rs` (Option<Res>), `tests/turn_economy.rs` + `tests/turn_economy/ult_out_of_turn.rs` | `c3855e2` |
| T3 | `bin/combat_cli/player.rs` | `afb205d` |
| Queue | `turn_system/types.rs` (`PendingBurstQueue`), `turn_system/mod.rs`, `turn_system/burst.rs`, `combat/plugin.rs`, `tests/turn_economy/ult_out_of_turn.rs` | `6ed4b3e` |

## How to use

- **Programmatically / AI / UI:** write a `UltBurstRequest { attacker, target }`
  message (registered in both entrypoints). If legal, the burst fires next frame.
- **Interactive CLI:** when a non-active ally has a ready ult during your turn,
  the menu offers `⚡ Burst: <unit> Ultimate` (and a `▶ Continue my turn` opt-out)
  before the normal action prompt. Selecting it fires the burst without ending
  your turn. The non-interactive proof path returns before this, so CI
  determinism (R004) is unaffected. (The CLI prompts only during the player's own
  `WaitingAction`, so its bursts always fire immediately; the queue matters for a
  free-input front-end like the windowed UI, where ult can be pressed any time.)

## Verification (fresh, this session)

- `cargo check` (headless default) — pass
- `cargo check --features windowed` — pass (at T2; T3 touched only the combat_cli
  bin, leaving windowed unaffected)
- `cargo nextest run --profile agent` — **673 passed, 1 skipped** (11 new burst
  tests: 4 seam + 7 invariant, incl. queue-during-enemy-turn and queue-during-AV-gap)
- `cargo clippy --all-targets` — no findings in any new code (incl. queue commit)

## Manual verification still needed

Interactive burst *selection* in the real terminal CLI cannot be driven from
auto-mode (no TTY; K001). The burst engine path is fully proven headlessly; the
CLI only emits the request. Recommend a quick manual `cargo run --bin combat_cli`
play-through with a charged off-turn ally to confirm the menu UX.
