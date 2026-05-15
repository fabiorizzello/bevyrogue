# S03: S03

**Goal:** Provare mode parity e invarianza tra clock headless/windowed nel BeatRunner.
**Demo:** DryRun≡Execute≡Preview verde; two-clock verde; circuit breaker @256.

## Must-Haves

- I tre mode condividono lo stesso stream strutturale; HeadlessAuto e Windowed convergono; il circuit breaker ferma loop infiniti.

## Proof Level

- This slice proves: Integration proof con run deterministici e stream comparison.

## Integration Closure

Chiude le invarianti del runner prima dell’integrazione con il pipeline produttivo.

## Verification

- Halt del runner resta tracciabile e i mismatch clock/mode diventano rilevabili.

## Tasks

- [x] **T01: Riallineato il DB per S03 tramite task amministrativo.** `est:XS`
  Task amministrativo creato per riallineare il tracking GSD allo stato reale del lavoro già completato fuori dal flusso operativo registrato.
  - Verify: gsd_milestone_status shows slice/task progression for M021
