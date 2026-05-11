# S05 Research — Resource affordances and Energy cap wiring

## Summary

S05 supports **R085** directly and advances **R084** indirectly. The concrete slice deliverable is two-part:

1. **Real Energy-cap enforcement:** `RoundEnergyTracker` exists and is spawned by normal bootstrap, but the live action pipeline currently bypasses it and calls `Energy::gain(...)` directly for `GrantEnergy` effects.
2. **Queryable deferred resource affordances:** S04 already introduced `ResourceKind::{TamerGauge,TamerCommand,EnergyCap}` and reason codes (`TamerGaugeDeferred`, `TamerCommandDeferred`, `EnergyCapReached`), but there is no purpose-built query for Tamer Commands/Child boost declarations yet; the current details only appear opportunistically in action-resource detail plumbing.

Baseline focused verification:

- `cargo test-dev --test action_affordance_query --test resource_caps` passes.
- `cargo test-dev --test form_identity` currently fails 8/10, matching the S04 note that Form Identity regressions pre-exist this slice. The immediate cause relevant to S05 is that Form Identity `GrantEnergy` skills are canonical RON `Hidden(reason: UnimplementedEffect)` with `shape: SelfOnly`, while `pipeline::step_declaration` rejects non-`Single` target shapes before any `GrantEnergy` reaches the Energy pipeline.

## Requirements Targeting

- **R085 primary:** Energy caps must be enforced in the real pipeline; Tamer Gauge/Commands and Child Tamer Gauge boost must be represented as queryable deferred/hidden affordances so UI cannot lie.
- **R084 support:** Any resource state/reason surfaced to UI/CLI/tests must come from the shared query vocabulary, not UI hardcoding.
- Existing validated **R073** says Energy max/caps already exist, but S05 should prove the real pipeline uses them, because `src/combat/turn_system/pipeline.rs` currently bypasses the tracker.

## Skill Discovery

Installed skills do not include a Rust/Bevy-specific skill. Relevant external skill search:

- `mindrally/skills@rust` — 249 installs — general Rust guidance. Install command: `npx skills add mindrally/skills@rust`.
- `bfollington/terma@bevy` — 117 installs — Bevy-specific. Install command: `npx skills add bfollington/terma@bevy`.
- `sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs — Bevy ECS patterns. Install command: `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert`.

No install is required for this slice; codebase patterns are already clear.

Loaded-skill rules that should inform execution:

- `verify-before-complete`: do not mark S05 complete without fresh command output in the completion message. At minimum run focused tests for Energy/query work and note the known `form_identity` state if unresolved.
- `test`: this slice is verification-heavy; prefer deterministic integration tests around the existing Bevy app/update loop rather than relying only on unit tests for helper methods.

## Implementation Landscape

### Energy model

- `src/combat/energy.rs`
  - `Energy { current, max }` component; `gain(amount)` only clamps at `max`.
  - `EnergyGainSource::{SecondaryAction, External}`.
  - `RoundEnergyTracker { secondary_gained, external_gained }` component; `try_gain(source, amount)` enforces +10 secondary / +30 external caps.
  - Existing unit tests cover the tracker in isolation only.

- `src/combat/bootstrap.rs`
  - Normal `spawn_unit_from_def` attaches both `Energy::default()` and `RoundEnergyTracker::default()` to units.
  - Many older integration tests manually spawn units and only attach `Energy`, so any pipeline query should handle fixtures deliberately (either update fixtures or provide a clear fallback behavior).

- `src/combat/turn_system/mod.rs`
  - `advance_turn_system` resets `RoundFlags` (`break_sealed`, `form_identity_used`) at start of a unit turn.
  - It does **not** currently query or reset `RoundEnergyTracker`; add it to the query tuple and reset the active unit’s tracker in the same reset block.
  - `ResolveActorsQuery` currently does not include Energy or RoundEnergyTracker; `resolve_action_system` passes a separate `Query<&mut Energy>` into the pipeline.

- `src/combat/turn_system/pipeline.rs`
  - Live bypass at lines around the Energy grant block:
    - if `outcome.succeeded && inflight.action.energy_grant > 0`, it does `energy.gain(inflight.action.energy_grant)` and emits `EnergyGained { amount: inflight.action.energy_grant }`.
  - This is the main S05 seam: change this to route through `RoundEnergyTracker::try_gain(EnergyGainSource::SecondaryAction, requested)` and then clamp through `Energy::gain(actual)`.
  - Important: if the actor is near `Energy.max`, `Energy::gain` does not report actual applied amount. Consider adding an `Energy::try_gain(amount) -> i32` or `Energy::gain_capped(amount) -> i32` helper so the event amount is truthful.

- `src/combat/follow_up.rs`
  - `resolve_follow_up_action_system` mirrors `resolve_action_system` and passes `mut energy_q: Query<&mut Energy>` into `step_app`; this must be updated with the same query signature.
  - `form_identity_listener_system` schedules Form Identity as a `FollowUpIntent`, so Form Identity Energy gains flow through this path once shape/implementation semantics allow them to execute.

### Form Identity interaction risk

Canonical Form Identity skills are in `assets/data/skills.ron`:

- `greymon_form_identity`, `garurumon_form_identity`, `kabuterimon_form_identity`: `Hidden(reason: UnimplementedEffect)`, `shape: SelfOnly`, `effects: [GrantEnergy(5)]`.
- `kyubimon_form_identity`: `Hidden`, `SelfOnly`, `effects: [SelfAdvance(20)]`.
- `dorugamon_form_identity`: `Hidden`, `SelfOnly`, `effects: [ToughnessHit(10)]`.
- `angemon_form_identity`: `Hidden`, `Single Enemy`, `effects: [Damage(15)]`.

`resolve_action` does not check `SkillImplementation`, but `step_declaration` rejects all non-`Single` `TargetShape`s via `target_shape_rejection_reason`. Because Form Identity energy skills are now `SelfOnly`, they never reach `step_app`; this explains the baseline `form_identity` failures for Energy and self-advance. S05 planners should decide whether restoring Form Identity execution is in scope for Energy cap wiring. It likely is, because otherwise the only canonical `GrantEnergy` path cannot prove real pipeline cap enforcement.

Possible low-scope approach:

- Keep Form Identity skills out of player action affordances by **not adding them to `UnitSkills.skills`**; they already live only in `FormIdentityKit` config.
- Allow internal follow-up execution for implemented modifier-only `SelfOnly` effects by targeting `source == target` and treating `SelfOnly` as executable only when source and target match. This avoids UI exposure while preserving query truthfulness.
- Be careful with `dorugamon_form_identity`: a `SelfOnly` `ToughnessHit` would hit the actor if executed literally, but historical comments say it is a separate toughness-hit follow-up against the event target. This is a data/semantics mismatch not caused by Energy caps; do not accidentally make it damage ally toughness.

### Query/resource affordance model

- `src/combat/action_query.rs`
  - `UnitQuerySnapshot` has `energy: i32` but no Energy cap/tracker fields.
  - `ResourceKind` already includes `TamerGauge`, `TamerCommand`, `ChargedTelegraph`, `EnemyTrait`, and `EnergyCap`.
  - `resource_detail_status(ResourceKind::EnergyCap, current, required)` maps shortfall to `EnergyCapReached`, but no current code constructs EnergyCap details for actual `GrantEnergy` effects/cap state.
  - `build_resource_details` only emits SP and Ultimate details for implemented skills; hidden/deferred implementation returns hidden/deferred SP/Ultimate details only.

Natural S05 additions:

- Extend `UnitQuerySnapshot` with round Energy cap counters or remaining budgets, e.g. `energy_secondary_gained`, `energy_external_gained`, or `energy_secondary_remaining`, `energy_external_remaining`. Backward compatibility will require fixture defaults or test updates.
- Add small pure helpers such as:
  - `query_energy_cap_affordance(actor, source, requested) -> ResourceAffordanceDetail`
  - `query_tamer_command_affordances(snapshot/actor) -> Vec<ResourceAffordanceDetail>` returning deferred Tamer Gauge/Command details.
- If adding fields to `UnitQuerySnapshot`, update `tests/action_affordance_query.rs` fixture `unit(...)` immediately.

### Tamer/Child command-resource declarations

Design docs define:

- `docs/combat_design.md`: Tamer Gauge max 100, charges with team actions; Tamer Commands are Data Scan (20), Emergency Guard (50), Retreat (100).
- Child has a Tamer Gauge boost when Basic attacking a revealed-intent enemy, but the gauge itself is not executable today.

Current code has no `TamerGauge` component/resource and no command intent type. Therefore S05 should **declare**, not implement:

- Tamer Gauge: `ResourceStatus::Deferred { reason: TamerGaugeDeferred }`.
- Tamer Commands: `ResourceStatus::Deferred { reason: TamerCommandDeferred }`, ideally with `required: Some(20/50/100)` and `current: None` or a clear absent value.
- Child Tamer Gauge boost: deferred/hidden declaration with `TamerGaugeDeferred` until intent reveal + gauge exists.

This can live in `action_query.rs` as a separate query surface rather than overloading skill action affordance, because Tamer Commands are not `SkillDef`s and should not be faked as skills.

## Natural Seams / Suggested Task Boundaries

1. **Pure resource affordance declarations**
   - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`, `docs/skill_legality_contract.md` if docs need detail.
   - Add/query deferred Tamer Gauge, Tamer Commands, Child boost resource details.
   - Add Energy cap affordance helper and tests for `EnergyCapReached` when remaining cap is 0 or requested > remaining.
   - This can be done without touching Bevy pipeline.

2. **Energy cap helper and real pipeline wiring**
   - Files: `src/combat/energy.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/follow_up.rs`, tests in `tests/resource_caps.rs` or new `tests/energy_cap_pipeline.rs`.
   - Add actual-applied helper on `Energy` and use `RoundEnergyTracker` for `GrantEnergy` pipeline grants.
   - Reset per-unit tracker in `advance_turn_system`.
   - Update query signatures in both direct and follow-up action resolution paths.

3. **Canonical Form Identity Energy path recovery (if required for pipeline proof)**
   - Files: `src/combat/follow_up.rs`, `src/combat/turn_system/pipeline.rs` or `src/combat/resolution.rs`, maybe `assets/data/skills.ron` depending chosen semantics.
   - Restore the canonical `GrantEnergy(5)` path so Energy cap tests can use real skills rather than artificial fixtures.
   - Treat this carefully because it intersects S04/S06 implementation-status semantics. Hidden should mean “not user-facing”, not necessarily “engine can never execute internal reactive skill”.

4. **Docs/tests alignment**
   - Files: `docs/combat_ui_readiness_gap_matrix.md`, `docs/skill_legality_contract.md`, `tests/ui_readiness_gap_matrix_docs.rs`, `tests/skill_legality_contract_docs.rs` if wording changes.
   - Ensure docs say Energy caps are now wired and Tamer/Child command resources are deferred declarations, not executable systems.

## Risks and Constraints

- **Borrow/query churn:** adding `RoundEnergyTracker` to `ResolveActorsQuery` will force many destructuring updates. Lower-risk alternative is to change the separate Energy query to `Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>` and update only `resolve_action_system`, `resolve_follow_up_action_system`, and `pipeline::step_app` signatures.
- **Event truthfulness:** `EnergyGained { amount }` should report actual applied Energy, not requested Energy. If cap or max clamps to 0, either emit no `EnergyGained` or add a separate failure/cap event. Current event enum has no cap-specific event.
- **Fixture consistency:** manual spawns in tests often omit `RoundEnergyTracker`; add it where cap behavior is under test. Do not accidentally weaken production enforcement because old fixtures are incomplete.
- **Hidden vs internal execution:** S04 uses `SkillImplementation::Hidden` to hide UI affordances. Form Identity internal bonuses may still need to execute. Avoid treating Hidden as universally unexecutable in engine until S06 decides authoritative validation semantics.
- **DORUgamon data mismatch:** `dorugamon_form_identity` is `SelfOnly` but has `ToughnessHit(10)`, which only makes sense against the event target. Do not use self-target normalization blindly for all Form Identity skills unless this mismatch is resolved/deferred explicitly.

## Recommended First Proof

Build the smallest red/green proof before broad refactors:

1. Add an integration test that creates a real Bevy app, manually spawns an actor with `Energy` + `RoundEnergyTracker`, uses a simple implemented `GrantEnergy(15)` skill, casts it twice in the same round, and asserts Energy increases by `10` total for `EnergyGainSource::SecondaryAction` and the second event does not overreport.
2. Add a pure query test that an actor with exhausted secondary Energy cap returns an `EnergyCap` resource detail with `ResourceStatus::Disabled { reason: EnergyCapReached }`.
3. Add a pure query test for Tamer command declarations: Data Scan / Emergency Guard / Retreat are `Deferred` with `TamerGaugeDeferred`/`TamerCommandDeferred`, not enabled.

Only after this proof should executors decide how much of the pre-existing `form_identity` regression to fix in S05.

## Verification Commands

Focused S05 verification candidates:

```bash
cargo test-dev --test action_affordance_query
cargo test-dev --test resource_caps
cargo test-dev --test form_identity
cargo test-dev energy::
```

If Form Identity recovery is included, require `cargo test-dev --test form_identity` to pass or document any remaining non-S05 failures precisely.

Before closing the slice, run at least:

```bash
cargo test-dev --test action_affordance_query --test resource_caps --test form_identity
cargo test-dev skills_ron
```

Full `cargo test-dev` currently has known `form_identity` failures from S04 baseline, so completion should either fix them or explicitly distinguish any remaining baseline failures from S05 changes. Windowed compile is not required until S07, but `cargo check --features "dev windowed"` was green in S04 and can be used as a regression guard if query types are public API.
