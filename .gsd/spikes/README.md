---
name: M017 pre-planning spikes
created: 2026-05-12
purpose: De-risk M017 (Digimon kit + animation FSM + blueprint plugins) by validating architectural assumptions on a small concrete sample before milestone planning.
status: scaffolded
---

# M017 Pre-Planning Spike Portfolio

Five short investigations that produce findings consumed by `gsd_plan_milestone M017`.
Output: `RESEARCH.md` (always) + optional `DECISION.md` / `sketches/` per spike.

## Spike index

| ID  | Directory                          | Goal                                                                                  | Effort | Blocks M017? |
|-----|------------------------------------|----------------------------------------------------------------------------------------|--------|--------------|
| SP1 | `spike-kernel-primitives/`         | Audit `src/combat/` vs §02-02b/§02-08 design primitives; produce gap table.            | 4–8h   | yes          |
| SP2 | `spike-blueprint-api/`             | Stabilize blueprint plugin trait + registry + migration plan for existing 4 blueprints.| 4–6h   | yes          |
| SP3 | `spike-skill-dsl-coverage/`        | Map 24 skills × `Effect` variants; identify missing variants.                          | 2–3h   | partial      |
| SP4 | `spike-asset-schema/`              | Validate `clip.ron` + `animation_fsm.ron` on Agumon `baby_flame` + `baby_burner`.       | 2–4h   | no           |
| SP5 | `spike-pipeline-determinism/`      | Confirm sprite pipeline produces reproducible `_atlas.json` post commit `a4fea2b`.     | 30min  | no           |

## Execution order

1. **SP1 + SP3** in parallel — kernel + DSL audit (close kernel↔skill contract).
2. **SP2** after SP1 — needs primitive surface from SP1 to design trait.
3. **SP4 + SP5** in parallel, can run any time.
4. Then `gsd_plan_milestone M017`.

## Promotion of findings

- **Decisions** → `gsd_save_decision` → `.gsd/DECISIONS.md`.
- **Rules / gotchas** → `capture_thought` → memory store.
- **Sample assets (SP4)** → baseline for slice S04.
- **Gap tables (SP1/SP3)** → input to milestone planning (scope of S01/S02/S03).
- **Migration plan (SP2)** → input to slices S03b/d/f/g.

## Conventions

- `RESEARCH.md` = primary deliverable, audience is the M017 planner.
- `sketches/` = throwaway illustrative code, **never compiled**, **never imported** by `src/`.
- Spikes **never edit** `docs/future_design_draft/` (canon lock-in).
- Spikes **never edit** `src/` (validation only — implementation belongs in M017 slices).
