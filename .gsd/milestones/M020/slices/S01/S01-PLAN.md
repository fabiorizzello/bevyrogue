# S01: Nuovi eventi reactive bus: UltimateUsed + UnitDied payload

**Goal:** Aggiungere al bus reattivo (`CombatEventKind`) il variant `UltimateUsed { unit_id }` emesso una sola volta per ogni cast di ultimate, e rinominare `OnKO` in `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` con payload riempito al momento del KO, mantenendo invariati semantica e listener esistenti.
**Demo:** cargo test passa con un nuovo test che verifica UltimateUsed emesso e un test che verifica UnitDied porta i campi corretti

## Must-Haves

- cargo test green (≥74 test, includendo i due nuovi); `tests/ultimate_event.rs` verifica esattamente un `UltimateUsed { unit_id }` per cast con id = attaccante; `tests/unit_died_payload.rs` verifica che `UnitDied` porti `status_remaining` con i kind attivi e `heated_remaining` con la durata Heated residua; `cargo check` headless e `cargo check --features windowed` senza warning nuovi; nessuna occorrenza residua di `CombatEventKind::OnKO` nel codebase.

## Proof Level

- This slice proves: contract + integration: il bus è il single-source-of-truth e i nuovi/rinominati variant vengono provati con test integrazione headless.

## Integration Closure

Wiring del nuovo emit nei 4 blocchi di resource-hoist di `pipeline.rs` (single-target, Blast/AllEnemies, AllAllies, PerHop) e nei 6 emit/match sites di `OnKO`. Nessun nuovo entrypoint runtime; integra dentro il loop di azione esistente.

## Verification

- Aumenta il segnale del bus: `UltimateUsed` rende osservabile in modo univoco l'evento "ultimate spent"; `UnitDied` porta lo stato status/heated rimanente, utile per listener post-morte (revenge effects, dot bookkeeping) e per il flusso JSONL (`src/combat/jsonl_logger.rs`).

## Tasks

- [x] **T01: Aggiungere CombatEventKind::UltimateUsed + emit nei 4 hoist e test ultimate_event** `est:1h`
  Aggiunge il nuovo variant `UltimateUsed { unit_id: UnitId }` a `CombatEventKind` (vicino a `UltGain`) ed emette l'evento una sola volta per cast nei quattro blocchi di resource-hoist di `pipeline.rs` (single-target ~561, Blast/AllEnemies ~1077, AllAllies ~1357, PerHop ~1861), gated da `matches!(inflight.action.ult_effect, UltEffect::Reset)`. Source e target = attacker_id (mirroring UltGain). Test integrazione headless in stile `tests/ultimate_meter.rs` che drive `ActionIntent::Ultimate` con `ult.current == max` ed asserisce: (a) esattamente un evento `UltimateUsed` con `unit_id == attacker`; (b) nessun `UltimateUsed` su Basic o Skill non-Reset.
  - Files: `src/combat/events.rs`, `src/combat/turn_system/pipeline.rs`, `tests/ultimate_event.rs`
  - Verify: cargo test --test ultimate_event && cargo check && cargo check --features windowed

- [x] **T02: Rinominare OnKO → UnitDied { status_remaining, heated_remaining } con payload** `est:1h30m`
  Rinomina il variant `OnKO` in `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` in `src/combat/events.rs`. Riempi il payload dentro `apply_damage_only` (`src/combat/resolution.rs` ~559-561 e ~780) usando il parametro `defender_status: Option<&StatusBag>` già in scope: `status_remaining` = `bag.iter().map(|inst| inst.kind.clone()).collect()` (vec vuoto se `None`); `heated_remaining` = `bag.get_dur(&StatusEffectKind::Heated).unwrap_or(0)`. Aggiorna i match arm in `pipeline.rs` (4 siti: ~458, ~975, ~1357, ~1690) usando pattern `UnitDied { .. }` per scartare il payload (comportamento invariato). In `src/combat/turn_system/mod.rs:488` il payload non ha bag in scope: emetti con `status_remaining: vec![], heated_remaining: 0` e commento di una riga. Aggiorna i self-test in `resolution.rs` (1298, 1338) e i test in `tests/combat_coherence.rs:451`, `tests/follow_up_triggers.rs:193` (stringhe JSON attese -> `{"kind":"UnitDied","status_remaining":[],"heated_remaining":0}`), `tests/event_stream.rs:251,268`, `tests/pipeline_dispatch.rs:253,268`, `tests/toughness_enemy_only.rs:208`. Aggiungi `tests/unit_died_payload.rs`: setup defender con `Heated`(dur 2) + `Slowed`(dur 1) nel `StatusBag`, infliggi danno fatale via `apply_effects`, asserisci che il `UnitDied` finale porti `status_remaining` contenente entrambi i kind e `heated_remaining == 2`.
  - Files: `src/combat/events.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `tests/combat_coherence.rs`, `tests/follow_up_triggers.rs`, `tests/event_stream.rs`, `tests/pipeline_dispatch.rs`, `tests/toughness_enemy_only.rs`, `tests/unit_died_payload.rs`
  - Verify: cargo test && cargo check --features windowed && ! rg -n 'CombatEventKind::OnKO' src tests

## Files Likely Touched

- src/combat/events.rs
- src/combat/turn_system/pipeline.rs
- tests/ultimate_event.rs
- src/combat/resolution.rs
- src/combat/turn_system/mod.rs
- tests/combat_coherence.rs
- tests/follow_up_triggers.rs
- tests/event_stream.rs
- tests/pipeline_dispatch.rs
- tests/toughness_enemy_only.rs
- tests/unit_died_payload.rs
