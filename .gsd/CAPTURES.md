# Captures

### CAP-8d133d1a
**Text:** vedo un sacco di file relativi a identità custom dei digimon ma sotto combat/ battery_loop, precision_mind_game(è ancora usato?)? tieni conto che non dobbiamo mantenerci retrocompatibili o altro. voglio che dal kernel spariscano le logiche/strutture dati custom dei specifici digimon. in più i test che non servono realmente si possono togliere. è normale avere tutti quei test per il nostro numeor di LOC del progetto effettivo? Fai un passaggio di prune/remove orphan code/obsoleto. magari tramite un ultimo slice a fine m021
**Captured:** 2026-05-17T06:50:03.872Z
**Status:** done
**Classification:** defer
**Resolution:** Completed across M021 waves 1-5: kernel cleaned of digimon-specific logic (battery_loop, precision_mind_game removed), orphan code pruned, dead-code warnings eliminated. Further structural reorganization done in post-M021 refactor (combat submodule split).
**Rationale:** This is important follow-up work, but it belongs in a later cleanup slice rather than changing the current S12 scope.
**Resolved:** 2026-05-17T08:18:14Z
**Completed:** 2026-05-18
**Milestone:** M021

### CAP-af4db4ca
**Text:** anche enemy_counterplay - ogni enemy avrà il suo counterplay, non bisogna inserire la logica nel kernel, se non primitive
**Captured:** 2026-05-17T06:51:21.060Z
**Status:** done
**Classification:** defer
**Resolution:** Completed: counterplay.rs consolidated as typed per-enemy data declarations (not kernel logic). enemy_counterplay.rs merged into counterplay.rs. Module moved to combat/encounter/ submodule alongside bootstrap and enemy_ai.
**Rationale:** This is a reusable architectural note, but it does not require immediate action in the current slice.
**Resolved:** 2026-05-17T08:18:14Z
**Completed:** 2026-05-18
**Milestone:** M021

### CAP-7c065a44 (pt.2 — Visual architecture)
**Text:** 1. Architettura Visiva (M023)
   * Niente Fisica/Collider per i VFX: In un RPG a turni deterministico, usare motori fisici (Rapier/Avian)
     per gestire impatti o VFX è un anti-pattern che rompe il determinismo e introduce overhead inutile.
   * VFX tramite Cue e Reactive Bus: Usa il sistema di CueExt (già previsto nella tua CompiledTimeline) per
     inviare segnali visivi.
   * Action Queue Feedback: La UI di M023 dovrebbe mostrare graficamente gli Intent che "esplodono" in sync
     con l'animazione.
**Captured:** 2026-05-17T13:26:46.110Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the M023 visual architecture guidance to its future milestone.
**Rationale:** The VFX/Cue/UI feedback ideas target M023 rendering work.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M023

### CAP-159d33b5
**Text:** 3. La "Sfida" di M023: Il Sync
  Il punto più critico del tuo piano è la Slice 2 di M023: Basic attack windup → strike → recovery.
   * In un sistema turn-based, il danno non deve apparire quando premi il tasto, ma quando il pugno "tocca" il
     nemico nell'animazione.
   * Nota positiva: Grazie al lavoro fatto in M021 col Two-clock model, hai già la base tecnica per mettere in
     "pausa" il kernel finché lo sprite non raggiunge il frame di impatto.
**Captured:** 2026-05-17T13:27:42.535Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer this as an M023 animation-sync planning note tied to the future visual stack and combat presentation slices.
**Rationale:** It is a milestone-ahead design constraint about animation-impact timing that depends on later M023 presentation work, not on the current S13 proof slice.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M021

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
**Milestone:** M021

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
