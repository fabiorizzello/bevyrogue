---
estimated_steps: 1
estimated_files: 10
skills_used: []
---

# T02: Rinominare OnKO → UnitDied { status_remaining, heated_remaining } con payload

Rinomina il variant `OnKO` in `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` in `src/combat/events.rs`. Riempi il payload dentro `apply_damage_only` (`src/combat/resolution.rs` ~559-561 e ~780) usando il parametro `defender_status: Option<&StatusBag>` già in scope: `status_remaining` = `bag.iter().map(|inst| inst.kind.clone()).collect()` (vec vuoto se `None`); `heated_remaining` = `bag.get_dur(&StatusEffectKind::Heated).unwrap_or(0)`. Aggiorna i match arm in `pipeline.rs` (4 siti: ~458, ~975, ~1357, ~1690) usando pattern `UnitDied { .. }` per scartare il payload (comportamento invariato). In `src/combat/turn_system/mod.rs:488` il payload non ha bag in scope: emetti con `status_remaining: vec![], heated_remaining: 0` e commento di una riga. Aggiorna i self-test in `resolution.rs` (1298, 1338) e i test in `tests/combat_coherence.rs:451`, `tests/follow_up_triggers.rs:193` (stringhe JSON attese -> `{"kind":"UnitDied","status_remaining":[],"heated_remaining":0}`), `tests/event_stream.rs:251,268`, `tests/pipeline_dispatch.rs:253,268`, `tests/toughness_enemy_only.rs:208`. Aggiungi `tests/unit_died_payload.rs`: setup defender con `Heated`(dur 2) + `Slowed`(dur 1) nel `StatusBag`, infliggi danno fatale via `apply_effects`, asserisci che il `UnitDied` finale porti `status_remaining` contenente entrambi i kind e `heated_remaining == 2`.

## Inputs

- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/status_effect.rs`
- `tests/combat_coherence.rs`
- `tests/follow_up_triggers.rs`
- `tests/event_stream.rs`
- `tests/pipeline_dispatch.rs`
- `tests/toughness_enemy_only.rs`
- `tests/status_blessed_offensive.rs`
- `.gsd/milestones/M020/slices/S01/S01-RESEARCH.md`

## Expected Output

- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/combat_coherence.rs`
- `tests/follow_up_triggers.rs`
- `tests/event_stream.rs`
- `tests/pipeline_dispatch.rs`
- `tests/toughness_enemy_only.rs`
- `tests/unit_died_payload.rs`

## Verification

cargo test && cargo check --features windowed && ! rg -n 'CombatEventKind::OnKO' src tests

## Observability Impact

Il bus ora trasporta lo snapshot status del defender al momento del KO; la wire JSON in `src/combat/jsonl_logger.rs` cambia il discriminatore da `OnKO` a `UnitDied` con payload (rilevante per analisi log).
