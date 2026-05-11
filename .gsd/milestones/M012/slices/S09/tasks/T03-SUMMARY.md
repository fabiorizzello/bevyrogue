---
id: T03
parent: S09
milestone: M012
key_files:
  - docs/ui_handoff_m012.md
key_decisions:
  - Used actual function names from source (query_action_affordance singular, CombatQuerySnapshot as top-level wrapper) rather than plan sketch names
  - Included query_intent_legality in the entry-point table — it's the point-validation path that UI should use before submitting an action to the engine
  - Separated engine path (build_snapshot_from_ecs, SP bypass) from UI/CLI path (build_snapshot_from_ecs_with_sp, real SP) to make the correct consumer choice explicit
duration: 
verification_result: passed
completed_at: 2026-05-01T17:15:29.829Z
blocker_discovered: false
---

# T03: Created docs/ui_handoff_m012.md — cold-start reference for the next graphical UI milestone covering the legality query API, implemented/deferred/hidden mechanics, data-vs-design simplifications, and consumer rules

**Created docs/ui_handoff_m012.md — cold-start reference for the next graphical UI milestone covering the legality query API, implemented/deferred/hidden mechanics, data-vs-design simplifications, and consumer rules**

## What Happened

Read `src/combat/action_query.rs` to verify all actual function names and snapshot types before writing. Key findings: the main entry point is `query_action_affordance` (singular, not plural as in the plan sketch); `query_all_target_affordances` and `query_target_affordance` are both present; `query_intent_legality` is a point-validation helper not mentioned in the plan but worth including; `CombatQuerySnapshot` is the top-level snapshot (wraps `UnitQuerySnapshot`). Used the gap matrix (updated by T01) and combat_design.md Data Alignment Notes (added by T02) as source material for sections 2 and 3. The doc covers: (1) query entry-point table with signatures, snapshot construction for engine vs UI/CLI paths, and the four status shapes; (2) implemented/deferred/hidden mechanic tables derived from the gap matrix; (3) data-vs-design simplifications summarized from T02's section; (4) a concrete consumer code snippet illustrating the correct branch pattern with no skill-ID hardcoding; (5) key-files table pointing to all integration-relevant files. Kept the doc scannable — five sections, tables over prose, a code example that a new agent can paste and adapt.

## Verification

Ran `test -f docs/ui_handoff_m012.md && grep -q 'action_query' docs/ui_handoff_m012.md && grep -q 'UnitQuerySnapshot' docs/ui_handoff_m012.md` — exit 0, all three checks pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f docs/ui_handoff_m012.md && grep -q 'action_query' docs/ui_handoff_m012.md && grep -q 'UnitQuerySnapshot' docs/ui_handoff_m012.md` | 0 | ✅ pass | 15ms |

## Deviations

Minor naming correction: task plan listed `query_action_affordances()` (plural) and `query_target_affordances()` (plural); actual source has `query_action_affordance` and `query_target_affordance` (singular). Used real names throughout. Added `query_intent_legality` which was not in the plan sketch but is the correct point-validation entry point for UI confirmation paths.

## Known Issues

none

## Files Created/Modified

- `docs/ui_handoff_m012.md`
