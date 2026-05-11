# Integration tests

Tutti i test girano headless (no `windowed`). Naming **funzionale** — un file = una capacità.

## Bootstrap & party

| File | Cosa verifica |
|------|---------------|
| `bootstrap_spawn_composition.rs` | Spawn composizione encounter da `SelectionRequest` |
| `party_config_validation.rs` | Deserializzazione `PartyConfig` + validazione end-to-end |
| `party_selection_validation.rs` | Regole selezione 4 rookies (count, duplicati, IDs validi) |
| `roster_catalog.rs` | Roster canonico (D039) caricato da RON |
| `roster_smoke.rs` | Smoke run con roster completo (M006) |

## Turn / action pipeline

| File | Cosa verifica |
|------|---------------|
| `encounter_e2e.rs` | End-to-end encounter scriptato |
| `event_stream.rs` | `CombatEvent` bus emesso correttamente |
| `boundary_contract.rs` | Vincoli HP/Toughness ai bordi |
| `validation_snapshot.rs` | Snapshot di osservabilità (`capture_validation_snapshot`) |
| `commander_flow.rs` | Commander targeting / regole nemiche |

## Combat mechanics

| File | Cosa verifica |
|------|---------------|
| `enemy_ai.rs` | Routing decisioni nemiche (M008/S01) |
| `ultimate_meter.rs` | Accumulazione + flush ult charge (M008/S03) |
| `sp_economy.rs` | Cap SP, generazione, regressione 20-turn (M008/S05) |
| `combat_coherence.rs` | Coerenza interazioni cross-modulo (M008/S06) |
| `status_effect_apply.rs` | Applicazione StatusEffect via skill (M008/S04) |
| `status_effect_integration.rs` | Integrazione StatusEffect in resolve_action |
| `status_effect_turn_tick.rs` | Tick durata + scadenza StatusEffect |
| `follow_up_triggers.rs` | Trigger follow-up FIFO (M005+) |
| `follow_up_reentrancy.rs` | Reentrancy / depth limit follow-up |
| `revive_semantics.rs` | KO → revive lifecycle |
| `patamon_revive.rs` | Skill revive specifica Patamon |

## Convenzioni

- Helper privati in cima al file (`fn unit(...)`, `fn build_app()`, …).
- No wall-clock, no RNG senza seed.
- Asset RON caricati via fixture inline o `assets/data/*`.
- Quando aggiungi un test: file = nome funzionale (no prefisso milestone/slice).
