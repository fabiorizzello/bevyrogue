---
id: S01
milestone: M017
status: ready
---

# S01: Enum rewrite + RON migration + tests cascade — Context

## Goal

Sostituire la vecchia tassonomia status (`Burn`/`Freeze`/`Shock`/`DeepFreeze`) con il vocabolario canon §H.1 (`Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed` + `Burn`/`Shock` reserved gas-era), migrare i file RON, e ripristinare la build verde — senza ancora cablare la semantica per-status (delegata a S02-S05).

## Why this Slice

È la fondazione vocabolaria di M017: ogni slice successiva (S02 policy apply/refresh, S03 Heated/Chilled, S04 Paralyzed/Slowed, S05 Blessed, S06 observability) si poggia sui nuovi nomi. Senza il rewrite, ogni `Effect::ApplyStatus` parlerebbe ancora la vecchia lingua e ogni test downstream nascerebbe già sporco. Va fatta prima di tutto e va lasciata su build verde, altrimenti S02 parte rotta.

## Scope

### In Scope

- Rewrite enum `StatusKind` (o equivalente) in `src/combat/status_effect.rs` con le 5 variant canon attive + `Burn`/`Shock` reserved (dichiarate ma non applicabili).
- Migrazione naming negli emitter/observers: `events.rs` (payload `OnStatusApplied`), `observability.rs`, `log.rs`, `jsonl_logger.rs`, `damage.rs`, `speed.rs`, `sp.rs`/`ultimate.rs`, `turn_system/pipeline.rs` (solo rename + hook stubs no-op per S02+).
- Migrazione RON: aggiornare `assets/data/skills.ron` con id canon secondo mappa di default (`Burn→Heated`, `Freeze→Chilled`, `Shock→Paralyzed`, `DeepFreeze→Slowed`).
- Validator in `src/data/skills_ron.rs`: a load-time il loader accetta i 5 id canon (`heated`/`chilled`/`paralyzed`/`slowed`/`blessed`) e **rifiuta con errore chiaro** qualsiasi altro id, **inclusi `burn`/`shock`** (reserved gas-era: dichiarati nell'enum ma non applicabili in v0 — fail-fast).
- Cascade test: rename riferimenti in `tests/follow_up_chains.rs`, `tests/combat_coherence.rs`, `tests/form_identity.rs` (testano altre cose, non semantica status).
- **Cancellazione** dei 4 file test legacy della vecchia semantica: `tests/status_effect_apply.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_effect_integration.rs`, `tests/status_accuracy.rs`. S02-S05 scrivono test fresh sulla semantica canon.
- Aggiornamento `docs/combat_current.md` sezione status (tassonomia canon).

### Out of Scope

- Semantica per-status (DoT Heated, amp% damage, Chilled −20% speed, Paralyzed skip-turn, Slowed delay-on-apply, Blessed buff dealt + Ult charge + cleanse-immune) → S02-S05.
- Policy `refresh_max_dur` + cleanse filter Debuff-only (logica vera) → S02.
- Nuove reactive event variants (`StatusApplied` tipizzato, `UltimateUsed`, `UnitDied` extended payload) → M020.
- DR pipeline + `BuffKind::DR` clamp 0.5 → M019.
- Heal/Cleanse come `Effect` variants → M019.
- AdvanceTurn/DelayTurn split + gauge clamp → M018.
- TargetShape resolver expansion → M018.
- Stack-aware Heated × N → post-M017 (D009).
- Implementazione behavior delle variant reserved `Burn`/`Shock` → fuori M017.
- Test `#[ignore]` con inventario: scartato, sostituito da delete-and-rewrite-fresh.

## Constraints

- D004 + D009: le 5 variant attive sono single-instance per target; nessun counter multi-stack.
- D004: `Confused` dropped — non deve riapparire nell'enum né nei RON.
- Determinismo (CLAUDE.md): nessun wall-clock, nessun RNG senza seed nei test.
- Headless first (D008): nessuna dipendenza winit/wgpu/egui introdotta; tutto deve compilare con `cargo check` default.
- Eventi single-source-of-truth: `CombatEvent` resta il bus; il rename del payload `OnStatusApplied` non deve introdurre canali paralleli.
- Verifica di chiusura: `grep -r 'Burn\|Freeze\|Shock\|DeepFreeze' src/ tests/` ritorna match **solo** per le variant reserved `Burn`/`Shock` nell'enum (non `Freeze`/`DeepFreeze`, non nei test).
- Verifica build: `cargo check` + `cargo test` (full headless integration suite) verdi a fine slice.

## Integration Points

### Consumes

- `assets/data/skills.ron` — sorgente skill che usa gli id status (riscritto in place secondo mappa default).
- `src/data/skills_ron.rs` — schema `Effect::ApplyStatus` e validator (loosen + tighten su id canon).
- `src/combat/events.rs` — payload `OnStatusApplied` (rename field type a nuovo `StatusKind`).
- Test esistenti non-status (`follow_up_chains.rs`, `combat_coherence.rs`, `form_identity.rs`) che incidentalmente nominano i vecchi status.

### Produces

- `src/combat/status_effect.rs` — enum `StatusKind` canon (5 attive + 2 reserved); apply/refresh/tick restano stub no-op fino a S02.
- `src/data/skills_ron.rs` — validator che fail-fast su id non canon (incluso `burn`/`shock`).
- `assets/data/skills.ron` — id status migrati a vocabolario canon.
- `docs/combat_current.md` — sezione status aggiornata al canon §H.1.
- Suite test ridotta: 4 file legacy cancellati; rinominati i riferimenti nei 3 file non-status.

## Open Questions

- Nessuna aperta al kickoff. Le scelte grey-area sono risolte:
  - Reserved `burn`/`shock` in RON → **fail-fast a load-time** (non no-op silenzioso, non warn).
  - Mappa di rename → **default** (`Burn→Heated`, `Freeze→Chilled`, `Shock→Paralyzed`, `DeepFreeze→Slowed`).
  - Test legacy semantica → **delete-and-rewrite-fresh** in S02-S05 (no `#[ignore]`, no inventario).
