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

### CAP-7c065a44 (pt.1 — Asset structure)
**Text:** Decentrare il RON delle Skill: Il file monolitico assets/data/skills.ron va diviso. Sfrutta l'architettura di M021 per avere un file skills.ron globale solo per le mosse comuni, e sposta le skill specifiche nelle cartelle dei Digimon (es. assets/data/digimon/agumon/skills.ron).
**Captured:** 2026-05-17T13:26:46.110Z
**Status:** done
**Resolution:** Completed: skill RON split into per-digimon files (assets/data/digimon/{name}/skills.ron + assets/data/enemies/{name}/skills.ron). Shared demo skills removed — no more monolithic skills.ron.
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
