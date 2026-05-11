# S05: S05

**Goal:** Enforce per-unit Energy gain caps in the real combat action pipeline and expose resource affordances for Energy caps, Tamer Gauge/Commands, and Child Tamer Gauge boost dependencies through the shared query vocabulary so UI/CLI/tests cannot infer or hardcode resource truth.
**Demo:** After this: Energy caps are enforced in the real pipeline, and Tamer/Child command-resource dependencies are declared for UI even if full command execution is deferred.

## Must-Haves

- ## Must-Haves
- R085 primary: `GrantEnergy` effects resolved by the live Bevy action/follow-up pipeline route through `RoundEnergyTracker` and `Energy` max clamping; emitted `EnergyGained` events report the actual applied amount and do not overreport blocked/clipped gains.
- R085 primary: the active unit's `RoundEnergyTracker` is reset at turn start with the same lifecycle as `RoundFlags`, preserving the 10 secondary / 30 external per-unit cap semantics across turns.
- R085 primary: Tamer Gauge, Data Scan, Emergency Guard, Retreat, and Child Tamer Gauge boost are represented as queryable deferred/hidden resource affordances with `TamerGaugeDeferred` / `TamerCommandDeferred`, not as executable actions.
- R084 support: Energy cap and Tamer/Child resource state/reasons come from `src/combat/action_query.rs` using `ResourceKind`, `ResourceStatus`, and `LegalityReasonCode`; no UI-facing rule should be expressed as a display-string or skill-ID-specific special case.
- Existing R073 is re-proven in the real pipeline: Energy max/caps already exist, and this slice proves the normal action resolution path uses them rather than bypassing the tracker.
- ## Threat Surface
- **Abuse**: in-test or future UI callers may inject repeated `ActionIntent`/`FollowUpIntent` messages or crafted `GrantEnergy` skills to exceed per-turn Energy budgets; the engine must cap them authoritatively.
- **Data exposure**: none; this slice exposes combat resource status only, with no PII/secrets.
- **Input trust**: `ActionIntent`, RON skill data, and pure query snapshots are trusted code/test inputs today but must be treated as future UI/CLI boundary data; malformed or incomplete fixtures should fail safely or produce deferred/disabled affordances.
- ## Requirement Impact
- **Requirements touched**: R084, R085, existing validated R073.
- **Re-verify**: pure action/resource query tests, real Bevy action pipeline Energy cap tests, canonical Form Identity tests, and skills RON parsing/docs checks.
- **Decisions revisited**: D053 and D054 remain in force; this slice applies the existing choice to declare deferred mechanics queryably rather than implementing full Tamer Command execution.
- ## Proof Level
- This slice proves: integration
- Real runtime required: yes — tests must exercise Bevy `resolve_action_system` / follow-up resolution, not only pure helper functions.
- Human/UAT required: no
- ## Verification
- `cargo test-dev --test action_affordance_query` must pass with new assertions for EnergyCap, Tamer Command, and Child boost affordance details.
- `cargo test-dev --test resource_caps` must pass with a real-pipeline Energy cap regression test covering repeated `GrantEnergy` casts, Energy max clipping, and truthful `EnergyGained` event amounts.
- `cargo test-dev --test form_identity` must pass if T03 restores canonical internal Form Identity execution; if any failures remain, the summary must identify exact failing assertions and prove they are outside S05's Energy/self-target work.
- `cargo test-dev skills_ron` must pass to prove canonical RON still parses and reason codes remain valid.
- ## Observability / Diagnostics
- Runtime signals: `CombatEventKind::EnergyGained { unit_id, amount }` remains the inspectable event for actual Energy mutation; `ResourceAffordanceDetail` values expose query-side cap/deferred state.
- Inspection surfaces: deterministic integration tests can inspect `Energy`, `RoundEnergyTracker`, `CombatEvent`, and pure `ActionAffordance.resource_details` without windowed UI.
- Failure visibility: cap exhaustion should be visible as zero/no `EnergyGained` rather than an overreported requested amount; query failures should carry `EnergyCapReached`, `TamerGaugeDeferred`, or `TamerCommandDeferred`.
- Redaction constraints: none.
- ## Integration Closure
- Upstream surfaces consumed: S04 `src/combat/action_query.rs` query types/reason vocabulary; existing `src/combat/energy.rs` `Energy` and `RoundEnergyTracker`; live action pipeline in `src/combat/turn_system/pipeline.rs`; follow-up/Form Identity path in `src/combat/follow_up.rs`.
- New wiring introduced in this slice: live `GrantEnergy` effects call the cap-aware tracker; active-turn reset clears the tracker; pure query API declares Energy cap and Tamer/Child deferred resources.
- What remains before the milestone is truly usable end-to-end: S06 must use this query as engine validation for illegal injected intents, S07 must wire CLI/windowed adapters to query output, and S08/S09 will add enemy counterplay declarations/docs alignment.

## Proof Level

- This slice proves: integration

## Integration Closure

Consumes S04's pure legality/query contract and wires the Energy part into actual Bevy runtime systems. Downstream slices still need to add authoritative preflight rejection (S06) and CLI/windowed presentation (S07), but after S05 the resource facts they consume are truthful and queryable.

## Verification

- CombatEvent::EnergyGained reports actual applied Energy; pure ResourceAffordanceDetail output exposes cap/deferred reasons; tests inspect Energy/RoundEnergyTracker state and event streams directly.

## Tasks

- [x] **T01: Added pure energy-cap and deferred Tamer resource affordance queries** `est:1h 15m`
  Expected `skills_used` frontmatter for executor: `test`, `verify-before-complete`.

Why: S05 must make non-executable resource systems queryable before UI work. Add pure, DSL/query-backed resource affordance helpers rather than any UI hardcoding. The existing S04 query already has `ResourceKind::{TamerGauge,TamerCommand,EnergyCap}` and reason codes, but lacks public helpers/data for actual Energy cap budgets and Tamer/Child command declarations.

Do:
1. Extend `UnitQuerySnapshot` in `src/combat/action_query.rs` with Energy cap budget/counter fields needed to answer secondary/external Energy cap queries deterministically. Choose names that preserve meaning, e.g. `energy_secondary_gained` and `energy_external_gained`, and update existing fixtures in `tests/action_affordance_query.rs` with zero defaults.
2. Add pure query helpers in `src/combat/action_query.rs` for Energy caps and deferred Tamer/Child resources. At minimum provide queryable details for Energy cap remaining/exhausted, Tamer Gauge, Tamer Commands (`Data Scan` 20, `Emergency Guard` 50, `Retreat` 100), and Child Tamer Gauge boost. These should return `ResourceAffordanceDetail` using `ResourceKind`, `ResourceStatus`, and `LegalityReasonCode`, not display strings.
3. Add tests to `tests/action_affordance_query.rs` proving Energy cap detail is enabled when budget remains, disabled with `EnergyCapReached` when exhausted or requested exceeds remaining, and Tamer/Child declarations are deferred/hidden with `TamerGaugeDeferred` / `TamerCommandDeferred` and required costs where known.
4. Update docs only where needed so `docs/skill_legality_contract.md` and `docs/combat_ui_readiness_gap_matrix.md` describe S05's contract accurately: Tamer/Child systems are declared/deferred; Energy cap state is queryable.

Failure Modes (Q5): incomplete snapshots should default to conservative zero-used cap budgets in tests or force explicit construction; do not panic in public query helpers for missing non-critical resource state.
Load Profile (Q6): pure query is O(number of declared commands) plus existing snapshot scanning; it must not allocate unbounded data or inspect ECS.
Negative Tests (Q7): include exhausted cap, partial remaining cap, and deferred command declarations that are not enabled/executable.

Done when: the pure query surface can answer all S05 resource-affordance questions with machine-readable reason codes, and `cargo test-dev --test action_affordance_query` passes.
  - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`, `docs/skill_legality_contract.md`, `docs/combat_ui_readiness_gap_matrix.md`
  - Verify: cargo test-dev --test action_affordance_query

- [x] **T02: Wire RoundEnergyTracker into the live GrantEnergy pipeline and reset it at turn start.** `est:1h 30m`
  Expected `skills_used` frontmatter for executor: `test`, `verify-before-complete`.

Why: Existing R073 validated caps in isolation, but `src/combat/turn_system/pipeline.rs` currently bypasses `RoundEnergyTracker` and calls `Energy::gain(requested)` directly. This task proves the real runtime path enforces caps and reports actual mutation.

Do:
1. Add an `Energy` helper in `src/combat/energy.rs` that returns the actual applied amount after max clamping, e.g. `gain_capped(amount) -> i32`; keep existing `gain` behavior for compatibility or delegate it to the new helper.
2. Change the Energy query path in `resolve_action_system`, `resolve_follow_up_action_system`, and `pipeline::step_app` to fetch `Energy` together with optional `RoundEnergyTracker` for the attacker entity. For production entities with a tracker, route `GrantEnergy` through `RoundEnergyTracker::try_gain(EnergyGainSource::SecondaryAction, requested)` and then `Energy::gain_capped(actual_by_round_cap)`.
3. Emit `CombatEventKind::EnergyGained` only for the actual applied amount, or at minimum never emit a positive amount when round cap or Energy max applies zero. Do not overreport the requested amount.
4. Add `RoundEnergyTracker` to `advance_turn_system`'s query tuple and reset the active unit's tracker in the same block that resets `RoundFlags` at the start of the unit turn.
5. Add integration coverage in `tests/resource_caps.rs` using a real `App` with `resolve_action_system`, a simple implemented `GrantEnergy(15)` skill, `Energy`, and `RoundEnergyTracker`. Assert two same-round casts apply at most 10 total secondary Energy, Energy max clipping is truthful, tracker counters reflect applied cap budget, and event amounts do not exceed actual Energy gained.

Failure Modes (Q5): manual legacy fixtures may omit `RoundEnergyTracker`; decide explicitly whether to treat missing tracker as legacy uncapped compatibility or update fixtures under test. Production bootstrap already attaches the tracker, so cap-specific tests must spawn it.
Load Profile (Q6): per action this adds one component query and constant arithmetic; no shared resources beyond Bevy ECS message/event queues.
Negative Tests (Q7): repeated same-round grant past cap, actor near `Energy.max`, and tracker reset at next turn.

Done when: real Bevy action resolution enforces per-unit Energy caps and `cargo test-dev --test resource_caps` passes.
  - Files: `src/combat/energy.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/follow_up.rs`, `tests/resource_caps.rs`
  - Verify: cargo test-dev --test resource_caps

- [x] **T03: Partially wired canonical Form Identity follow-up execution through the cap-aware pipeline, but same-entity self-target application and DORUgamon trigger handling still need follow-up work.** `est:1h 30m`
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
  - Files: `src/combat/follow_up.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/resolution.rs`, `tests/form_identity.rs`, `tests/resource_caps.rs`, `docs/combat_ui_readiness_gap_matrix.md`
  - Verify: cargo test-dev --test form_identity --test resource_caps

- [x] **T04: Repair Form Identity runtime remediation and pipeline compile regression** `est:2h`
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
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/follow_up.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/mod.rs`, `tests/form_identity.rs`, `tests/resource_caps.rs`
  - Verify: cargo test-dev --test form_identity --test resource_caps

- [x] **T05: Run focused S05 contract verification and tighten docs if needed** `est:45m`
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
  - Files: `tests/action_affordance_query.rs`, `tests/resource_caps.rs`, `tests/form_identity.rs`, `docs/skill_legality_contract.md`, `docs/combat_ui_readiness_gap_matrix.md`
  - Verify: cargo test-dev --test action_affordance_query && cargo test-dev --test resource_caps && cargo test-dev --test form_identity && cargo test-dev skills_ron

## Files Likely Touched

- src/combat/action_query.rs
- tests/action_affordance_query.rs
- docs/skill_legality_contract.md
- docs/combat_ui_readiness_gap_matrix.md
- src/combat/energy.rs
- src/combat/turn_system/pipeline.rs
- src/combat/turn_system/mod.rs
- src/combat/follow_up.rs
- tests/resource_caps.rs
- src/combat/resolution.rs
- tests/form_identity.rs
