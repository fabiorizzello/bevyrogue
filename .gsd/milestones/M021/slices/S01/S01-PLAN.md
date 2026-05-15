# S01: S01

**Goal:** Estrarre CombatPlugin e introdurre le primitive kernel/framework iniziali in src/combat/api/.
**Demo:** cargo check headless + windowed puliti; CombatPlugin in main.rs; src/combat/api/ con i 7 file primitive; cast_id su CombatEvent; canary Intent::DealDamage end-to-end.

## Must-Haves

- CombatPlugin è montato in main.rs, cast_id è propagato negli eventi, e il canary DealDamage gira end-to-end.

## Proof Level

- This slice proves: Compile + integration tests mirati + grep strutturali.

## Integration Closure

Chiude l’estrazione del plugin e allinea il bootstrap del runtime sul nuovo layer api/.

## Verification

- CombatEvent porta cast_id lungo il flusso e mantiene il bus osservabile.

## Tasks

- [x] **T01: Riallineato il DB per S01 tramite task amministrativo.** `est:XS`
  Task amministrativo creato per riallineare il tracking GSD allo stato reale del lavoro già completato fuori dal flusso operativo registrato.
  - Verify: gsd_milestone_status shows slice/task progression for M021
