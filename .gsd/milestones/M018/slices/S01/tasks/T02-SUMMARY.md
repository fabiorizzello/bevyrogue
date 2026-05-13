---
id: T02
parent: S01
milestone: M018
key_files:
  - src/data/skills_ron.rs
  - src/combat/events.rs
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/log.rs
  - src/combat/observability.rs
  - src/combat/turn_system/pipeline.rs
  - src/ui/combat_panel.rs
  - assets/data/skills.ron
  - src/headless.rs
  - examples/dump_schedule.rs
key_decisions:
  - Cap at emission site: skill_advance/skill_delay apply pct.min(50) in the extractor before building the event — no accumulator-side cap needed
  - Slowed first-apply emission changed from TurnAdvance{amount_pct:-30} to DelayTurn{amount_pct:30} — sign encoded in variant name, not value
  - apply_turn_advance_system renamed to apply_av_ops_system; no backward-compat shim (task required no residual callers)
  - SelfAdvance effect maps to AdvanceTurn on source (attacker), capped to u32.min(50)
duration: 
verification_result: passed
completed_at: 2026-05-13T15:42:04.029Z
blocker_discovered: false
---

# T02: Removed Effect::TurnAdvance(i32) and CombatEventKind::TurnAdvance entirely; split into AdvanceTurn(u32)/DelayTurn(u32) across DSL, event bus, resolver, log, observability, UI, and all callers

**Removed Effect::TurnAdvance(i32) and CombatEventKind::TurnAdvance entirely; split into AdvanceTurn(u32)/DelayTurn(u32) across DSL, event bus, resolver, log, observability, UI, and all callers**

## What Happened

Split the signed `TurnAdvance(i32)` primitive into two unsigned variants throughout the codebase:

**Effect DSL** (`src/data/skills_ron.rs`): Replaced `Effect::TurnAdvance(i32)` with `Effect::AdvanceTurn(u32)` and `Effect::DelayTurn(u32)`. Updated test roundtrips accordingly.

**Event bus** (`src/combat/events.rs`): Replaced `CombatEventKind::TurnAdvance { target, amount_pct: i32 }` with `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` and `CombatEventKind::DelayTurn { target, amount_pct: u32 }`.

**ResolvedAction** (`src/combat/state.rs`): Split `turn_advance_pct: i32` into `advance_pct: u32` + `delay_pct: u32`.

**Resolver** (`src/combat/resolution.rs`): Replaced `skill_turn_advance` extractor with `skill_advance` and `skill_delay` — both apply `pct.min(50)` at extraction time (cap at emission site). `apply_effects` emits `AdvanceTurn` for advance_pct, `DelayTurn` for delay_pct, and `AdvanceTurn` for SelfAdvance (clamped to u32). No pre-cap accumulator.

**System** (`src/combat/turn_system/mod.rs`): Renamed `apply_turn_advance_system` → `apply_av_ops_system`; rewrote to match on `AdvanceTurn`/`DelayTurn` and dispatch to `resistance::apply_advance`/`apply_delay` respectively. Updated `headless.rs` and `examples/dump_schedule.rs` to import the new name.

**Log** (`src/combat/log.rs`): Split `LogEntry::TurnAdvance` → `LogEntry::AdvanceTurn { amount_pct: u32 }` + `LogEntry::DelayTurn { amount_pct: u32 }`.

**Observability** (`src/combat/observability.rs`): Split `ValidationLogEntry::TurnAdvance` analogously; updated mapping and format functions (delay renders as `delay(target=X,amount=Y)`).

**Pipeline** (`src/combat/turn_system/pipeline.rs`): Updated both match blocks to `AdvanceTurn`/`DelayTurn`; changed Slowed first-apply emission from `TurnAdvance{amount_pct:-30}` to `DelayTurn{amount_pct:30}`.

**UI** (`src/ui/combat_panel.rs`): Updated match on LogEntry to handle `AdvanceTurn` and `DelayTurn`.

**assets/data/skills.ron**: Updated `TurnAdvance(20)` → `AdvanceTurn(20)` (one occurrence, in a deferred Row-AoE skill — positive = advance).

**Test sweep**: Updated 7 integration test files with `ResolvedAction` literals (`advance_pct:0, delay_pct:0`); updated `follow_up_triggers.rs`, `combat_coherence.rs`, `form_identity.rs`, `tempo_resistance.rs`, `status_slowed_delay.rs`, and `status_slowed_delay.rs` for the new variants.

## Verification

cargo check: zero errors. rg -n 'TurnAdvance' src/ assets/ tests/ shows only `TurnAdvanced` (the turn-order event, unrelated struct) and legacy shim doc-comments in resistance.rs — zero occurrences of `Effect::TurnAdvance` or `CombatEventKind::TurnAdvance`. cargo test: all 156+ tests across all integration test suites pass (0 failures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 2030ms |
| 2 | `cargo test 2>&1 | grep -E 'test result'` | 0 | pass — all suites ok, 0 failed | 15000ms |
| 3 | `rg -n 'Effect::TurnAdvance|CombatEventKind::TurnAdvance' src/ assets/ tests/` | 1 | pass — zero occurrences (exit 1 = no match) | 200ms |

## Deviations

None — plan executed as written.

## Known Issues

None

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/log.rs`
- `src/combat/observability.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/ui/combat_panel.rs`
- `assets/data/skills.ron`
- `src/headless.rs`
- `examples/dump_schedule.rs`
