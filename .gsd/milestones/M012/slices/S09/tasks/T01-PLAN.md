---
estimated_steps: 10
estimated_files: 2
skills_used: []
---

# T01: Reclassify gap matrix to reflect S02–S08 delivery and fix doc-contract test

Update `docs/combat_ui_readiness_gap_matrix.md` to reclassify all `ToFixNow` entries. S02–S08 resolved every `ToFixNow` item: Revive, Ally Toughness, Zero-max Toughness, Energy caps, SP/ultimate readiness, Attacker state, Commander target, Structured failure reasons, and Windowed active unit are now `Implemented`. Row/AllEnemies TargetShape and Mixed-effect (angemon_ult) are now `Deferred` (explicitly gated via `UnimplementedTargetShape`/`UnimplementedEffect`). Enemy counterplay/telegraphs is now `Deferred` with typed declarations from S08.

For each reclassified row, update the 'Current evidence' column to reflect post-S08 state and the 'UI truth risk' column to reflect the resolved status. Keep the 'Contract note' column unchanged (it describes the contract, which remains valid).

The doc-contract test `tests/ui_readiness_gap_matrix_docs.rs` has a test `gap_matrix_uses_each_required_status_family` that asserts `| \`ToFixNow\` |` appears in at least one table row. Since M012 resolved all ToFixNow items, update this test to remove `ToFixNow` from the required-in-table-rows list (but keep the vocabulary definition test unchanged, since the classification vocabulary section still defines `ToFixNow` as a concept).

Slice: S09 — Doc/data alignment and UI handoff docs
Milestone: M012

R085 relevance: The gap matrix IS the R085 contract doc. Updating it to reflect what S01–S08 actually delivered is the primary R085 validation artifact.

Key constraints:
- The vocabulary definition section must still mention all four statuses (`Implemented`, `ToFixNow`, `Deferred`, `Hidden`) so the vocabulary test passes
- All other tests in `ui_readiness_gap_matrix_docs.rs` must remain green — mechanic names, reason codes, boundary text, etc. must stay in the doc
- The `Downstream use rules` section can note that `ToFixNow` is retained as vocabulary for future milestones even though M012 resolved all current instances

## Inputs

- ``docs/combat_ui_readiness_gap_matrix.md` — current gap matrix with stale ToFixNow classifications`
- ``tests/ui_readiness_gap_matrix_docs.rs` — doc-contract test that asserts on matrix vocabulary and content`

## Expected Output

- ``docs/combat_ui_readiness_gap_matrix.md` — updated with all ToFixNow entries reclassified to Implemented or Deferred`
- ``tests/ui_readiness_gap_matrix_docs.rs` — updated to not require ToFixNow in table rows`

## Verification

cargo test-dev --test ui_readiness_gap_matrix_docs && cargo test-dev --test skill_legality_contract_docs
