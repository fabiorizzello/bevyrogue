# S04 Research: Presentation beat and RON metadata boundary

## Summary

S04 is mostly contract-hardening, not a new runtime subsystem. The current code already keeps `SkillDef.animation_sequence` and `SkillDef.qte` out of gameplay resolution: runtime combat reads `effects`, `targeting`, `implementation`, `sp_cost`, `damage_tag`, and `custom_signals`, but the scan found no gameplay reads of `animation_sequence` or `qte` outside schema/assets/tests/docs. S03 also made `OnCombatBeat` and mirrored `CombatKernelTransition::Beat` live, so S04 must be precise: canonical lifecycle beats are part of the shared combat/kernel event surface, while RON presentation metadata and future UI/CLI narration remain non-authoritative consumers/cues.

Primary requirements supported by this slice:

- **R093 / R095:** RON stays declarative data/custom-signal input; presentation fields do not become final gameplay authority.
- **R094 / R096:** seeded Patamon blueprint and generic kernel authority must not be bypassed by animation/QTE strings.
- **R095 (milestone context wording):** presentation beats, trigger metadata, floating text, UI cues, and CLI narration are non-authoritative consumers of canonical combat output.
- **R100 support:** S04 proof stays deterministic/headless and should not broaden into S06 whole-suite fixture repair.

## Skills Discovered

- Loaded required process skills: `api-design`, `design-an-interface`, `grill-me`, `observability`, and `write-docs`.
- Installed globally for later units after `npx skills find` discovery:
  - `apollographql/skills@rust-best-practices` — high install count and directly relevant to Rust test/code changes.
  - `bfollington/terma@bevy` — directly relevant to Bevy ECS/testing patterns despite lower install count.
- No separate RON-specific professional skill appeared relevant; results were generic Rust/Tauri/noise.

Skill guidance applied:

- `design-an-interface`: avoid introducing a shallow presentation-runtime API unless callers need it now; the deep interface is the existing canonical event/query/snapshot surface plus a negative metadata contract.
- `grill-me`: codebase answered the root decision: this slice should harden an existing boundary, not ask the user to choose a new mechanic model.
- `observability`: prove via durable signals that a future agent can inspect — event stream, kernel transitions, validation snapshots, and executable verifier/doc checks.
- `write-docs`: docs should name the reader/post-read action and state the invariant cold, not narrate implementation history only.
- `api-design`: no HTTP API is involved; the useful rule is caller honesty/evolvability. Do not add a consumer-facing contract that implies metadata can affect results.

## Implementation Landscape

### Data schema and RON

- `src/data/skills_ron.rs`
  - `SkillDef` has gameplay-relevant fields (`damage_tag`, `sp_cost`, `targeting`, `implementation`, `effects`, `custom_signals`) plus presentation fields:
    - `animation_sequence: Option<Vec<String>>`
    - `qte: Option<String>`
  - `custom_signals` has `#[serde(default)]` and is copied to `ResolvedAction`.
  - `animation_sequence` / `qte` are plain `Option` fields; RON omission already parses as `None`, but Rust struct literals still need `..Default::default()` or explicit fields.
  - `validate_skill_book` validates effects/targeting/implementation only; it does not interpret animation/QTE metadata.
- `assets/data/skills.ron`
  - Several ultimates contain `animation_sequence` and `qte` strings.
  - `patamon_ult` contains both a real custom signal (`Patamon(BuildHolySupportGrace(amount: 1))`) and presentation metadata. This is the best canonical contrast case for S04.

### Runtime authority path

- `src/combat/resolution.rs`
  - `resolve_action` copies only gameplay fields into `ResolvedAction`, including `custom_signals`.
  - `apply_effects` computes SP, damage, toughness, revive, ult charge, status, turn advance, etc.; it does not read `animation_sequence`, `qte`, or Patamon-specific custom-signal semantics.
- `src/combat/state.rs`
  - `ResolvedAction` contains `custom_signals` but has no presentation metadata fields. This is the simplest unit-level proof surface.
- `src/combat/blueprints/mod.rs` and `src/combat/blueprints/patamon.rs`
  - Generic router dispatches `SkillCustomSignal` wrappers to per-Digimon modules.
  - Patamon maps `BuildHolySupportGrace` to `CombatKernelTransition::HolySupport(HolySupportTransition::build_grace(amount))`.
  - No animation/QTE strings are available at this seam.
- `src/combat/turn_system/mod.rs` / `src/combat/turn_system/pipeline.rs`
  - `emit_combat_beat` emits both `OnCombatBeat { beat }` and mirrored `OnKernelTransition { transition: CombatKernelTransition::Beat(beat) }`.
  - Blueprint transitions are dispatched only after `ResolutionOutcome::succeeded`.
  - `FloatingDamage` is spawned from actual `OnDamageDealt` outcomes, not RON animation metadata.
- `src/combat/kernel.rs`
  - `CombatBeatId` is canonical lifecycle vocabulary (`Declared`, `PreApp`, `Impact`, `Damage`, `ExtraHit`, `Applied`, `Resolved`).
  - `register_combat_kernel_runtime` wires kernel resources/applier systems. This is needed in S04 runtime tests when asserting Holy Support/Twin Core snapshots remain canonical.
- Mechanic appliers (`holy_support.rs`, `twin_core.rs`, `battery_loop.rs`, `predator_loop.rs`, `precision_mind_game.rs`) mutate state from canonical `OnKernelTransition` events. Do **not** write tests claiming injected kernel events are ignored; the contract is that presentation metadata cannot create those canonical transitions.

### Existing tests and useful patterns

- `tests/patamon_blueprint_seam.rs`
  - Best helper pattern for canonical `skills.ron`, app setup with `register_combat_kernel_runtime`, event cursor/drain, and Holy Support snapshot assertions.
  - Already proves custom signals are parsed, carried, blueprint-dispatched, and observed in `HolySupportState`/snapshot.
- `tests/event_stream.rs`
  - Best pattern for asserting lifecycle beat events and mirrored kernel beat transitions.
  - Current targeted run passes: `cargo test --test patamon_blueprint_seam --test event_stream` => 8/8 tests pass.
- `tests/status_effect_apply.rs`
  - Shows one broad fixture still missing `custom_signals`; full `cargo test --no-run` remains blocked by broad fixture drift. Do not make S04 depend on whole-suite compilation unless the plan intentionally absorbs S06 fixture repair.
- `tests/ui_readiness_gap_matrix_docs.rs`
  - Currently includes missing `docs/combat_ui_readiness_gap_matrix.md` and is a known broad-suite artifact blocker. It is not required for S04 targeted proof unless the plan chooses to repair that old M012 doc gap.

### Existing docs/verifiers

- `docs/combat_authority_map.md`
  - Already states `animation_sequence` and `qte` are parsed presentation metadata and not runtime gameplay authority.
  - Requirement row for R097 says beats are live while presentation metadata remains non-authoritative.
- `docs/combat_mixed_pattern_drift_ledger.md`
  - D8 is the S04-owned drift: “Add explicit contract proof that `animation_sequence` and `qte` remain presentation metadata and do not alter combat resolution.”
  - R097 handoff warns that `OnCombatBeat` is live/synchronized, but presentation fields remain non-authoritative.
- `scripts/verify_combat_authority_audit.py`
  - Currently verifies S02/S03 claims and tracked path references. It should be extended with S04/D8 claim markers once the new test/doc exists.
- `scripts/verify_m015_failure_ledger.py`
  - Current verifier passes and still expects S03 targeted evidence plus broad no-run blocker classification. Updating it for S04 is optional unless the failure ledger is changed to require S04 evidence.

Current verifier status:

- `python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py` exits 0.
- `cargo test --no-run` remains exit 101 from known broad fixture/schema/doc blockers, including missing `custom_signals` / `animation_sequence` / `qte` fields in older `SkillDef` literals, `UnitDef` fixture drift, and the missing UI readiness gap matrix doc.

## Natural Seams for Planning

1. **Targeted headless integration proof (highest value / do first)**
   - Add `tests/presentation_metadata_boundary.rs`.
   - Reuse app setup patterns from `tests/patamon_blueprint_seam.rs` and beat assertions from `tests/event_stream.rs`.
   - Keep all `SkillDef` literals using `..Default::default()` so this test does not add to broad fixture drift.

2. **Docs/audit update**
   - Update `docs/combat_authority_map.md` to move the metadata claim from “current finding” to “S04-proven contract.”
   - Update D8 in `docs/combat_mixed_pattern_drift_ledger.md` from `safe placeholder` / S04-owned to closed/normalized by S04, with the new test as evidence.
   - Optionally add a focused `docs/presentation_metadata_boundary.md` if the planner wants a standalone reader-facing doc. If added, link it from the authority map and update the verifier so the doc cannot disappear silently.

3. **Executable verifier tightening**
   - Extend `scripts/verify_combat_authority_audit.py` with S04 markers:
     - D8 row closed/normalized.
     - `tests/presentation_metadata_boundary.rs` referenced.
     - explicit claim that `animation_sequence` and `qte` do not alter `ResolvedAction`, action query, event beat sequence, kernel transitions, or validation snapshots.
   - Optional stronger check: scan gameplay directories for forbidden `animation_sequence` / `qte` references. Be careful: future `src/ui`/CLI display reads may be legitimate presentation consumption, so scope any scan to gameplay authority paths (`src/combat`, excluding docs/tests) rather than all source.

## Recommended Test Contract

Create `tests/presentation_metadata_boundary.rs` with three focused tests:

1. **RON/custom-signal contrast test**
   - Parse canonical `assets/data/skills.ron`.
   - Assert `patamon_ult` has both `custom_signals` and presentation metadata.
   - Assert the metadata fields are distinct from the custom signal by checking `holy_breeze` or a metadata-only skill has no custom signals.
   - Purpose: RON can carry both, but only typed custom signals feed blueprint/kernel behavior.

2. **ResolvedAction/action-query equality test**
   - Build two `SkillBook`s with the same skill id/effects/targeting/resources; one has dramatic metadata (`animation_sequence: Some(["Kernel(HolySupport)", ...])`, `qte: Some("Increase damage by 999 and grant Grace")`), one has no metadata.
   - Resolve the same `ActionIntent` against each.
   - Assert the resulting `ResolvedAction` values are identical, especially damage/SP/target shape/custom signals.
   - Also assert `query_action_affordance` returns the same statuses/resource details/targets for both books.
   - Purpose: metadata cannot affect preflight legality or resolved action payload.

3. **Runtime event/snapshot equality and no synthetic kernel transition test**
   - Run the same action through two headless `App`s with `register_combat_kernel_runtime`.
   - Compare event kind sequences or key subsets:
     - same `OnCombatBeat` sequence (`Declared`, `PreApp`, `Impact`, `Damage`, `Applied`, `Resolved` for a damaging action),
     - same mirrored `CombatKernelTransition::Beat` sequence,
     - same damage / skill-cast outcomes,
     - no metadata strings appear in event debug output.
   - Assert metadata-only skill produces no `CombatKernelTransition::HolySupport` and leaves `HolySupportState.grace == 0` even if metadata text pretends to grant grace.
   - Purpose: presentation metadata cannot become kernel authority; only effects/custom signals can.

Avoid tests that directly write `OnKernelTransition` and expect no mutation. Existing architecture intentionally applies canonical kernel events to state; presentation code should not write those events.

## Recommended Implementation Order

1. Add `tests/presentation_metadata_boundary.rs` with local helpers and run only that target.
2. If the test needs shared helper cleanup, prefer local helper functions over broad test-fixture refactors; broad `SkillDef` fixture repair is S06-owned.
3. Update `docs/combat_authority_map.md` and `docs/combat_mixed_pattern_drift_ledger.md` for D8/S04 proof.
4. Extend `scripts/verify_combat_authority_audit.py` to require the new S04 proof markers.
5. Run targeted verification:
   - `cargo test --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam`
   - `python3 scripts/verify_combat_authority_audit.py`
   - `python3 scripts/verify_m015_failure_ledger.py`
6. Optionally run `cargo test --no-run` only to confirm blockers remain the already-classified S06/S05 broad blockers; do not require it green for S04.

## Verification Commands for Executors

Primary S04 completion command:

```bash
cargo test --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam \
  && python3 scripts/verify_combat_authority_audit.py \
  && python3 scripts/verify_m015_failure_ledger.py
```

Context checks already run during research:

```bash
cargo test --test patamon_blueprint_seam --test event_stream
# exit 0; event_stream 1/1, patamon_blueprint_seam 7/7

python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py
# exit 0; both verifiers passed
```

Known non-goal for this slice:

```bash
cargo test --no-run
# currently exits 101 due to known broad fixture/schema/doc blockers classified in docs/m015_failure_ledger.md
```

## Risks and Pitfalls

- **Beat wording trap:** `OnCombatBeat` / `CombatKernelTransition::Beat` are not “presentation-only.” They are canonical lifecycle/kernel signals now and can trigger shared hooks such as Twin Core. S04 should prove that presentation metadata cannot author those beats or alter their sequence.
- **Direct event injection trap:** Bevy tests can write `CombatEventKind::OnKernelTransition` directly and applier systems will mutate mechanic state. That is expected for canonical event tests; it is not a presentation metadata path.
- **Broad-suite trap:** Many old tests still fail to compile because of missing `SkillDef`/`UnitDef` fields. S04 should avoid turning into broad fixture repair unless the planner explicitly chooses to consume S06 work.
- **Schema trap:** Adding `#[serde(default)]` to `animation_sequence`/`qte` may be harmless for RON symmetry, but it does not solve Rust struct literal compilation. The better S04 proof is tests/docs, not relying on serde defaults.
- **Over-interface trap:** A `PresentationMetadata` runtime API is probably premature. If future UI needs to display QTE/animation hints, add a read-only presenter DTO later; do not add a path from metadata into action resolution/kernel transitions now.

## Sources Read / Evidence

- `src/data/skills_ron.rs` — schema, validation, `SkillDef` presentation fields.
- `assets/data/skills.ron` — canonical metadata and Patamon custom-signal contrast.
- `src/combat/resolution.rs` — `ResolvedAction` construction and effect application ignore presentation metadata.
- `src/combat/state.rs` — `ResolvedAction` carries custom signals, not animation/QTE fields.
- `src/combat/blueprints/mod.rs`, `src/combat/blueprints/patamon.rs` — typed custom-signal to kernel transition seam.
- `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs` — lifecycle beat emission and post-success blueprint dispatch.
- `src/combat/kernel.rs` — `CombatBeatId`, registry, runtime registration.
- `src/combat/holy_support.rs`, `src/combat/twin_core.rs`, `src/combat/battery_loop.rs`, `src/combat/predator_loop.rs`, `src/combat/precision_mind_game.rs` — canonical kernel-event appliers.
- `src/combat/observability.rs` — validation snapshot contents; no presentation metadata.
- `tests/patamon_blueprint_seam.rs`, `tests/event_stream.rs`, `tests/ui_readiness_gap_matrix_docs.rs`, `tests/boundary_contract.rs`, `tests/status_effect_apply.rs` — existing proof/helper/blocker landscape.
- `docs/combat_authority_map.md`, `docs/combat_mixed_pattern_drift_ledger.md`, `docs/m015_failure_ledger.md` — current audit and drift owner state.
- `scripts/verify_combat_authority_audit.py`, `scripts/verify_m015_failure_ledger.py` — executable doc/audit gates.
