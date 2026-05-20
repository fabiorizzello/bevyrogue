# Integration tests

Tutti i test girano headless (no `windowed`). Naming **funzionale** — un file = una capacità.

## Bootstrap & party

| File | Cosa verifica |
|------|---------------|
| `bootstrap_spawn_composition.rs` | Spawn composizione encounter da `SelectionRequest` |
| `party_config_validation.rs` | Deserializzazione `PartyConfig` + validazione end-to-end |
| `party_selection_validation.rs` | Regole selezione 4 rookies (count, duplicati, IDs validi) |
| `roster_catalog.rs` | Roster canonico (D039) caricato da RON |

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

- **Helper condivisi in `tests/common/`** (preferenza). Sottomoduli stabili:
  - `common::units::{attacker, defender, unit}` — `Unit` factory standard.
  - `common::actions::{basic_resolved, ult_resolved, default_ult, ready_ult}` —
    `ResolvedAction` / `UltimateCharge` fixtures.
  - `common::apply::{LegacyOpsHarness, ApplyOpts, run_damage, run_ult_delta}` —
    wrapper su `apply_legacy_ops` (13 args) che possiede lo stato mutabile.
  - Helper "build_app" stile-Bevy per scenari restano nel `mod.rs`.
  - Includili nei test con `mod common;` (auto-discovered, no `[[test]]` extra).
- Helper privati nel file di test SOLO se non sono riutilizzabili o se aggiungerli
  in `common` introdurrebbe coupling tra capability diverse.
- No wall-clock, no RNG senza seed.
- Asset RON caricati via fixture inline o `assets/data/*`.
- Nome file = funzionale (no prefisso milestone/slice).

## Parametrizzazione

- **`rstest`** per tabelle di casi noti (`#[case]`). Esempi: `triangle_matchup.rs`
  (16 celle attributo×attributo), `tempo_resistance.rs` (curve points),
  `scenario_ttk.rs` (Minion/MiniBoss/Boss preset matrix). Da preferire a N
  `#[test]` con corpo quasi identico.
- **`proptest`** per invarianti su tutto il dominio: AV floor/ceiling,
  monotonia `TempoResistance::multiplier`, immunità di `Blessed` al cleanse.
  Vedi `tests/properties.rs`. I controesempi shrinkati vengono persistiti in
  `tests/proptest-regressions/` come regressioni permanenti.
