# PLAN — Out-of-Turn Ultimate (HSR-style Burst)

**Workflow:** small-feature `260522-1-small-feature`
**Scope:** see `CONTEXT.md`. Decisions D1–D4 are assumed.
**Isolation note:** implement-phase commits wait for M003's `auto.lock` to free.

## Approach (the one tricky bit)

The burst system validates the request through the **existing legality gate**
(D1), then makes the Ult pass the `is_active` check for exactly one resolution
cycle via a transient resource `OutOfTurnBurst(Option<UnitId>)`:

- `build_snapshot_from_ecs` (`action_query/types.rs:135-139`) marks
  `is_active = (id == active_unit) || OutOfTurnBurst == Some(id)`.
- The burst system sets `OutOfTurnBurst = Some(attacker)`, writes
  `ActionIntent::Ultimate`, and the existing pipeline resolves it normally.
- `OutOfTurnBurst` is cleared after the cycle. `TurnOrder.active_unit` and the
  burst unit's `ActionValue` are **never** touched (D2). Phase guard ensures the
  request is only honored in `WaitingAction` (D3).

This keeps SP cost, gauge-ready, KO, stun, and targeting checks intact — the
burst path differs from a normal Ult only in the `is_active` derivation.

---

## Tasks

### T1 — Burst request message + AV-preserving legality seam
**Files:** `src/combat/turn_system/types.rs` (new `UltBurstRequest` message),
`src/combat/action_query/types.rs` (add `OutOfTurnBurst` resource; widen
`is_active` derivation in `build_snapshot_from_ecs`).
**Change:** Define `#[derive(Message)] struct UltBurstRequest { attacker: UnitId, target: UnitId }`.
Add `#[derive(Resource, Default)] struct OutOfTurnBurst(Option<UnitId>)`. In
snapshot build, treat the burst unit as active for legality only.
**Verify:** `cargo check`; new headless test asserts
`query_action_affordance(.., Ultimate)` returns `Enabled` for a non-active unit
when `OutOfTurnBurst == Some(it)` and gauge ready, and `Disabled{UltimateNotReady}`
when gauge not ready — proving the gauge/SP checks still bite.
**Commit:** `feat(combat): out-of-turn ult legality seam + burst request message`

### T2 — Burst resolution system (AV-free, phase-gated)
**Files:** new `src/combat/turn_system/burst.rs`; register in
`src/combat/turn_system/mod.rs` / combat plugin, ordered **before**
`resolve_action_system`.
**Change:** System drains `UltBurstRequest`. For each: guard
`phase == WaitingAction`; run the legality check (D1) with the burst kind; if
legal, set `OutOfTurnBurst = Some(attacker)` and write `ActionIntent::Ultimate
{ attacker, target }`. Add a follow-on tiny system (or end-of-cycle clear) that
resets `OutOfTurnBurst` to `None` after resolution. Assert no write to
`ActionValue` or `TurnOrder.active_unit`.
**Verify:** `cargo check`; headless test: ready non-active unit + `UltBurstRequest`
→ `UltimateUsed` event emitted, gauge drained, **and** burst unit's `ActionValue`
unchanged + `active_unit` unchanged.
**Commit:** `feat(combat): resolve out-of-turn ult bursts without consuming AV`

### T3 — combat_cli surfacing
**Files:** `src/bin/combat_cli/player.rs`.
**Change:** When building the action menu, if a *non-active* party unit has a
ready ult gauge and phase is `WaitingAction`, offer a "Burst: <unit> Ultimate"
entry that writes `UltBurstRequest` instead of `ActionIntent`. Non-interactive
proof mode: deterministic (off unless a scenario flag opts in) to preserve R004.
**Verify:** `cargo check`; `cargo run` (headless CLI) smoke in a scenario with a
charged off-turn unit shows the burst option and fires it. Manual windowed check
NOT required (no render changes).
**Commit:** `feat(combat-cli): offer out-of-turn ult burst when gauge ready`

### T4 — Invariant tests (rejection + AV preservation)
**Files:** new cases under `tests/turn_economy/` (existing scope; see R003) —
e.g. `tests/turn_economy/ult_out_of_turn.rs` included via the scope harness.
**Change:** rstest table covering: (a) burst fires off-turn when ready; (b)
rejected when gauge not ready; (c) rejected in `Resolving`/`WaitingForTurn`
phase; (d) rejected when burst unit KO/stunned; (e) AV of burst unit and turn
order are byte-identical before/after; (f) SP is spent. Seeded `bevy_rand`.
**Verify:** `cargo nextest run` (agent profile) green for the new scope cases.
**Commit:** `test(turn-economy): out-of-turn ult burst invariants`

---

## Verify phase (after T1–T4)
- `cargo check` (headless default) and `cargo check --features windowed`
- `cargo nextest run` full suite (agent profile)
- `cargo clippy --all-targets`
- Write `SUMMARY.md`.

## Task ordering / dependencies
T1 → T2 → T3, T4 (T3 and T4 both depend on T2; independent of each other).
Each task is independently committable.
