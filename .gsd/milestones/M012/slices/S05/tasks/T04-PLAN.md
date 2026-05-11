---
estimated_steps: 12
estimated_files: 6
skills_used: []
---

# T04: Repair Form Identity runtime remediation and pipeline compile regression

Why: T03 proved that Form Identity follow-ups are being scheduled, but the runtime execution/application path is still broken for same-entity self-only effects and canonical DORUgamon behavior. The verification gate also found a compile regression in `src/combat/turn_system/pipeline.rs`, so final verification cannot even run until the scoped bindings/events issue is corrected.

Do:
1. Restore `src/combat/turn_system/pipeline.rs` compilation by keeping `events`, attacker/defender KO/stun bindings, and any derived early-return checks within valid scopes, or by restructuring the action step so event collection and validation happen before component query tuples are dropped.
2. Finish same-entity self-target application for internal Form Identity follow-ups such as hidden `GrantEnergy` and `SelfAdvance`, ensuring the acting/follower entity can be both source and target without aliasing/query conflicts or target-shape rejection.
3. Preserve hidden-vs-user-facing semantics: `SkillImplementation::Hidden` Form Identity effects may execute only through internal follow-up scheduling and must remain absent/disabled/deferred from normal player action affordances.
4. Reconcile DORUgamon canonical cast-trigger behavior with the skill-book/tag matching introduced in T03 so its toughness follow-up targets the triggering enemy as intended, while Angemon/DORUgamon offensive/toughness follow-ups do not retarget to self.
5. Keep Energy cap behavior authoritative in all repaired paths: `GrantEnergy` must still route through `RoundEnergyTracker` when present, clamp to `Energy.max`, and emit `EnergyGained` only for actual applied gain.
6. Update or add focused assertions in `tests/form_identity.rs` and `tests/resource_caps.rs` only to pin the intended behavior; do not weaken existing canonical expectations.

Failure Modes (Q5): avoid broad retargeting of all Form Identity skills to self; avoid bypassing cap accounting in an internal follow-up fast path; avoid fixing compilation by dropping event emission or KO/stun guards.
Load Profile (Q6): remediation remains per-action/per-follow-up constant work with no ECS scans beyond existing queries.
Negative Tests (Q7): same-entity self-only energy applies, repeated same-round energy remains capped, DORUgamon/Angemon target enemy behavior remains intact, hidden skills remain non-user-facing, and compile diagnostics are clean.

Done when: `cargo test-dev --test form_identity --test resource_caps` compiles and the remaining failures, if any, are behavioral failures with clear scope rather than pipeline compile errors.

## Inputs

- `.gsd/milestones/M012/slices/S05/tasks/T03-SUMMARY.md`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `tests/form_identity.rs`
- `tests/resource_caps.rs`

## Expected Output

- `Compiling `cargo test-dev --test form_identity --test resource_caps` no longer fails with out-of-scope variables in pipeline.rs.`
- `Canonical Form Identity self-only Energy/self-advance execution works through the cap-aware runtime path.`
- `DORUgamon canonical follow-up targeting behavior is restored without exposing hidden skills to UI.`

## Verification

cargo test-dev --test form_identity --test resource_caps
