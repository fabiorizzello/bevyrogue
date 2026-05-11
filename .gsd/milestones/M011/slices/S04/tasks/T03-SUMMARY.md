---
id: T03
parent: S04
milestone: M011
key_files:
  - src/bin/combat_cli.rs
key_decisions:
  - Reordered combat_dashboard_system before event_logger_system in the chain because MessageReader is consuming — whichever system runs first drains the queue.
  - Fired the initial TurnAdvanced from bootstrap_system after apply_composition so the advance_turn_system loop can start; mirrors the pattern in headless.rs.
duration: 
verification_result: passed
completed_at: 2026-04-27T18:48:15.370Z
blocker_discovered: false
---

# T03: Fixed combat_dashboard_system compile errors and message-ordering bug so the formatted HP/SP/Toughness/UltCharge dashboard renders on every turn advance.

**Fixed combat_dashboard_system compile errors and message-ordering bug so the formatted HP/SP/Toughness/UltCharge dashboard renders on every turn advance.**

## What Happened

The dashboard system was already scaffolded from a prior session but had three compile errors and a runtime ordering bug preventing it from ever rendering.

Compile fixes:
1. `println!("\n" + "=".repeat(60))` → `println!("\n{}", "=".repeat(60))` (string concat syntax)
2. `sp_pool.0` → `sp_pool.current` / `sp_pool.max` (SpPool has named fields, not a tuple struct)
3. `CombatEventKind::TurnOrderSeeded` in `matches!` needed `{ .. }` since it's a struct variant

Runtime fix — the dashboard never rendered because `event_logger_system` ran before `combat_dashboard_system` in the `.chain()` and consumed all `CombatEvent` messages (MessageReader is consuming). Reordered so `combat_dashboard_system` runs first.

Second runtime fix — even after reordering, the dashboard only triggers on `TurnAdvanced` or specific `CombatEventKind` events, but nobody fired the first `TurnAdvanced`. Added `MessageWriter<TurnAdvanced>` to `bootstrap_system`: after `apply_composition` seeds the queue, `order.advance()` returns the first unit ID and we emit `TurnAdvanced(first)`. Also added the missing `TurnOrderSeeded` combat event to match the headless.rs pattern.

Also bumped the timeout from 20 ticks (2s) to 60 ticks (6s) to give the engine time to run past data loading before exiting.

## Verification

`cargo check --bin combat_cli` — clean. `echo "" | cargo run --bin combat_cli 2>/dev/null` — dashboard prints with SP, turn order, and per-unit HP/ULT/TGH rows. `cargo test` — all 20+ integration tests pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --bin combat_cli` | 0 | ✅ pass | 340ms |
| 2 | `echo "" | cargo run --bin combat_cli 2>/dev/null | head -80` | 0 | ✅ pass — dashboard with SP/TurnOrder/HP/ULT/TGH rendered | 5200ms |
| 3 | `cargo test` | 0 | ✅ pass — all tests green | 45000ms |

## Deviations

Bumped tick timeout from 20 to 60 (2s → 6s) to ensure data loading completes before exit in the non-interactive test path.

## Known Issues

None.

## Files Created/Modified

- `src/bin/combat_cli.rs`
