# S04: S04

**Goal:** Introdurre SignalBus, bridge eventi->signal, PassiveRunner e dispatcher BlueprintSignal.
**Demo:** Renamon kitsune_grace verde; JSONL Blueprint round-trip; debug_assert mismatch.

## Must-Haves

- Reactive passives e Blueprint transitions passano end-to-end con JSONL round-trip e guard taxonomy.

## Proof Level

- This slice proves: End-to-end integration proof sul percorso reactive.

## Integration Closure

Chiude il layer reattivo necessario per blueprint/passive sopra il kernel generico.

## Verification

- CombatKernelTransition::Blueprint e SignalBus rendono osservabili i trigger reattivi.

## Tasks

- [x] **T99: Riallineato il DB per S04.** `est:XS`
  Task amministrativo creato per riallineare il tracking GSD alla slice già completata e già documentata tramite summary su disco.
  - Verify: gsd_milestone_status shows task exists and slice can be closed
