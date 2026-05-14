---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Aggiungere CombatEventKind::UltimateUsed + emit nei 4 hoist e test ultimate_event

Aggiunge il nuovo variant `UltimateUsed { unit_id: UnitId }` a `CombatEventKind` (vicino a `UltGain`) ed emette l'evento una sola volta per cast nei quattro blocchi di resource-hoist di `pipeline.rs` (single-target ~561, Blast/AllEnemies ~1077, AllAllies ~1357, PerHop ~1861), gated da `matches!(inflight.action.ult_effect, UltEffect::Reset)`. Source e target = attacker_id (mirroring UltGain). Test integrazione headless in stile `tests/ultimate_meter.rs` che drive `ActionIntent::Ultimate` con `ult.current == max` ed asserisce: (a) esattamente un evento `UltimateUsed` con `unit_id == attacker`; (b) nessun `UltimateUsed` su Basic o Skill non-Reset.

## Inputs

- `src/combat/events.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/types.rs`
- `.gsd/milestones/M020/slices/S01/S01-RESEARCH.md`
- `tests/ultimate_meter.rs`

## Expected Output

- `src/combat/events.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/ultimate_event.rs`

## Verification

cargo test --test ultimate_event && cargo check && cargo check --features windowed

## Observability Impact

Nuovo segnale univoco sul bus per ogni cast di ultimate; abilita listener downstream a distinguere `UltGain` (carica) da `UltimateUsed` (spesa) senza ricostruire stato.
