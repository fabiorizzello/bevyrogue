---
estimated_steps: 13
estimated_files: 6
skills_used: []
---

# T03: Partially wired canonical Form Identity follow-up execution through the cap-aware pipeline, but same-entity self-target application and DORUgamon trigger handling still need follow-up work.

Expected `skills_used` frontmatter for executor: `test`, `verify-before-complete`.

Why: Canonical `GrantEnergy(5)` examples are Form Identity skills that are hidden from player action affordances and use `SelfOnly`. S04 left known `form_identity` regressions because the pipeline rejects non-`Single` shapes before these internal follow-ups can execute. S05 should prove Energy cap wiring against real content without making hidden Form Identity skills user-facing.

Do:
1. Restore internal execution for Form Identity follow-ups that target the acting unit with `SelfOnly` or other self-directed modifier effects. Keep `SkillImplementation::Hidden` hidden from action/query affordances; hidden means not user-facing, not necessarily impossible for internal reactive systems.
2. Prefer a narrow internal-follow-up path: when `FollowUpOriginKind::FormIdentity` schedules a self-effect such as `GrantEnergy` or `SelfAdvance`, target the follower/source rather than an enemy. Avoid changing normal player action target semantics or making general `SelfOnly` skills externally executable before S06 validation.
3. Preserve DORUgamon/Angemon semantics. DORUgamon's toughness follow-up must still affect the triggering enemy, not self; Angemon's damage follow-up must still target the Virus enemy. Do not blindly retarget every Form Identity skill to self.
4. Update `tests/form_identity.rs` fixtures to include `RoundEnergyTracker` where Energy cap behavior is asserted, then make the canonical Form Identity suite pass under the new cap-aware pipeline. If one assertion remains outside this slice, document the exact reason in task/slice summary rather than masking it.
5. Add or extend a focused `tests/resource_caps.rs` assertion proving canonical Form Identity `GrantEnergy(5)` can trigger twice across tracker resets but cannot bypass same-round cap.
6. Update docs only if the SelfOnly/Form Identity deferred/hidden contract wording is now stale.

Failure Modes (Q5): hidden-vs-internal semantics can accidentally expose hidden skills to UI or retarget offensive follow-ups to self; tests must pin both negative cases.
Load Profile (Q6): no scaling concern beyond existing follow-up message queue; each internal follow-up is one extra action cycle.
Negative Tests (Q7): Greymon trigger must not fire from another unit's Ice hit, Form Identity once-per-round guard must still work, and DORUgamon/Angemon target behavior must not regress.

Done when: canonical Form Identity Energy/self-advance behavior works through the same cap-aware runtime path while hidden/deferred affordance semantics remain query-only and not user-facing.

## Inputs

- `src/combat/follow_up.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `tests/form_identity.rs`
- `tests/resource_caps.rs`
- `docs/combat_ui_readiness_gap_matrix.md`

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `tests/form_identity.rs`
- `tests/resource_caps.rs`
- `docs/combat_ui_readiness_gap_matrix.md`

## Verification

cargo test-dev --test form_identity --test resource_caps

## Observability Impact

Follow-up/Form Identity execution remains inspectable through `CombatEvent` streams (`OnActionDeclared`, `EnergyGained`, `TurnAdvance`, damage/toughness events) and `RoundFlags.form_identity_used`.
