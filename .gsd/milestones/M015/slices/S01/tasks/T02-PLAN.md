---
estimated_steps: 23
estimated_files: 8
skills_used: []
---

# T02: Build the comprehensive test failure ledger

Turn the research inventory into a tracked, executor-ready failure ledger that classifies compile blockers, runtime reds, stale/obsolete tests, mechanical fixture drift, missing artifacts, and CLI gaps without attempting broad repairs.

Expected executor skills: `test`, `verify-before-complete`.

## Steps
1. Re-run or focus-check enough Cargo commands after T01 to confirm the current blocker set, using `gsd_exec` for noisy output and preserving only summaries in the ledger.
2. Populate `docs/m015_failure_ledger.md` with tables for blocking command, explicit manifest tests, auto-discovered compile blockers, runtime reds, artifact gaps, CLI gap, and downstream owner recommendations.
3. Classify stale Holy Support APIs and Twin Core state assertions as architecture-drift candidates unless current source proves they are simple fixture fixes.
4. Keep the ledger factual: every row needs command/evidence, classification, affected files or targets, and planned downstream owner (S02/S03/S05/S06 or this slice).

## Must-Haves
- [ ] Every failure signal listed in S01 research is represented or explicitly marked superseded by a newer command result.
- [ ] No row remains `unknown`/`TBD`; uncertain cases must be labeled as `real regression or stale assertion candidate` with a downstream owner.
- [ ] The ledger warns not to restore old `HolySupportAffordance`, `Effect::HolySupportRequest`, `TargetShape::SelfTarget`, or `TwinCoreState.resonance/heat` APIs without S02/S03 confirmation.

## Failure Modes
| Dependency | On error | On timeout | On malformed response |
|------------|----------|------------|-----------------------|
| Individual Cargo target commands | Record the failing target and classify from compiler/runtime excerpt | Record timeout separately from failure and defer to S06 if repeated | Re-run focused target once; if still ambiguous classify as candidate with evidence gap |

## Load Profile
- **Shared resources**: Cargo target cache and test binaries.
- **Per-operation cost**: Dozens of focused no-run/runtime target checks if research evidence is stale.
- **10x breakpoint**: Re-running the entire suite for every edit wastes time; batch with `gsd_exec` and summarize.

## Negative Tests
- **Malformed inputs**: Ledger verifier in T04 must fail if required category headings or target names are missing.
- **Error paths**: Include non-zero command exits as expected evidence, not executor failure, when they are being classified.
- **Boundary conditions**: Empty category tables are not acceptable for known blocker classes.

## Inputs

- ``docs/m015_failure_ledger.md``
- ``tests/holy_support_affordance.rs``
- ``tests/holy_support_roster_contract.rs``
- ``tests/twin_core_integration.rs``
- ``tests/holy_support_mechanics.rs``
- ``tests/holy_support_resolution.rs``
- ``tests/ui_readiness_gap_matrix_docs.rs``
- ``src/data/skills_ron.rs``
- ``src/data/units_ron.rs``
- ``src/combat/action_query.rs``
- ``src/combat/twin_core.rs``
- ``src/bin/combat_cli.rs``

## Expected Output

- ``docs/m015_failure_ledger.md``

## Verification

`grep -E "battery_loop_resolution|holy_support_affordance|holy_support_roster_contract|twin_core_integration|ui_readiness_gap_matrix_docs|combat_cli" docs/m015_failure_ledger.md` finds all required signals, and focused commands used for classification are listed in the ledger.

## Observability Impact

The ledger becomes the diagnostic inspection surface for downstream slices, converting transient compile/runtime failures into stable categorized evidence.
