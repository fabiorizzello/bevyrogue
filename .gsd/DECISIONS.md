# Decisions Register

This file is squashed to current decisions only. Historical decision rows are recoverable from milestone artifacts and git history; current implementation should follow this snapshot.

## Current decisions

| ID | Scope | Current decision | Rule |
|---|---|---|---|
| CD001 | Combat extension architecture | Unique Digimon, enemy, and party behavior extend through typed hook/blueprint seams. | Do not add central per-entity branches inside shared combat systems. Use per-Digimon Rust blueprints plus generic kernel transitions/hooks. |
| CD002 | Combat kernel | Tactical Cycle, Strain, Flow, Fatigue, beats, tags, and mechanic transitions live in typed Rust kernel/state. | Keep the kernel generic and branch-light. Mechanic modules own shared state machines; blueprints feed them. |
| CD003 | Presentation sync | Presentation beats and animation/QTE metadata are non-authoritative. | `animation_sequence`, `qte`, beat wording, and presentation trigger strings may synchronize UI/CLI/VFX only after combat emits canonical events/snapshots. They must not decide damage, legality, state transitions, status outcomes, resources, targeting, victory, or defeat. |
| CD004 | RON role | RON owns numbers, targeting declarations, costs, metadata, presentation metadata, and typed custom-signal intent. | RON is not a hidden gameplay scripting engine. Unique behavior enters Rust through typed `custom_signals`. |
| CD005 | Blueprint ownership | Long-term Digimon identity belongs in per-Digimon Rust blueprint modules; shared mechanic modules are primitives. | Patamon/Holy Support is the seeded proof. Full roster migration remains future work. |
| CD006 | Kernel transitions in blueprint flow | `CombatKernelTransition` is canonical observable/mutation output after blueprint resolution. | Unique Digimon behavior starts in the blueprint module, then emits generic transitions. Do not put unique identity logic in the transition enum itself. |
| CD007 | CLI/UI authority | CLI, UI, AI, logs, and tests consume shared action query, `CombatEvent`, beats, kernel state, and `ValidationSnapshot`. | No consumer-owned gameplay path. No CLI/windowed skill-ID-specific legality rules. |
| CD008 | Test repair policy | Stale tests are rewritten to current source-of-truth contracts. | Do not restore removed APIs just to make tests green unless a new explicit architecture decision restores them. |
| CD009 | M013/M015 provenance | Missing or contradictory M013 closure evidence remains a historical gap superseded by M015 proof. | Do not backfill M013 as if proof existed then. Current baseline lives in M015 docs and `docs/combat_current.md`. |
| CD010 | Next architecture direction | Next combat architecture work should expand per-Digimon blueprint migration before presentation polish. | Prefer M016-style vertical slices that migrate one high-risk Digimon/mechanic through RON custom signal -> Rust blueprint -> kernel transition -> event/snapshot/CLI proof. |

## Canonical references

- `docs/combat_current.md`
- `docs/contracts/combat_authority_map.md`
- `docs/contracts/combat_mixed_pattern_drift_ledger.md`
- `docs/contracts/presentation_metadata_boundary.md`
- `docs/contracts/combat_cli_shared_surface_proof.md`
- `docs/contracts/m015_failure_ledger.md`
- `.gsd/milestones/M015/M015-VALIDATION.md`

---

## Decisions Table

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 |  | architecture | Per-Digimon blueprint implementation for Twin Core mechanics. | Agumon and Gabumon blueprints process character-specific custom signals into Twin Core tag transitions. | Decouples character-specific mechanic logic from the core combat kernel while maintaining data-driven control via skills.ron. | Yes | agent |
