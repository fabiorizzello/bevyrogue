# S03: Skill DSL targeting and legality metadata

**Goal:** Make skill targeting and implementation legality explicit in the SkillDef DSL and canonical skills.ron catalog so S04 can query action/target affordances without inferring intent from effect shapes or skill IDs. After this slice, canonical `assets/data/skills.ron` contains first-class targeting/legalities metadata, invalid or contradictory sample skills fail loudly, and the existing resolution path preserves target shape from `SkillDef.targeting.shape` rather than scanning `Effect::Damage`.
**Demo:** After this: canonical `skills.ron` contains targeting/legalities metadata, parses cleanly, and invalid sample skills fail loudly.

## Must-Haves

- `src/data/skills_ron.rs` defines first-class targeting metadata on `SkillDef` covering target shape, target side, target life/HP state, self-targeting policy, and implementation status/reason.
- The schema uses stable contract reason codes from `docs/skill_legality_contract.md` for deferred/hidden/unsupported metadata; no skill-ID-specific legality registry or UI rule table is introduced.
- `assets/data/skills.ron` is migrated so all 72 canonical skills declare `targeting` and `implementation` metadata.
- Canonical Row/AoE-like skills are explicitly marked `Deferred(reason: UnimplementedTargetShape)` while Row execution remains unsupported; the mixed-effect `angemon_ult` is explicitly not presented as a normal implemented action.
- Revive-like skills declare ally/KO/single-target semantics in data, without relying on `Effect::Revive` inference.
- Form Identity / follow-up support skills keep current execution compatibility while declaring truthful current selected-target semantics; later S05/S06 may refine resource/source affordances.
- Semantic validation rejects contradictory declarations, including damage effect shape mismatches, revive skills that do not declare KO targeting, implemented non-single shapes, and deferred metadata without a stable reason.
- `resolve_action` reads `target_shape` from `skill.targeting.shape` so no-damage skills and intentionally mismatched fixtures prove metadata is the source of truth.
- Requirement impact: R084 and R085 are advanced by explicit data/schema ownership, canonical metadata, and loud unsupported/deferred declarations. Re-verify canonical skill parsing/validation, target-shape truthfulness, revive semantics, and all Rust fixtures that construct `SkillDef` directly.
- Threat surface: malformed or modded RON skill data must not trick future UI/query code into showing false actions, wrong-side targets, or unsupported AoE as executable. Missing/unknown metadata must fail deserialization, and well-typed contradictions must fail semantic validation.

## Proof Level

- This slice proves: Contract-level and integration-regression proof. Tests validate the DSL schema, canonical asset data, semantic validation errors, and the resolution path that hands target-shape metadata to S02's rejection logic. No human/UAT or real windowed runtime is required for S03 unless exported type changes unexpectedly break feature-gated UI compilation.

## Integration Closure

Upstream consumed: S02 `ResolvedAction.target_shape` and `target_shape_rejection_reason`, plus `docs/skill_legality_contract.md` stable reason vocabulary. New wiring: `SkillDef` owns targeting/implementation metadata, canonical RON carries it, and `resolve_action` copies `targeting.shape` into `ResolvedAction`. Remaining milestone work: S04 exposes the pure legality/affordance query API; S05 handles resource affordances and Energy caps; S06 makes engine validation consume the query; S07 wires CLI/windowed consumers to the query.

## Verification

- Validation errors should name the offending skill id and stable reason/category so future agents can diagnose bad RON without stepping through Bevy asset loading. Runtime observability continues through S02's `OnActionFailed { reason: "UnimplementedTargetShape:<Shape>" }` unsupported-shape failure signal. There are no secret or PII redaction concerns.

## Tasks

- [x] **T01: Introduce SkillDef targeting metadata and migrate Rust fixtures** `est:2h`
  ---
estimated_steps: 5
estimated_files: 21
skills_used:
  - tdd
  - verify-before-complete
---

Add the explicit DSL types that S03 needs, attach them to `SkillDef`, and update every Rust-side `SkillDef { ... }` construction site so the codebase compiles against the new required metadata before the canonical RON migration begins.

Steps:
1. In `src/data/skills_ron.rs`, add serde-friendly metadata types: `SkillTargeting`, `TargetSide`, `TargetLife`, `SelfTargetRule`, `SkillImplementation`, and a stable `LegalityReasonCode` enum with at least the reason codes needed by this slice (`UnimplementedTargetShape`, `UnimplementedEffect`, and target-side/life codes used by validation tests). Use `#[serde(deny_unknown_fields)]` on `SkillDef` and metadata structs.
2. Add required `targeting: SkillTargeting` and `implementation: SkillImplementation` fields to `SkillDef`; derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize` consistently with existing types.
3. Add small constructor/helper functions in test modules where useful, but do not introduce a sidecar legality registry. Every fixture must declare metadata that matches its intended current behavior.
4. Update all Rust fixture literals currently found by `rg -l "SkillDef \\{" src tests` so damage skills declare `shape: Single`, `side: Enemy`, `life: Alive`, `self_rule: Forbid`, `implementation: Implemented`; revive fixtures declare `side: Ally`, `life: Ko`; existing Row/AoE truthfulness fixtures declare Row metadata with deferred implementation unless the test is specifically proving mismatch validation later.
5. Keep existing behavior assertions intact; this task is a schema/compile migration and should not change runtime validation semantics yet.

Must-haves:
- `SkillDef` metadata is required in Rust, not optional/defaulted away.
- Stable reason codes are machine identifiers, not display strings.
- No UI/CLI/windowed skill-ID-specific legality table is introduced.
- Fixture migration preserves current test intent for damage, revive, SP, toughness, status, follow-up, and ultimate tests.

Failure Modes:
- **Malformed fixture metadata**: compile may pass but later validation may fail; use clear helper names and comments for revive vs offensive vs Row-deferred fixtures.
- **Serde compatibility drift**: unknown fields must fail after `deny_unknown_fields`; do not add broad catch-all variants.
- **Mechanical churn misses**: `cargo check --tests` is the guard for all direct `SkillDef` literals.

Load Profile:
- Shared resources: none beyond compile/test time.
- Per-operation cost: serde structs are static data; no runtime allocation beyond loading existing skill data.
- 10x breakpoint: a larger catalog increases validation iteration linearly; no complex lookup is introduced in this task.

Negative Tests:
- Planned in T02 after canonical RON is migrated; T01 should not claim validation coverage beyond type-level required fields.

Verification:
- `cargo check --tests`
- `rg "SkillDef \\{" src tests` shows all remaining literals explicitly include `targeting:` and `implementation:` or are inside helper constructors that do.

Inputs:
- `src/data/skills_ron.rs` — existing SkillDef, Effect, TargetShape, and serde tests.
- `src/combat/resolution_tests.rs` — direct SkillDef fixtures for resolution behavior.
- `src/combat/follow_up_tests.rs` — direct SkillDef fixtures for follow-up behavior.
- `src/combat/turn_system/tests.rs` — direct SkillDef fixtures for turn-system behavior.
- `tests/resource_caps.rs` — direct SkillDef fixtures for resource tests.
- `tests/sp_economy.rs` — direct SkillDef fixtures for SP tests.
- `tests/revive_semantics.rs` — revive fixture semantics.
- `tests/patamon_revive.rs` — revive fixture semantics.
- `tests/target_shape_truthfulness.rs` — Row/AllEnemies shape fixtures from S02.
- `tests/toughness_enemy_only.rs` — toughness fixtures.
- `tests/status_effect_apply.rs` — status fixtures.
- `tests/status_effect_integration.rs` — status fixtures.
- `tests/boundary_contract.rs` — boundary contract fixtures.
- `tests/combat_coherence.rs` — coherence fixtures.
- `tests/damage_breakdown_log.rs` — damage fixture.
- `tests/encounter_e2e.rs` — encounter fixture.
- `tests/event_stream.rs` — event stream fixture.
- `tests/status_accuracy.rs` — status accuracy fixture.
- `tests/toughness_categories.rs` — toughness category fixture.
- `tests/ultimate_meter.rs` — ultimate fixture.

Expected Output:
- `src/data/skills_ron.rs` — new metadata schema attached to SkillDef.
- `src/combat/resolution_tests.rs` — fixtures migrated to explicit metadata.
- `src/combat/follow_up_tests.rs` — fixtures migrated to explicit metadata.
- `src/combat/turn_system/tests.rs` — fixtures migrated to explicit metadata.
- `tests/resource_caps.rs` — fixtures migrated to explicit metadata.
- `tests/sp_economy.rs` — fixtures migrated to explicit metadata.
- `tests/revive_semantics.rs` — fixtures migrated to explicit metadata.
- `tests/patamon_revive.rs` — fixtures migrated to explicit metadata.
- `tests/target_shape_truthfulness.rs` — fixtures migrated to explicit metadata.
- `tests/toughness_enemy_only.rs` — fixtures migrated to explicit metadata.
- `tests/status_effect_apply.rs` — fixtures migrated to explicit metadata.
- `tests/status_effect_integration.rs` — fixtures migrated to explicit metadata.
- `tests/boundary_contract.rs` — fixtures migrated to explicit metadata.
- `tests/combat_coherence.rs` — fixtures migrated to explicit metadata.
- `tests/damage_breakdown_log.rs` — fixtures migrated to explicit metadata.
- `tests/encounter_e2e.rs` — fixtures migrated to explicit metadata.
- `tests/event_stream.rs` — fixtures migrated to explicit metadata.
- `tests/status_accuracy.rs` — fixtures migrated to explicit metadata.
- `tests/toughness_categories.rs` — fixtures migrated to explicit metadata.
- `tests/ultimate_meter.rs` — fixtures migrated to explicit metadata.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution_tests.rs`, `src/combat/follow_up_tests.rs`, `src/combat/turn_system/tests.rs`, `tests/resource_caps.rs`, `tests/sp_economy.rs`, `tests/revive_semantics.rs`, `tests/patamon_revive.rs`, `tests/target_shape_truthfulness.rs`, `tests/toughness_enemy_only.rs`, `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/boundary_contract.rs`, `tests/combat_coherence.rs`, `tests/damage_breakdown_log.rs`, `tests/encounter_e2e.rs`, `tests/event_stream.rs`, `tests/status_accuracy.rs`, `tests/toughness_categories.rs`, `tests/ultimate_meter.rs`
  - Verify: cargo check --tests && rg "SkillDef \\{" src tests

- [x] **T02: Migrate canonical skills.ron and add loud validation** `est:2h`
  ---
estimated_steps: 5
estimated_files: 2
skills_used:
  - tdd
  - verify-before-complete
---

Migrate the canonical skill catalog to the new metadata schema and add semantic validation tests that make false or unsupported affordances fail loudly before S04 consumes the data.

Steps:
1. Update all 72 entries in `assets/data/skills.ron` with `targeting` and `implementation` fields. Use consistent RON formatting near existing fields so future content authors can copy patterns.
2. Classify canonical skills without skill-ID-specific runtime rules: normal single-target damage/follow-up/ultimate skills are `Implemented` enemy/alive/single/forbid-self; revive skills are `Implemented` ally/KO/single; Row skills are `Deferred(reason: UnimplementedTargetShape)`; mixed-effect target semantics such as `angemon_ult` must not be marked as a normal implemented action while semantics are unresolved.
3. In `src/data/skills_ron.rs`, implement `validate_skill_book(&SkillBook) -> Result<(), SkillBookValidationError>` or equivalent structured error API. It must reject contradictory metadata with the offending skill id and stable reason/category.
4. Extend in-module tests to cover metadata round-trip, canonical parse+validate, missing required metadata fails parse, unknown metadata field fails parse, `Damage(target: Row)` with `targeting.shape: Single` fails validation, `Revive` with non-KO targeting fails validation, and `Implemented` plus non-single shape fails while current execution supports only Single.
5. Ensure tests assert stable machine reason codes such as `UnimplementedTargetShape`, not localized display copy.

Must-haves:
- Canonical `assets/data/skills.ron` parses and validates cleanly.
- Invalid sample skills fail loudly at either deserialization or semantic validation, depending on whether they are structurally malformed or contradictory.
- Row and mixed-effect unsupported semantics are explicit in data as deferred/disabled metadata, not silently inferred from effects.
- `angemon_ult` remains visible to future consumers only according to truthful implementation metadata; do not normalize it with a skill-id exception in code.

Failure Modes:
- **Malformed RON**: serde parse error should point to the data entry; keep changes small and run the focused parser test often.
- **Contradictory but well-typed data**: semantic validation must return an error rather than letting future UI infer false affordances.
- **Reason mismatch**: reason enum names must stay aligned with `docs/skill_legality_contract.md` vocabulary.

Load Profile:
- Shared resources: one in-memory skill catalog.
- Per-operation cost: validation is O(number of skills × effects per skill), acceptable for a 72-skill catalog.
- 10x breakpoint: catalog validation remains linear; if content grows 10x, error reporting should still name the first offending skill clearly.

Negative Tests:
- **Malformed inputs**: missing `targeting` and unknown targeting field RON snippets must fail deserialization.
- **Error paths**: well-typed contradictory snippets must return validation errors with skill id and reason/category.
- **Boundary conditions**: non-damage revive and form-identity skills must validate from explicit metadata rather than damage-effect defaults.

Verification:
- `cargo test-dev skills_ron`
- `grep -c "targeting:" assets/data/skills.ron` returns 72.
- `grep -c "implementation:" assets/data/skills.ron` returns 72.

Inputs:
- `src/data/skills_ron.rs` — schema from T01 and existing canonical tests.
- `assets/data/skills.ron` — canonical skill catalog to migrate.
- `docs/skill_legality_contract.md` — stable reason/status vocabulary to mirror in code tests.

Expected Output:
- `assets/data/skills.ron` — all canonical skills include explicit targeting and implementation metadata.
- `src/data/skills_ron.rs` — validation API and focused parser/semantic validation tests.
  - Files: `assets/data/skills.ron`, `src/data/skills_ron.rs`, `docs/skill_legality_contract.md`
  - Verify: cargo test-dev skills_ron && test "$(grep -c 'targeting:' assets/data/skills.ron)" -eq 72 && test "$(grep -c 'implementation:' assets/data/skills.ron)" -eq 72

- [x] **T03: Source resolved target shape from DSL metadata** `est:1h`
  ---
estimated_steps: 4
estimated_files: 4
skills_used:
  - tdd
  - verify-before-complete
---

Wire the existing resolution path to consume `SkillDef.targeting.shape` as the authoritative target-shape source and update regression tests so S02's unsupported-shape rejection will now be driven by S03 metadata instead of effect-shape inference.

Steps:
1. In `src/combat/resolution.rs`, change `resolve_action` so `ResolvedAction.target_shape` is copied from `skill.targeting.shape`. Remove or narrow the old `skill_target_shape(&skill.effects)` helper so no-damage skills no longer default to Single through effect inference.
2. Update `src/combat/resolution_tests.rs` to prove the new source of truth. Include a fixture where `Effect::Damage { target: Single }` but `targeting.shape: Row` resolves to Row, and a no-damage revive fixture that resolves to the explicit Single shape from metadata.
3. Update `tests/target_shape_truthfulness.rs` only as needed so Row/AllEnemies rejection still proves pre-mutation failure with `UnimplementedTargetShape:<Shape>` after the metadata migration.
4. Run revive and target-shape regressions to ensure existing execution behavior is preserved while shape truth now comes from metadata.

Must-haves:
- No code path infers selected target shape solely from `Effect::Damage` for `ResolvedAction`.
- S02 behavior remains true: non-single shapes are rejected before lifecycle mutation with stable `UnimplementedTargetShape:<Shape>` reasons.
- Revive and other no-damage skills use explicit metadata, not a fallback default, for selected shape.
- This task does not implement full side/life/self legality enforcement; that remains for S04/S06.

Failure Modes:
- **Metadata/effect mismatch**: validation from T02 should catch canonical contradictions; resolution should trust validated metadata and tests should make that boundary explicit.
- **Unsupported shape regression**: Row/AllEnemies must still fail before mutation, not execute as single-target.
- **Revive regression**: existing revive behavior must remain green; S03 only changes metadata source, not execution legality.

Load Profile:
- Shared resources: existing skill book lookup in resolution.
- Per-operation cost: reading a copied enum from `SkillDef` is trivial and cheaper than scanning effects.
- 10x breakpoint: no new scaling concern; resolution still performs the existing skill lookup.

Negative Tests:
- **Boundary conditions**: mismatched effect shape vs metadata shape fixture proves metadata wins; Row/AllEnemies tests prove unsupported shapes do not mutate state.
- **Error paths**: missing skill behavior remains `None` as before and is not widened in this task.

Verification:
- `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive`
- `cargo test-dev skills_ron`

Inputs:
- `src/combat/resolution.rs` — existing `resolve_action`, shape helper, and rejection helper.
- `src/combat/resolution_tests.rs` — direct resolution unit tests.
- `tests/target_shape_truthfulness.rs` — S02 integration tests for unsupported-shape rejection.
- `src/data/skills_ron.rs` — metadata schema and validation from T01/T02.

Expected Output:
- `src/combat/resolution.rs` — resolution copies target shape from `SkillDef.targeting.shape`.
- `src/combat/resolution_tests.rs` — tests prove metadata, not effect inference, drives `ResolvedAction.target_shape`.
- `tests/target_shape_truthfulness.rs` — regression tests updated for metadata fixtures if needed.
- `src/data/skills_ron.rs` — only touched if minor helper visibility adjustments are needed.
  - Files: `src/combat/resolution.rs`, `src/combat/resolution_tests.rs`, `tests/target_shape_truthfulness.rs`, `src/data/skills_ron.rs`
  - Verify: cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive && cargo test-dev skills_ron

- [x] **T04: Run final S03 contract regressions and align documentation references** `est:45m`
  ---
estimated_steps: 4
estimated_files: 3
skills_used:
  - test
  - verify-before-complete
---

Close the slice by running the contract-level regression set and making any small documentation/test-name alignment needed so future S04 executors know that S03's output is the `SkillDef` metadata contract, not a completed query API.

Steps:
1. Run the planned S03 verification commands and fix any failures that are caused by stale imports, outdated test fixture metadata, or contract vocabulary drift.
2. If validation introduces new public reason-code names or metadata terminology, update `docs/skill_legality_contract.md` only to keep names aligned; do not expand S03 into the S04 query API design.
3. Ensure no CLI/windowed/UI file gained per-skill legality hardcoding while migrating data. Use ripgrep checks for obvious skill-id legality branches if touched files raise suspicion.
4. Leave a concise comment in `src/data/skills_ron.rs` or tests explaining that side/life/self metadata is declared in S03 and enforced/queryable in later S04/S06 slices.

Must-haves:
- Final focused regression commands pass freshly in this workspace.
- Contract vocabulary in docs/tests/code is aligned on stable reason names.
- No consumer-specific legality table or per-skill UI/CLI workaround was introduced.
- The slice stops at metadata plus resolution shape propagation; no partial pure-query API is added here.

Failure Modes:
- **Vocabulary drift**: doc contract test should catch missing reason/status names; update code or docs deliberately, not casually.
- **Scope creep**: if implementation starts adding preflight query API or engine side/life enforcement, stop and defer that work to S04/S06 unless needed to fix compile/test regressions.
- **Windowed surprise**: S03 should not normally require windowed compile, but if exported type changes break windowed imports, run `cargo check --features "dev windowed"` and fix compile-only fallout.

Load Profile:
- Shared resources: test runner only.
- Per-operation cost: deterministic Rust tests; no runtime load concerns.
- 10x breakpoint: none for this validation/documentation task.

Negative Tests:
- **Malformed inputs**: covered by `cargo test-dev skills_ron` from T02.
- **Boundary conditions**: covered by target-shape and revive regressions from T03.
- **Error paths**: covered by unsupported shape rejection tests and validation errors.

Verification:
- `cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs --test revive_semantics --test patamon_revive`
- `cargo test-dev skills_ron`
- Optional if exported type changes touch feature-gated UI imports: `cargo check --features "dev windowed"`

Inputs:
- `docs/skill_legality_contract.md` — contract vocabulary baseline.
- `src/data/skills_ron.rs` — final schema/validation/tests.
- `src/combat/resolution.rs` — final metadata-to-resolution wiring.
- `tests/skill_legality_contract_docs.rs` — doc vocabulary regression.
- `tests/target_shape_truthfulness.rs` — unsupported-shape regression.

Expected Output:
- `docs/skill_legality_contract.md` — only updated if needed for exact reason-code alignment.
- `src/data/skills_ron.rs` — final comments/import fixes if needed.
- `src/combat/resolution.rs` — final import/helper cleanup if needed.
- `tests/skill_legality_contract_docs.rs` — only updated if names intentionally change.
- `tests/target_shape_truthfulness.rs` — final fixture cleanup if needed.
  - Files: `docs/skill_legality_contract.md`, `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `tests/skill_legality_contract_docs.rs`, `tests/target_shape_truthfulness.rs`
  - Verify: cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs --test revive_semantics --test patamon_revive && cargo test-dev skills_ron

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/resolution_tests.rs
- src/combat/follow_up_tests.rs
- src/combat/turn_system/tests.rs
- tests/resource_caps.rs
- tests/sp_economy.rs
- tests/revive_semantics.rs
- tests/patamon_revive.rs
- tests/target_shape_truthfulness.rs
- tests/toughness_enemy_only.rs
- tests/status_effect_apply.rs
- tests/status_effect_integration.rs
- tests/boundary_contract.rs
- tests/combat_coherence.rs
- tests/damage_breakdown_log.rs
- tests/encounter_e2e.rs
- tests/event_stream.rs
- tests/status_accuracy.rs
- tests/toughness_categories.rs
- tests/ultimate_meter.rs
- assets/data/skills.ron
- docs/skill_legality_contract.md
- src/combat/resolution.rs
- tests/skill_legality_contract_docs.rs
