# M013: M013: Combat architecture revision + animation beat pipeline

**Vision:** Re-spec the combat core around the revised design in docs/combat_design_revised.md: typed combat kernel, redesigned 6+6 Digimon roster, combat-authoritative beat metadata for presentation, and an interactive CLI proof path that stays aligned with the future UI contract.

## Success Criteria

- The revised combat kernel models Tactical Cycle, Strain, Flow, Fatigue, tag lifetimes/consumption, and canonical beat emission as typed gameplay state.
- All 12 Digimon are redesigned against the revised combat spec with explicit resources, payoff windows, and inter-line interactions.
- Core action beats can be mapped to sound/VFX/damage/extra-hit presentation markers without making animation authoritative for gameplay timing.
- The interactive combat CLI exercises the same shared combat pipeline, event surfaces, and affordance model that the future UI will consume.
- The system remains deterministic, headless-safe, and free of hardcoded per-Digimon gameplay branching in Rust.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: A headless test can emit Tactical Cycle / Strain / Flow / Fatigue transitions and canonical beat markers for a representative action.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: The Fire/Ice family builds and spends shared resonance/heat resources through headless tests.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: Renamon/Kyubimon use momentum-style timing and exposed windows through the shared beat pipeline.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: Patamon/Angemon use Grace / Martyr Light style resources and surface through the same affordance model.

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: Tentomon/Kabuterimon implement the revised battery/circuit loop through the same combat state model.

- [x] **S06: S06** `risk:medium` `depends:[]`
  > After this: Dorumon/Dorugamon implement exploit/prey-lock/berserk-style payoffs in the revised kernel.

- [x] **S07: S07** `risk:high` `depends:[]`
  > After this: The interactive CLI plays the revised combat and exposes beat markers in the same shared surfaces the future UI will use.

## Boundary Map

## Boundary Map

### S01 → S02, S03, S04, S05, S06, S07
Produces:
- typed combat kernel primitives for Tactical Cycle, Strain, Flow, Fatigue, and canonical beat emission
- `assets/data/combat_beats.ron` contract and parser helpers for presentation beat markers
- canonical ambiguity defaults for values that the revised design leaves unspecified

Consumes:
- M012 legality/query surface (`src/combat/action_query.rs`, `src/data/skills_ron.rs`) as the current shared affordance baseline
- current combat event bus and turn pipeline as the starting integration seam

### S02 → S07
Produces:
- revised Fire/Ice line kits and supporting data for Agumon, Greymon, Gabumon, Garurumon

Consumes:
- S01 typed kernel and beat contract

### S03 → S07
Produces:
- revised Tempo/precision line kits and supporting data for Renamon, Kyubimon

Consumes:
- S01 typed kernel and beat contract

### S04 → S07
Produces:
- revised Holy/support line kits and supporting data for Patamon, Angemon

Consumes:
- S01 typed kernel and beat contract

### S05 → S07
Produces:
- revised Battery line kits and supporting data for Tentomon, Kabuterimon

Consumes:
- S01 typed kernel and beat contract

### S06 → S07
Produces:
- revised Digital/predator line kits and supporting data for Dorumon, Dorugamon

Consumes:
- S01 typed kernel and beat contract

### S07
Produces:
- interactive CLI wiring for the revised combat pipeline
- end-to-end proof that the CLI consumes the same combat/event/beat surfaces the future UI will use

Consumes:
- S01 through S06 outputs for the combat kernel, roster data, and beat metadata
