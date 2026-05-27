# Captures

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

