# Captures

### CAP-7c065a44 (pt.2 — Visual architecture)
**Text:** 1. Architettura Visiva (M003)
   * Niente Fisica/Collider per i VFX: In un RPG a turni deterministico, usare motori fisici (Rapier/Avian)
     per gestire impatti o VFX è un anti-pattern che rompe il determinismo e introduce overhead inutile.
   * VFX tramite Cue e Reactive Bus: Usa il sistema di CueExt (già previsto nella tua CompiledTimeline) per
     inviare segnali visivi.
   * Action Queue Feedback: La UI di M003 dovrebbe mostrare graficamente gli Intent che "esplodono" in sync
     con l'animazione.
**Captured:** 2026-05-17T13:26:46.110Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the M003 visual architecture guidance to its future milestone.
**Rationale:** The VFX/Cue/UI feedback ideas target M003 rendering work.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M001

### CAP-159d33b5
**Text:** 3. La "Sfida" di M003: Il Sync
  Il punto più critico del tuo piano è la Slice 2 di M003: Basic attack windup → strike → recovery.
   * In un sistema turn-based, il danno non deve apparire quando premi il tasto, ma quando il pugno "tocca" il
     nemico nell'animazione.
   * Nota positiva: Grazie al lavoro fatto in M021 col Two-clock model, hai già la base tecnica per mettere in
     "pausa" il kernel finché lo sprite non raggiunge il frame di impatto.
**Captured:** 2026-05-17T13:27:42.535Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer this as an M003 animation-sync planning note tied to the future visual stack and combat presentation slices.
**Rationale:** It is a milestone-ahead design constraint about animation-impact timing that depends on later M003 presentation work, not on the current S13 proof slice.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M001

### CAP-b892da3a
**Text:** 4. La Fluent DSL in Rust (Query Builder Custom)
  Proposta Architetturale: Non usare librerie generiche esterne per creare builder di query (peggiorano i
  tempi di compilazione e sono verbosissime). Crea invece un wrapper iteratore "zero-cost" specifico per il
  tuo dominio direttamente in SkillCtx. Questo permette una sintassi componibile
  (.enemies().alive().with_status::<T>()) senza perdere le prestazioni native di Rust.

  Prompt Operativo per l'implementazione:
  > "Estendi SkillCtx creando un Fluent Builder interno per semplificare le query negli hook, eliminando il
  boilerplate dell'ECS per il Game Design.
  > 1. Crea una struct UnitQuery<'a> { world: &'a World, ids: Vec<UnitId> }.
  > 2. Implementa metodi componibili e semantici come .enemies(caster_team), .alive(), e .with_status::<T:
  Component>() che usano Vec::retain per filtrare la lista ids in-place.
  > 3. Aggiungi un punto d'ingresso ctx.query_units() che restituisca questo builder inizializzato con tutti
  gli UnitId.
  > Assicurati che i metodi usino i Generics di Rust (<T: Component>) per la type-safety e non reflection o
  stringhe a runtime."
**Captured:** 2026-05-17T13:28:28.741Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the fluent SkillCtx query-builder idea to a future ergonomics/API milestone after M021 validation remediation finishes.
**Rationale:** This is an architectural API enhancement with broader design impact on SkillCtx and hook ergonomics; it does not belong in the current S13 verification slice and should be evaluated in a dedicated future milestone or slice.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M001

### CAP-e660728b
**Text:** #3 — Impl-coupling debt (typed test-API)
  ┃
  ┃  Problema: ogni rename/spostamento dentro src/combat/ rompe 100+ file di test, anche quando l'invariante testato non è cambiato. I test attualmente conoscono dove vivon
  ┃  i tipi, non cosa osservano.
  ┃
  ┃  Slice ipotizzato (non aperto):
  ┃  - nuovo modulo pub mod test_api (o crate bevyrogue-test-api separato) che ri-esporta in forma stabile solo ciò che i test devono toccare: TestAppBuilder (già esiste in
  ┃  tests/common/app.rs), proiezioni di CombatState, costruttori di UnitId/Team, observer di eventi, snapshot deterministici.
  ┃  - regola: tests/*.rs può importare solo da bevyrogue::test_api::* + tests/common/*. Niente più use bevyrogue::combat::mechanics::sp::....
  ┃  - migrazione progressiva: il primo wave introduce l'API e converte ~10 file pilota; i successivi sono meccanici (sed-grade).
  ┃
  ┃  Perché è "architetturale" e non cleanup: definire la superficie minima richiede una decisione design (DECISIONS.md) — quali invarianti sono contratto pubblico per i
  ┃  test, quali restano interni e si testano via integration. È esattamente la lezione di D026/P005 (cfr. last_transition) generalizzata.
  ┃
  ┃  Beneficio: refactor interni di src/combat/ smettono di toccare tests/. Costo: ~150 file da migrare a regime, più la disciplina di mantenere test_api minimale (non
  ┃  diventi un "ri-export tutto").
**Captured:** 2026-05-20T13:58:15.476Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the typed test-API proposal to a future architectural/testing-debt slice; likely affected areas include `tests/`, `tests/common/app.rs`, a future `src/test_api` or test-support crate, and `DECISIONS.md`.
**Rationale:** This is a broad architectural testing-boundary change with large migration impact, not a quick task and not part of the current M002/S03 gate-evaluation work.
**Resolved:** 2026-05-20T20:31:31Z
**Milestone:** M002

### CAP-3e6c589c
**Text:** #4 — R003 inversa (tests/ → src/) per unit-test puri
  ┃
  ┃  Problema: R003 vieta src/.../tests/ e blocchi inline >100 LOC, e questo è giusto. Ma alcuni test in tests/ oggi sono veri unit test su funzioni pure (es. fold di
  ┃  modifier, math di damage, parser) che starebbero meglio inline accanto al codice — coesione locale, no boundary crossing, no Bevy App setup.
  ┃
  ┃  Slice ipotizzato:
  ┃  - definire un criterio formale per "back-relocate eleggibile": (a) testa una funzione pub(crate) o pub pura, (b) non istanzia App/World/Schedule, (c) il blocco resta
  ┃  <30 LOC, (d) non duplica copertura integration esistente.
  ┃  - relax mirato di R003: consentire #[cfg(test)] mod tests inline fino a 30 LOC (Tier-C residue diventa legittimo), con check in scripts/check_loc_cap.sh aggiornato.
  ┃  - prima passata: i 6 file Tier-C residui (headless.rs, mechanics/buffs.rs, mechanics/stun.rs, encounter/bootstrap.rs, bin/combat_cli.rs, observability/log.rs, ~159 LOC
  ┃  totali).
  ┃
  ┃  Perché architetturale: cambia R003. Va in DECISIONS.md come emendamento esplicito, non come deroga silenziosa.
**Captured:** 2026-05-20T13:58:26.908Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the R003 inline-unit-test amendment to a future testing-policy/refactor slice; likely affected areas include `tests/`, selected `src/...` modules, `scripts/check_loc_cap.sh`, and `DECISIONS.md`.
**Rationale:** This changes an architectural testing rule and requires deliberate policy/design work, so it should not be injected into the current M002/S03 gate-evaluation unit.
**Resolved:** 2026-05-20T20:31:31Z
**Milestone:** M002

### CAP-3a1dfcbc
**Text:** Ult out-of-turn (HSR-style burst) — DA FARE, magari dopo M003.

Modello desiderato: l'ult è lanciabile in qualsiasi momento, nel gap tra il turno di un PG e quello del successivo — non consuma il turno (burst fuori-turno).

Stato attuale (verificato 2026-05-22): NON supportato. Le ult passano per lo stesso path di basic/skill e richiedono che sia il turno dell'unità.
- Gate bloccante: `action_query/legality/action.rs:46-54` → ogni azione (Basic/Skill/Ultimate) rifiutata con `NotActiveUnit` se non è l'`active_unit`. Nessuna eccezione per l'ult.
- `is_active` = `id == turn_order.active_unit` (`action_query/types.rs:135-139`); la validazione risorse dell'ult gira dopo, non bypassa.
- Vincolo fase: `action.rs:57-66` richiede `CombatPhase::WaitingAction`, esiste solo con unità attiva.
- Scheduler: `turn_system/advance.rs:326` è l'unico punto che imposta `active_unit` (sistema AV). Nessuna coda/interrupt/burst.

3 opzioni di design quando lo affronteremo:
1. Path parallelo `InstantAction`/burst che salta il gate di turno e si risolve fuori dall'AV scheduler (il più pulito, modella davvero l'HSR).
2. Esenzione dell'Ultimate dal check `is_active` in `action.rs:46` + finestra di iniezione tra i turni.
3. Coda ult drenata dallo scheduler prima di passare il turno al prossimo attore.

Cross-cutting (scheduler + legality + input + pipeline) → milestone/slice GSD dedicato, non un fix da fine sessione. Eco in memory store MEM051.
**Captured:** 2026-05-22T09:36:00Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer out-of-turn ultimate (HSR-style burst) to a dedicated future milestone/slice after M003; cross-cutting work spanning the AV scheduler (`turn_system/advance.rs`), legality gate (`action_query/legality/action.rs`), input, and the resolution pipeline, with three design options to weigh when opened.
**Rationale:** The user explicitly tagged it "DA FARE, magari dopo M003" and flagged it as cross-cutting requiring a dedicated GSD milestone/slice, not an end-of-session fix; it has no bearing on the current M003 rendering scope.
**Resolved:** 2026-05-22T09:38:00Z
**Milestone:** M003
