# S12: RosterEntry blueprint-keyed + ValidationSnapshot from registry — UAT

**Milestone:** M021
**Written:** 2026-05-17T09:12:02.841Z

## UAT Type
Regression / contract verification

## Preconditions
- The repository is checked out at `/home/fabio/dev/bevyrogue`.
- The focused slice tests and cargo checks can run locally.
- No manual fixture edits are required.

## Steps
1. Run the focused slice verification suite:
   - `cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime`
2. Run the build checks:
   - `cargo check`
   - `cargo check --features windowed`
3. Run the structural grep checks:
   - Search for retired roster fields: `TwinCoreRosterMetadata|HolySupportRosterMetadata|pub twin_core:|pub holy_support:`
   - Search for retired shared validation fields: `ValidationTwinCoreSnapshot|holy_support: Option<|predator_loop: Option<|battery_loop: Option<|precision_mind_game: Option<`
4. Inspect the validation snapshot output in the tests to confirm owner-keyed sections are rendered deterministically and missing optional sections become `none`.

## Expected Outcomes
- All focused tests pass.
- Both cargo check modes pass.
- Retired digimon-named roster and validation seams do not appear in the shared source/tests surface.
- Validation snapshots render owner-keyed sections in stable order.
- Missing optional blueprint resources degrade to absent/none output rather than failing snapshot capture.

## Edge Cases
- Blueprint resources may be absent in a minimal world; capture should still succeed with `none` sections.
- Owner-keyed metadata order should stay stable across round trips.
- Validation contributor registration order should not affect rendered output.

## Not Proven By This UAT
- It does not prove future blueprint auto-discovery or per-digimon asset-directory loading.
- It does not exercise unrelated combat balance or gameplay tuning.
- It does not validate performance characteristics beyond basic compile and test health.
