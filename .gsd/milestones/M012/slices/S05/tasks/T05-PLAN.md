---
estimated_steps: 10
estimated_files: 5
skills_used: []
---

# T05: Run focused S05 contract verification and tighten docs if needed

Why: After the explicit remediation task, S05 still needs the original closure sweep across pure query contracts, runtime ECS wiring, canonical RON content, and docs. This task should only make small alignment fixes; any new runtime blocker should be treated as a blocker rather than hidden as documentation drift.

Do:
1. Run the focused verification commands: `cargo test-dev --test action_affordance_query`, `cargo test-dev --test resource_caps`, `cargo test-dev --test form_identity`, and `cargo test-dev skills_ron`.
2. If a focused test fails, fix the smallest contract/code/doc mismatch rather than weakening assertions. Keep reason-code assertions machine-readable.
3. Ensure docs do not claim Tamer Gauge/Commands or Child gauge boost are executable; they must say deferred/queryable until a later implementation slice.
4. Ensure docs and tests describe Form Identity hidden/internal semantics accurately: hidden skills are not user-facing affordances, but selected internal follow-up effects can execute through the runtime pipeline.
5. Optionally run `cargo check --features "dev windowed"` if public query type changes create concern for downstream UI compilation, but do not make windowed compile a blocker unless this task touched windowed code.

Failure Modes (Q5): failures may come from stale tests/docs after remediation; do not hide real pipeline failures as docs-only changes.
Negative Tests (Q7): keep assertions for cap exhaustion, no overreported `EnergyGained`, hidden/deferred Form Identity query behavior, DORUgamon/Angemon target behavior, and Tamer/Child deferred declarations.

Done when: fresh verification output proves the S05 stopping condition or documents any precisely scoped pre-existing non-S05 failure.

## Inputs

- `.gsd/milestones/M012/slices/S05/S05-PLAN.md`
- `tests/action_affordance_query.rs`
- `tests/resource_caps.rs`
- `tests/form_identity.rs`
- `docs/skill_legality_contract.md`
- `docs/combat_ui_readiness_gap_matrix.md`

## Expected Output

- `Focused S05 verification commands pass or have precisely scoped non-S05 failure notes.`
- `Docs accurately state Energy caps are enforced, Tamer/Child resources are queryable/deferred, and hidden Form Identity effects are internal-only.`

## Verification

cargo test-dev --test action_affordance_query && cargo test-dev --test resource_caps && cargo test-dev --test form_identity && cargo test-dev skills_ron
