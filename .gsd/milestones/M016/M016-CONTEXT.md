# M016: Per-Digimon Blueprint Migration and Roster Combat Identity

## Goal

Migrate revised combat identity from shared mechanic primitives into deliberate per-Digimon Rust blueprint ownership, while preserving M015 authority boundaries. The core objective is expanding the seed concept established by Patamon/Holy Support into the full roster, preventing unique mechanics from leaking into shared system branching.

## Constraints and Boundaries

- **RON remains declarative:** It declares typed custom-signal intent only, not gameplay execution authority.
- **Shared kernel remains generic:** Generic `CombatKernelTransition` values and shared hooks mutate canonical state. No character-specific branching inside `src/combat/turn_system/` or `src/combat/resolution.rs`.
- **Presentation remains non-authoritative:** Metadata like `animation_sequence`, `qte`, or text-triggers remain visual/audio cues only.
- **CLI/UI are consumers:** Consumers must use the shared action query, events, beats, kernel state, and snapshots rather than determining skill-ID-specific legality locally.

## Starting point

Review current architecture bounds before drafting work slices:
1. `docs/contracts/m015_failure_ledger.md`
2. `docs/combat_current.md`
3. `docs/contracts/combat_authority_map.md`
4. `docs/contracts/combat_mixed_pattern_drift_ledger.md`
5. `docs/contracts/presentation_metadata_boundary.md`
6. `docs/contracts/combat_cli_shared_surface_proof.md`

## Strategy

Tackle one high-risk existing primitive at a time, migrating it fully into the RON custom signal -> Rust blueprint -> kernel transition -> event/snapshot/CLI proof pipeline. 

Candidate slices in recommended priority:
1. **Tentomon/Kabuterimon Battery loop**
2. **Dorumon/DORUgamon Predator loop**
3. **Renamon/Kyubimon Precision loop**
4. **Agumon/Gabumon Twin Core refinement**

## Acceptance Criteria

Each migrated Digimon must prove:
- `assets/data/skills.ron` defines `custom_signals` instead of ad-hoc metadata fields for unique mechanics.
- A dedicated per-Digimon Rust module under `src/combat/blueprints/` handles signal interpretation.
- Shared mechanic modules (e.g. `src/combat/battery_loop.rs`) expose state and generic transitions (e.g. `BatteryLoopTransition`).
- Action query and real-binary tests continue to observe the same outcomes through shared ECS validation snapshots and `CombatEvent` streams.