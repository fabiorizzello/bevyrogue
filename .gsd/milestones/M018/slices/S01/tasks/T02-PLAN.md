---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T02: DSL + event bus + resolver: split AdvanceTurn/DelayTurn(u32), rimuovi TurnAdvance(i32)

Aggiungere `Effect::AdvanceTurn(u32)` e `Effect::DelayTurn(u32)` in `src/data/skills_ron.rs`. Aggiungere `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` e `DelayTurn { target, amount_pct: u32 }` in `src/combat/events.rs`. Riscrivere `apply_turn_advance_system` (in `src/combat/turn_system/mod.rs`) come `apply_av_ops_system` con match sui due nuovi event kinds — usa le funzioni di T01. Nel resolver (`src/combat/resolution.rs`) sostituire `skill_turn_advance` extractor con due extractor (`skill_advance` / `skill_delay`) che applicano `pct.min(50)` PRIMA di costruire l'evento (cap al sito di emissione, no pre-cap accumulator). Aggiornare `ResolvedAction` in `src/combat/state.rs`: split `turn_advance_pct: i32` in `advance_pct: u32` + `delay_pct: u32`. **Rimuovere `Effect::TurnAdvance(i32)` e `CombatEventKind::TurnAdvance` completamente** (no shim residuo). Sweep meccanico dei test che istanziano `ResolvedAction` (~15 file in tests/) — defaultano 0/0 con i nuovi campi.

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/state.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/log.rs`
- `src/combat/observability.rs`
- `src/combat/jsonl_logger.rs`
- `src/combat/resistance.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/state.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/log.rs`
- `src/combat/observability.rs`
- `src/combat/jsonl_logger.rs`

## Verification

cargo check && rg -n 'TurnAdvance' src/ assets/ tests/ deve mostrare zero occorrenze del vecchio signed `Effect::TurnAdvance` o `CombatEventKind::TurnAdvance`; solo `AdvanceTurn` / `DelayTurn`.
