# Requirements

This file is the current capability contract. Historical per-requirement closure evidence is archived in `.gsd/milestones/M015/M015-VALIDATION.md` and slice summaries.

## Active

None.

## Validated baseline

### M015 Combat Authority Closure Baseline

Validated requirements: R086, R088, R089, R090, R091, R092, R093, R094, R095, R096, R097, R098, R099, R100.

M015 established the current combat baseline:

- green deterministic headless verification baseline;
- failure-ledger-first repair process before broad fixes;
- current combat authority map across RON, action query, turn pipeline, kernel transitions, mechanic hooks, snapshots, CLI, UI, tests, and docs;
- seeded Patamon/Holy Support per-Digimon blueprint seam;
- RON as declarative data/custom-signal layer, not gameplay authority;
- generic branch-light combat kernel and shared hook state;
- presentation metadata (`animation_sequence`, `qte`, beat wording, presentation trigger strings) as non-authoritative;
- real `combat_cli` proof through shared action query, `CombatEvent`, `OnCombatBeat`, `OnKernelTransition`, and `ValidationSnapshot` surfaces;
- truthful supersession of incomplete M013 closure evidence without rewriting M013 history.

Canonical current evidence:

- `docs/combat_current.md`
- `docs/contracts/m015_failure_ledger.md`
- `docs/contracts/combat_authority_map.md`
- `docs/contracts/combat_mixed_pattern_drift_ledger.md`
- `docs/contracts/presentation_metadata_boundary.md`
- `docs/contracts/combat_cli_shared_surface_proof.md`
- `docs/contracts/combat_ui_readiness_gap_matrix.md`
- `scripts/verify_combat_authority_audit.py`
- `scripts/verify_m015_failure_ledger.py`

## Deferred / future work

- Full per-Digimon blueprint migration for the whole roster.
- Full revised 12-Digimon roster behavior and balance validation.
- Full playable CLI UX and windowed presentation pipeline consuming canonical combat surfaces.
- Roguelite Fatigue/run-loop integration.
- Boss conversion and hard-control policy.
- Complete Heavy taxonomy.

## Traceability

| ID set | Status | Evidence |
|---|---|---|
| R086, R088, R089-R100 | validated | `.gsd/milestones/M015/M015-VALIDATION.md`, `docs/contracts/m015_failure_ledger.md`, `docs/combat_current.md` |

## Coverage Summary

- Active requirements: 0
- Validated baseline groups: 1
- Unmapped active requirements: 0
