---
estimated_steps: 31
estimated_files: 10
skills_used: []
---

# T02: Classify mixed-pattern drift and downstream ownership

Create the drift ledger that S03/S04/S05 will use to decide what to normalize locally versus split forward. Expected executor skills: `write-docs`.

Steps:
1. Write `docs/combat_mixed_pattern_drift_ledger.md` with a table of drift items covering live action vs kernel events, runtime registration gaps, app/CLI/headless wiring gaps, missing per-Digimon blueprint ownership, RON custom-signal gaps, obsolete Holy/Twin test assumptions, presentation metadata boundary, snapshot assumptions, fixture drift, and precision placeholder status.
2. For each item, include severity, evidence paths, classification (`clear local drift`, `obsolete/stale test drift`, `rewrite-scale/follow-up candidate`, or `safe placeholder`), downstream owner slice, and recommended next action.
3. Add a section named `Local vs rewrite-scale boundary` that directly satisfies R093 and explains why kernel runtime/event bridge work is local, while broad per-Digimon migration is seeded in S03 rather than completed inside S02.
4. Add sections for R094-R098 downstream handoff notes so each later slice knows which artifact and code seam to consume.
5. Explicitly preserve prior decisions from durable memory: typed hook/kernel seams stay canonical; presentation beats are non-authoritative; shared combat systems should remain branch-light.

Must-haves:
- Drift IDs are stable and named so later slices can refer to them.
- At least one drift item covers each of D1-D11 from S02 research.
- The ledger does not recommend restoring deprecated APIs merely to satisfy stale tests unless a later slice makes an explicit architecture decision.
- S03, S04, and S05 each have at least one concrete handoff item.

Failure Modes:
| Dependency | On error | On timeout | On malformed response |
|------------|----------|------------|------------------------|
| Authority map from T01 | Stop and update `docs/combat_authority_map.md` first if it contradicts the drift ledger. | Not applicable; local files only. | Treat contradictions between docs and source as new drift rows rather than smoothing them over. |

Load Profile:
- Shared resources: local docs/source tree only.
- Per-operation cost: markdown table editing and file existence checks.
- 10x breakpoint: too many drift rows become unmaintainable; group low-risk mechanical fixture drift together.

Negative Tests:
- Malformed inputs: no placeholder drift rows with blank severity/classification/owner.
- Error paths: if evidence cannot be tied to a tracked path, mark it as unsupported and remove or rewrite the row.
- Boundary conditions: distinguish absence of a seam from broken runtime behavior; do not overclaim runtime proof.

Verification:
- `test -s docs/combat_mixed_pattern_drift_ledger.md`
- `grep -E "D1|D11|R093|S03|S04|S05|clear local drift|rewrite-scale" docs/combat_mixed_pattern_drift_ledger.md`

Observability Impact:
- Signals added/changed: tracked drift ledger with severity/classification/owner.
- How a future agent inspects this: open `docs/combat_mixed_pattern_drift_ledger.md` and follow stable drift IDs.
- Failure state exposed: unresolved architecture drift remains visible with downstream ownership rather than being hidden as compile-test noise.

## Inputs

- `docs/combat_authority_map.md`
- `docs/m015_failure_ledger.md`
- `src/combat/kernel.rs`
- `src/combat/follow_up.rs`
- `src/combat/twin_core.rs`
- `src/combat/holy_support.rs`
- `src/combat/battery_loop.rs`
- `src/combat/predator_loop.rs`
- `src/combat/precision_mind_game.rs`

## Expected Output

- `docs/combat_mixed_pattern_drift_ledger.md`

## Verification

test -s docs/combat_mixed_pattern_drift_ledger.md && grep -E "D1|D11|R093|S03|S04|S05|clear local drift|rewrite-scale" docs/combat_mixed_pattern_drift_ledger.md

## Observability Impact

Adds a stable diagnostic ledger that exposes architecture drift severity, evidence, and downstream owner for later normalization slices.
