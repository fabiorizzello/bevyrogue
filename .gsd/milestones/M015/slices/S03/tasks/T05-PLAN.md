---
estimated_steps: 14
estimated_files: 8
skills_used: []
---

# T05: Refresh contracts, docs, and final S03 verification ledger

Close the slice by aligning stale Holy/Twin contract tests and architecture docs with the new seeded seam, then run targeted verification and record any remaining whole-suite blockers truthfully.

Skills expected: `write-docs`, `tdd`, `verify-before-complete`.

Steps:
1. Finish rewriting `holy_support_affordance` and any remaining Holy/Twin/snapshot tests away from removed APIs and toward current action query, kernel state, and validation snapshot contracts.
2. Update `docs/combat_authority_map.md` and `docs/combat_mixed_pattern_drift_ledger.md` to mark D1/D2/D3/D9/D10-style local drift as normalized or seeded where the implementation proves it, and preserve rewrite-scale follow-ups for full blueprint migration.
3. Run the authority verifier and all targeted S03 tests; then run `cargo test --no-run` once to discover remaining blockers.
4. If `cargo test --no-run` still fails for S04-S06-owned or pre-existing reasons, update `docs/m015_failure_ledger.md` with the owner/classification instead of claiming a green full baseline.

Must-haves:
- Documentation and tests agree on the new source-of-truth flow and do not overclaim full CLI/presentation/final-regression proof.
- Targeted S03 tests pass.
- Any remaining full-suite blocker is classified with evidence and owner.

Failure Modes (Q5): docs can drift from code if the verifier only checks keywords; strengthen claim-scoped checks if needed. Whole-suite failures must not be hidden by targeted passing tests.
Load Profile (Q6): no runtime load risk; this task affects docs/tests and verifier checks.
Negative Tests (Q7): verifier should catch missing requirement/drift references, placeholder text, or tracked-path omissions after docs are updated.

## Inputs

- ``docs/combat_authority_map.md` — S02 source-of-truth map to update with the implemented seam.`
- ``docs/combat_mixed_pattern_drift_ledger.md` — S02 drift ledger with D1-D11 owner/classification rows.`
- ``scripts/verify_combat_authority_audit.py` — executable audit completeness verifier.`
- ``docs/m015_failure_ledger.md` — remaining no-run failure classification surface.`
- ``tests/holy_support_affordance.rs` — stale Holy Support affordance contract to normalize.`
- ``tests/patamon_blueprint_seam.rs` — end-to-end proof created in T03/T04.`

## Expected Output

- ``tests/holy_support_affordance.rs` — stale affordance assertions rewritten to current query/snapshot contract or consciously deferred with ledger entry.`
- ``tests/twin_core_mechanics.rs` — final current-contract assertions after T01/T02 changes.`
- ``tests/twin_core_integration.rs` — final current-contract assertions after T01/T02 changes.`
- ``tests/validation_snapshot.rs` — final snapshot assertions for registered kernel resources.`
- ``docs/combat_authority_map.md` — updated source-of-truth map showing the seeded Patamon blueprint seam.`
- ``docs/combat_mixed_pattern_drift_ledger.md` — updated D-row status/follow-up boundary for S03-normalized drift.`
- ``scripts/verify_combat_authority_audit.py` — strengthened if needed to verify the new seam claims.`
- ``docs/m015_failure_ledger.md` — remaining `cargo test --no-run` blockers classified if the whole suite is not green.`

## Verification

cargo test --test patamon_blueprint_seam --test holy_support_resolution --test holy_support_roster_contract --test holy_support_affordance --test event_stream --test twin_core_mechanics --test twin_core_integration --test validation_snapshot --test battery_loop_kernel --test predator_loop_kernel && python3 scripts/verify_combat_authority_audit.py
