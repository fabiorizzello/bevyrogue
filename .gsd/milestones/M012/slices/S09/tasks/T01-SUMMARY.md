---
id: T01
parent: S09
milestone: M012
key_files:
  - docs/combat_ui_readiness_gap_matrix.md
  - tests/ui_readiness_gap_matrix_docs.rs
key_decisions:
  - Reclassified Row/AllEnemies TargetShape and angemon_ult to Deferred (not Implemented) since they are explicitly gated rather than fully resolved
  - Kept all contract notes verbatim as instructed; only updated Current evidence and UI truth risk columns
  - Updated Downstream use rules to scope ToFixNow retention as vocabulary for future milestones
duration: 
verification_result: passed
completed_at: 2026-05-01T17:10:24.384Z
blocker_discovered: false
---

# T01: Reclassified all ToFixNow gap matrix entries to Implemented/Deferred per S02–S08 delivery and updated doc-contract test accordingly

**Reclassified all ToFixNow gap matrix entries to Implemented/Deferred per S02–S08 delivery and updated doc-contract test accordingly**

## What Happened

The gap matrix doc (`docs/combat_ui_readiness_gap_matrix.md`) had 9 `ToFixNow` entries that S02–S08 resolved. Each was reclassified:

- **Revive** → `Implemented`: KO ally targeting enforced, `TargetNotKo` reason exposed via shared target query.
- **Row/AllEnemies TargetShape** → `Deferred`: explicitly gated by `UnimplementedTargetShape`; no silent single-target fallback.
- **Mixed-effect target semantics (angemon_ult)** → `Deferred`: gated by `UnimplementedEffect`; UI gets a queryable deferred reason.
- **Ally Toughness** → `Implemented`: toughness affordances restricted to enemies with real break bars; ally state excluded at spawn.
- **Zero-max enemy Toughness** → `Implemented`: hidden or declared not-breakable with a stable reason; no `0/0` bars rendered.
- **Energy caps** → `Implemented`: cap enforcement consistent across grant paths; `EnergyCapReached` exposed in query.
- **SP/ultimate readiness** → `Implemented`: `SpShortfall`, `UltimateNotReady`, and effective SP cost returned before execution.
- **Attacker state** → `Implemented`: `AttackerKo`, `AttackerStunned`, `NotActiveUnit`, `WrongPhase` exposed as shared preflight affordances.
- **Commander target** → `Implemented`: `TargetIsCommander` shared reason; engine failure text derives from same code.
- **Structured failure reasons** → `Implemented`: stable reason codes cover action/target/resource/implementation; display strings derived from codes.
- **Windowed active unit** → `Implemented`: windowed UI consumes shared active-unit/action affordance query; no separate rule.

For each reclassified row, 'Current evidence' was updated to reflect post-S08 state and 'UI truth risk' updated to reflect the resolved status. All contract notes were kept verbatim. The Downstream use rules section rule 1 was updated to say "Later milestones" instead of "Later M012 slices", and rule 2 was rewritten to note `ToFixNow` is retained as vocabulary for future milestones even though M012 resolved all instances.

The doc-contract test `gap_matrix_uses_each_required_status_family` was updated to remove `"| \`ToFixNow\` |"` from the required-in-table-rows list (with an explanatory comment). The vocabulary definition test remains unchanged and still passes because the Classification vocabulary section still defines `ToFixNow` as a concept.

## Verification

Ran `cargo test --test ui_readiness_gap_matrix_docs` — all 7 tests pass. Ran `cargo test --test skill_legality_contract_docs` — all 10 tests pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ui_readiness_gap_matrix_docs` | 0 | ✅ pass | 220ms |
| 2 | `cargo test --test skill_legality_contract_docs` | 0 | ✅ pass | 210ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `docs/combat_ui_readiness_gap_matrix.md`
- `tests/ui_readiness_gap_matrix_docs.rs`
