# M015: M013 Closure and Combat Architecture Coherence

**Vision:** Complete what M013 left partial and verify the combat engine is coherent across RON data, per-Digimon blueprint seam, generic combat kernel, canonical events/snapshots, CLI, tests, and future UI.

## Success Criteria

- `cargo test --no-run` no longer fails on stale or missing integration-test declarations.
- `cargo test --no-fail-fast` is run after initial blockers are removed, and every failure is classified before final fixes.
- A concrete source-of-truth audit maps RON data/custom signals, per-Digimon blueprint logic, kernel authority, presentation beat metadata, snapshots, and CLI consumers.
- Clear mixed-pattern drift found during the audit is normalized in M015 unless it is rewrite-scale and explicitly split into a follow-up.
- CLI proof exercises shared action query, event, beat, kernel-observable state, and snapshot surfaces rather than CLI-only combat logic.
- M013/M015 closure artifacts truthfully state what was proven, fixed, deferred, or split forward.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: After this: A concrete failure ledger exists: stale targets, obsolete tests, real regressions, CLI gaps, and M013 validation/artifact gaps are classified with evidence.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: After this: A source-of-truth map shows where gameplay authority, RON data, blueprint logic, kernel state, presentation beats, snapshots, and CLI consumers actually live today.

- [x] **S03: S03** `risk:high` `depends:[]`
  > After this: After this: Clear drift is normalized toward `RON custom signals → per-Digimon blueprint module → kernel hooks → canonical state/events`, with at least one concrete per-Digimon seam established or seeded.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: After this: Tests/docs prove animation/trigger metadata is presentation-side and cannot become gameplay authority, while RON remains data/custom-signal input.

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: After this: The CLI proves combat through shared action query, event, beat, snapshot, and kernel-observable surfaces, with no CLI-only combat path.

- [x] **S06: S06** `risk:medium` `depends:[]`
  > After this: After this: The full test baseline is green or explicitly classified, and M013/M015 closure artifacts truthfully state what was proven, fixed, deferred, or split forward.

## Boundary Map

### S01 → S02
Produces failure ledger and blocker classification.

### S02 → S03
Produces combat authority map and drift ledger.

### S03 → S04
Produces normalized blueprint/kernel seam.

### S04 → S05
Produces presentation metadata non-authority contract.

### S05 → S06
Produces shared-surface CLI proof.

### S06 → downstream
Produces green baseline and truthful closure ledger for M016.
