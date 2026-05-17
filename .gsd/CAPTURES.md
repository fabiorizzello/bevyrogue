# Captures

### CAP-8d133d1a
**Text:** vedo un sacco di file relativi a identità custom dei digimon ma sotto combat/ battery_loop, precision_mind_game(è ancora usato?)? tieni conto che non dobbiamo mantenerci retrocompatibili o altro. voglio che dal kernel spariscano le logiche/strutture dati custom dei specifici digimon. in più i test che non servono realmente si possono togliere. è normale avere tutti quei test per il nostro numeor di LOC del progetto effettivo? Fai un passaggio di prune/remove orphan code/obsoleto. magari tramite un ultimo slice a fine m021
**Captured:** 2026-05-17T06:50:03.872Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer to a final M021 cleanup slice for pruning Digimon-specific kernel logic, orphan code, and obsolete tests.
**Rationale:** This is important follow-up work, but it belongs in a later cleanup slice rather than changing the current S12 scope.
**Resolved:** 2026-05-17T08:18:14Z
**Milestone:** M021

### CAP-af4db4ca
**Text:** anche enemy_counterplay - ogni enemy avrà il suo counterplay, non bisogna inserire la logica nel kernel, se non primitive
**Captured:** 2026-05-17T06:51:21.060Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer as a design constraint for future enemy implementations: keep counterplay logic in per-enemy modules/blueprints, not in kernel code.
**Rationale:** This is a reusable architectural note, but it does not require immediate action in the current slice.
**Resolved:** 2026-05-17T08:18:14Z
**Milestone:** M021

### CAP-7c065a44
**Text:** 1. Architettura Visiva (M023)
   * Niente Fisica/Collider per i VFX: In un RPG a turni deterministico, usare motori fisici (Rapier/Avian)
     per gestire impatti o VFX è un anti-pattern che rompe il determinismo e introduce overhead inutile.
   * VFX tramite Cue e Reactive Bus: Usa il sistema di CueExt (già previsto nella tua CompiledTimeline) per
     inviare segnali visivi. Esempio: il nodo Impact emette un evento, il sistema di rendering cattura
     l'evento (leggendo il target_id) e applica uno shader (es. un componente HitFlash { intensity: 1.0 }) o
     spawna particelle esattamente sulle coordinate dello sprite bersaglio.
   * Action Queue Feedback: Per evitare che il sistema a grafo risulti "incomprensibile" al giocatore, la UI
     di M023 dovrebbe mostrare graficamente gli Intent (es. piccole icone danno/buff) che "esplodono" in sync
     con l'animazione, chiarendo perché una mossa multi-hit o un loop sta avvenendo.

  2. Struttura degli Asset (M021 S12 / M022)
   * Decentrare il RON delle Skill: Il file monolitico assets/data/skills.ron va diviso. Sfrutta
     l'architettura di M021 per avere un file skills.ron globale solo per le mosse comuni, e sposta le skill
     specifiche nelle cartelle dei Digimon (es. assets/data/digimon/agumon/skills.ron). Questo eliminerà il
     rischio di conflitti e isolerà completamente l'identità di ogni personaggio.
**Captured:** 2026-05-17T13:26:46.110Z
**Status:** resolved
**Classification:** defer
**Resolution:** Defer the M023 visual architecture guidance and the M022 asset-structure split to their future milestones rather than changing S13.
**Rationale:** The VFX/Cue/UI feedback ideas target M023 rendering work, and the skill-RON decentralization is an asset-pipeline follow-up better handled in M022 or a later post-M021 slice, not during current M021 verification remediation.
**Resolved:** 2026-05-17T13:28:58Z
**Milestone:** M021

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
