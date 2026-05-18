# S02: S02

**Goal:** Portare nel runtime la Timeline FSM con BeatRunner e validazione boot-time dei riferimenti.
**Demo:** Fixture OnTurnStart kills target verde; validate_timeline_refs scopre typo; LoopFrame single-level su chain_bolt port.

## Must-Haves

- CompiledTimeline, BeatRunner e validate_timeline_refs sono attivi e provati da test di integrazione.

## Proof Level

- This slice proves: Integration tests headless + validation hooks a boot.

## Integration Closure

Chiude il ponte tra timeline pure-data e runtime validator nel CombatPlugin.

## Verification

- Errori di riferimento timeline falliscono prima del runtime con contesto diagnostico.

## Tasks

- [x] **T01: Riallineato il DB per S02 tramite task amministrativo.** `est:XS`
  Task amministrativo creato per riallineare il tracking GSD allo stato reale del lavoro già completato fuori dal flusso operativo registrato.
  - Verify: gsd_milestone_status shows slice/task progression for M021
