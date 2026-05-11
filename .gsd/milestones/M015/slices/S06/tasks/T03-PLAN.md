---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T03: Repair SkillDef fixture drift without restoring obsolete APIs

Update stale inline `SkillDef` fixtures with current neutral fields (`custom_signals`, `animation_sequence`, `qte`) or safe `..Default::default()`, preserving current source-of-truth contracts. Expected executor skills: `design-an-interface`, `tdd`, `verify-before-complete`, `write-docs`.

Steps: (1) re-run no-run or scoped compile and group `SkillDef` failures; (2) patch only current-field omissions in known fixture files and any same-class files revealed by the compiler; (3) grep/check that obsolete variants such as `TargetShape::SelfTarget` or removed Holy Support affordance APIs were not restored; (4) update the ledger.

Must-haves: fixture edits are data-schema drift repairs, not gameplay changes; obsolete expectations have named replacement coverage. Failure modes/Q5-Q7: compiler cascades are fixed by earliest schema class; malformed fixture assumptions are rewritten to current contracts rather than shimmed in source.

## Inputs

- ``docs/m015_failure_ledger.md` — classified SkillDef blocker rows.`
- ``tests/status_effect_apply.rs` — representative stale `SkillDef` fixture surface.`
- ``tests/engine_legality_integration.rs` — legality/query fixture surface.`
- ``tests/action_affordance_consumers.rs` — affordance consumer fixture surface.`
- ``tests/action_affordance_query.rs` — query fixture surface.`

## Expected Output

- ``tests/status_effect_apply.rs` — current `SkillDef` fixture shape where needed.`
- ``tests/engine_legality_integration.rs` — current `SkillDef` fixture shape where needed.`
- ``tests/action_affordance_consumers.rs` — current `SkillDef` fixture shape where needed.`
- ``tests/action_affordance_query.rs` — current `SkillDef` fixture shape where needed.`
- ``docs/m015_failure_ledger.md` — SkillDef blockers retired with evidence.`

## Verification

`cargo test --no-run` via `gsd_exec`; remaining failures, if any, must not be from known `SkillDef` field drift. Also run a grep/check for forbidden restored obsolete symbols before claiming completion.
