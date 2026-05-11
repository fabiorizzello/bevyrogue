# M016: Per-Digimon Blueprint Migration and Roster Combat Identity

**Vision:** Migrate revised combat identity from shared mechanic primitives into deliberate per-Digimon Rust blueprint ownership, while preserving M015 authority boundaries.

## Success Criteria

- skills.ron uses custom_signals for the migrated roster.
- Blueprints handle signal interpretation.
- Generic kernel transitions are used for state changes.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: After this: Battery mechanics operate entirely through `custom_signals` and the Tentomon blueprint, with CLI proof passing.

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: After this: Predator loop logic operates through the Dorumon blueprint.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: After this: Precision loop logic operates through the Renamon blueprint.

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: After this: Twin Core mechanics operate through their respective blueprints.

- [x] **S05: S05** `risk:low` `depends:[]`
  > After this: All M016 slices have matching summaries and UAT files on disk.

## Boundary Map

### S01 → S02
Produces:
- Validated pattern for stateful mechanic blueprints (`BatteryLoopTransition` equivalent generic transitions).
Consumes:
- nothing (first slice)

### S02 → S03
Produces:
- Validated pattern for status-driven conditional damage logic (Predator loop).
Consumes:
- The general `CombatKernelTransition` emission pattern from S01.

### S03 → S04
Produces:
- Blueprint handling for accuracy/evasion modifiers and precision checks.
Consumes:
- Previous blueprint patterns.
