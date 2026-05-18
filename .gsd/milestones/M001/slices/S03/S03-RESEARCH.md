# S03 Research: Validator L with adapter based checks

## Summary

S03 should add a generic animation validator inside the existing `src/animation` boundary, not under `src/combat`, because S01/S02 deliberately established animation as a cohesive public module. The validator needs to join `AnimGraph` + `Clip` + adapter-provided catalogs and return typed diagnostics for blocking failures while allowing advisory warnings for reachability/cancel-style checks.

The highest-risk proof is the cross-asset seam: validate the real Agumon `anim_graph.ron` against real `clip.ron` and a catalog derived outside animation core from `SkillBook`/project data. The core validator should not import `src/data` or Digimon-specific modules directly. Tests can build the adapter catalog in `tests/anim_fsm_validation.rs` using existing `aggregate_skill_book()` or direct test fixtures.

Active requirements owned by this slice:
- R004: invalid animation assets at boot fail fast with typed diagnostics.
- R005: cross-asset validation uses explicit adapter seams, not direct animation-core coupling to Digimon/gameplay internals.
- R008: all validation remains headless-first; no `windowed`, winit, wgpu, or UI dependency.

## Skills Discovered

- Required skill invocations were requested, but the `Skill` tool is not exposed in this harness. I read the installed skill docs directly instead.
- `decompose-into-slices`: reinforced vertical tasking and first-risk proof; planner should make validator API + one broken fixture the first task, not a horizontal pile of every fixture.
- `design-an-interface`: relevant because the adapter/validator seam is load-bearing. I applied the rule to compare distinct interface shapes below.
- `grill-me`: relevant open decisions should be resolved from code, not asked of user. The repo already answers module placement and boot/readiness patterns.
- `write-docs`: final research is written for a fresh planner/executor; no long-lived docs need updating in S03 unless a public validation contract doc is desired.
- `bevy`: installed and directly relevant. Key rules: use plugin structure, keep systems ordered, and prefer `cargo check`/headless tests for fast verification. Bevy 0.18 is in use locally; existing code already uses `MessageReader<AssetEvent<T>>` and `RonAssetPlugin` patterns.
- No external skill install was needed: core technologies are Rust, Bevy, serde/RON, thiserror, all already represented by installed/project patterns. I did not run `npx skills find` because installed skills cover the directly relevant technologies.

## Existing Implementation Landscape

### Animation module

Files:
- `src/animation/mod.rs`: public module seam exporting `anim_graph`, `clip`, and `plugin`.
- `src/animation/anim_graph.rs`: closed typed graph schema. Important types: `AnimGraph`, `ClipId`, `NodeId`, `ParamKey`, `StatusId`, `ParticleId`, `FrameRange`, `AnimNode`, `AnimEdge`, `TransitionTarget`, `Command`, `ParamRef`, `Predicate`.
- `src/animation/clip.rs`: strict typed `Clip` schema with `ClipMeta`, `FrameSize`, `ClipRange`. `ClipRange::len()` uses inclusive semantics.
- `src/animation/plugin.rs`: Bevy asset loader for graph and clip. Current readiness is split: `AnimationGraphLoadState.ready` and `AnimationClipLoadState.ready` independently flip only after load/modify event plus typed `Assets<T>` readability.

Current assets:
- `assets/digimon/agumon/anim_graph.ron`: graph references `clip: "skill"`, nodes in frames 60-77, commands `SpawnParticle(name: "baby_flame")` and `EmitDamage(status: Some("Heated"), mul: Literal(18), duration: Literal(3))`.
- `assets/digimon/agumon/clip.ron`: ranges include `skill: 60..=77`, `attack`, `idle`, etc.

Existing graph/clip tests:
- `tests/anim_graph_parse.rs`: parses closed enum vocabulary and rejects unknown command/predicate/target variants.
- `tests/anim_graph_asset.rs`: proves real Agumon graph loads as a typed Bevy asset and readiness does not flip early.
- `tests/clip_parse.rs`: parses clip and rejects unknown/malformed fields.
- `tests/clip_geometry_parity.rs`: proves Agumon clip exact geometry/range parity with source atlas JSON.
- `tests/clip_asset.rs`: proves real Agumon clip loads as typed Bevy asset and readiness does not flip early.

### Data/cross-asset sources

Files:
- `src/data/mod.rs`: owns data plugin and compile-time aggregate helpers. Useful test/API helpers: `aggregate_skill_book()`, `try_aggregate_skill_book()`, and `aggregate_skill_book_ron_text()`.
- `src/data/skills_ron/types.rs`: defines `SkillBook(pub Vec<SkillDef>)`, `SkillDef`, `SkillCustomSignal`, and target/effect metadata. `SkillDef.id` is `SkillId` and Agumon has `baby_flame` with damage amount 18 plus custom signals.
- `src/data/skills_ron/validation.rs`: existing example of typed validation error using `thiserror::Error`, category/reason/detail fields, and a `validate_skill_book(&SkillBook) -> Result<(), SkillBookValidationError>` pure-ish contract.
- `src/data/error.rs`: current `DataError` has `RonParse`, `Validation(SkillBookValidationError)`, and `TimelineCompile` variants. S03 can add an animation validation variant if boot integration uses DataPlugin-style errors.

Important constraint: `src/animation` should not import `src/data` or `src/combat` internals. Tests or an adapter module outside animation core can translate a `SkillBook` into generic catalogs.

## Recommended Interface Shape

Use a deep, small pure validator API in `src/animation/validation.rs`:

```rust
pub fn validate_anim_graph(
    graph: &AnimGraph,
    clip: &Clip,
    catalogs: &AnimationValidationCatalogs,
) -> AnimationValidationReport;

pub struct AnimationValidationCatalogs {
    pub params: BTreeSet<ParamKey>,
    pub statuses: BTreeSet<StatusId>,
    pub particles: BTreeSet<ParticleId>,
    pub skills: BTreeSet<SkillIdRef>, // optional/future if graph starts using SkillIdRef
}

pub struct AnimationValidationReport {
    pub diagnostics: Vec<AnimationValidationDiagnostic>,
}

impl AnimationValidationReport {
    pub fn is_ok(&self) -> bool;
    pub fn blocking_errors(&self) -> impl Iterator<Item = &AnimationValidationDiagnostic>;
}
```

Use `Result<(), AnimationValidationError>` only as a convenience wrapper, e.g. `validate_anim_graph_blocking(...) -> Result<(), AnimationValidationError>`, where the error carries all blocking diagnostics. This avoids losing warnings.

### Interface alternatives considered

1. **Single pure function with catalog bag (recommended).** Callers pass `&AnimGraph`, `&Clip`, and `&AnimationValidationCatalogs`. This keeps animation core generic, tests simple, and boot integration explicit.
2. **Trait object adapter (`dyn AnimationValidationAdapter`).** More flexible for runtime catalogs but adds unnecessary indirection now and makes tests heavier. Better later if live adapters need incremental reload semantics.
3. **DataPlugin-owned validator.** Easiest to wire against `SkillBook`, but violates R005/D002 by placing the real validation logic near data/gameplay and encouraging direct coupling.
4. **Bevy system-only validator.** Fits ECS style but is shallow: hard to test individual diagnostics and not needed for deterministic headless contract checks. A Bevy system can call the pure function later.

## Validator Checks to Implement

Blocking diagnostics should be stable enum variants, not string matching. Suggested `AnimationValidationCheck` variants:

- `EntryMissing`: `graph.entry` must exist in `graph.nodes`.
- `DanglingTransitionFrom`: every `edge.from` exists in `graph.nodes`.
- `DanglingTransitionTo`: every `TransitionTarget::Node(to)` exists in `graph.nodes`.
- `ExitUnreachable`: from `graph.entry`, some path must reach `TransitionTarget::Exit`.
- `DuplicatePriority`: within the same `from` node, explicit priorities should be unique. Treat `None` as not participating unless design chooses otherwise.
- `NodeFrameRangeInvalid`: `FrameRange(start,end)` must have `start <= end`.
- `NodeFrameRangeOutOfClip`: every node frame must sit inside the graph's referenced clip range.
- `ClipRangeMissing`: `graph.clip.0` must exist in `clip.ranges`.
- `ParamMissing`: each non-literal `ParamRef::{Static, Snapshot, BlueprintState}` must appear in `catalogs.params`.
- `StatusMissing`: `StatusId` references in commands and predicates should appear in `catalogs.statuses`.
- `ParticleMissing`: `ParticleId` in `SpawnParticle` should appear in `catalogs.particles` if the catalog is non-empty or if particles are in scope for S03.

Advisory/warning diagnostics:
- `UnreachableNode`: nodes unreachable from entry. M022 says reachability is warning-only.
- `CancelCoverageMissing`: M022 says cancel coverage is warning-only. Current schema does not encode an explicit cancel coverage requirement beyond `Predicate::UserInput(UserInputFilter::Cancel)`, so this may be a weak advisory check or deferred until runtime semantics exist.

StartQte `headless_default` was in old M022 checks, but the current schema makes it a required field in `Command::StartQte`; parse already fails if missing. S03 can document this as schema-enforced rather than re-checking it.

## Adapter Catalog Strategy

Keep the core type generic and create adapter construction outside `src/animation`.

Minimal S03 catalog for current Agumon:
- `params`: include known parameter keys used by tests. Real Agumon currently uses only `Literal`, so a broken fixture can prove missing `Static("atk_mul")` fails while a valid fixture passes with a provided key set.
- `statuses`: include canonical statuses used by real data and graph. Existing skill validation has `CANON_STATUS_IDS` private lowercase strings, while graph uses `"Heated"`. Do not make animation core depend on that private const; create a test/adapter catalog with `StatusId("Heated")`, `StatusId("Blessed")`, etc. If executor wants real integration, derive from `SkillBook` effects/custom signals outside animation.
- `particles`: include `ParticleId("baby_flame")` for Agumon.
- `skills`: if used, derive from `SkillBook.0.iter().map(|s| SkillIdRef(s.id.0.clone()))` outside animation.

Possible file for project-specific adapter:
- `src/animation/adapters.rs` is **not** recommended if it imports `crate::data`.
- Better: keep only generic catalog structs in `src/animation/validation.rs`, and put `SkillBook` conversion in a data-owned or test-owned place such as `src/data/animation_validation.rs` if boot integration needs it. This keeps dependency direction: data knows about animation catalogs; animation does not know data.

## Boot-Time Integration Options

For S03 contract tests, pure validation is enough. For R004 boot-time semantics, planner should add a minimal integration path that can fail typed diagnostics once graph+clip assets are loaded.

Practical staged integration:
1. Add pure validator and tests first.
2. Add `AnimationValidationLoadState` or extend plugin with a validation resource only after graph+clip ready and assets are readable.
3. For tests, avoid full asset reload complexity unless needed: parse RON with `include_str!` and call pure validator for deterministic typed diagnostics.
4. If wiring to Bevy, system ordering should run validation after both `track_animation_graph_loads` and `track_animation_clip_loads` update readiness. It should not mark validation ready from asset events alone.

`DataError` integration:
- If the executor wires validation through `DataPlugin`/boot error surfaces, add `DataError::AnimationValidation(AnimationValidationError)` and preserve diagnostics in Display.
- If the executor keeps it in `AnimationAssetPlugin`, use a separate `AnimationValidationError`/state resource and avoid forcing data-layer ownership.

## Natural Seams / Proposed Work Units

1. **Validator types + pure structural checks**
   - Files: `src/animation/validation.rs`, `src/animation/mod.rs`, `tests/anim_fsm_validation.rs`.
   - Checks: entry exists, dangling from/to, exit reachable, unreachable warning, duplicate priority.
   - First proof: one valid tiny graph passes; broken missing-entry fixture returns `EntryMissing` typed diagnostic.

2. **Clip/frame checks**
   - Files: same validator/test files.
   - Checks: clip range exists, frame start/end valid, node frames inside `clip.ranges[graph.clip.0]`.
   - Use `Clip` literals in tests; include a broken out-of-bounds fixture.

3. **Catalog-backed checks**
   - Files: same validator/test files, optionally a data/test adapter helper.
   - Checks: missing param/status/particle diagnostics.
   - Must prove adapter seam by passing a catalog explicitly; no imports from `crate::data` inside animation validator.

4. **Real Agumon integration test**
   - Files: `tests/anim_fsm_validation.rs`.
   - Parse `assets/digimon/agumon/anim_graph.ron` and `assets/digimon/agumon/clip.ron` via `include_str!` or `std::fs` from manifest dir.
   - Build a real-enough catalog with `baby_flame`, `Heated`, and any known params/particles. If using real data, derive skill IDs from `aggregate_skill_book()` outside animation.
   - Assert no blocking diagnostics.

5. **Optional boot validation state**
   - Files: `src/animation/plugin.rs`, possibly `src/data/error.rs` only if DataPlugin owns failure conversion.
   - Be cautious: current plugin supports multiple graph paths and clip paths as independent vectors but has no mapping from a graph asset path to a clip asset beyond `graph.clip` string and `Clip.ranges`. With one Agumon pair this is simple; with multiple clips in S04 the planner must define how graph assets choose which loaded `Clip` asset instance to validate against.

## First Proof Recommendation

Implement the pure validator report with only these checks first: `EntryMissing`, `DanglingTransitionTo`, `ClipRangeMissing`, `NodeFrameRangeOutOfClip`, and `ParamMissing`. Then write `tests/anim_fsm_validation.rs` with one valid graph+clip+catalog and one broken fixture for each. This retires the main uncertainty: typed diagnostics over structural + cross-asset data without coupling animation core to data internals.

After that, add the remaining M022 checks (`ExitUnreachable`, `DuplicatePriority`, warnings). Do not start with Bevy boot wiring; it is lower risk once the pure contract is stable.

## Verification Plan

Targeted commands:
- `cargo test --test anim_fsm_validation`
- `cargo test --test anim_graph_parse`
- `cargo test --test clip_parse`
- `cargo test --test anim_graph_asset`
- `cargo test --test clip_asset`
- `cargo test`

Optional compile checks if plugin/DataError wiring changes:
- `cargo check`
- `cargo check --features windowed` only if code touches feature boundaries; S03 should remain headless.

Test assertions should inspect typed enum variants directly rather than matching display strings. Display strings should still include enough context for boot failures: graph id/file if available, node id, check, and detail.

## Risks / Watch-Outs

- `ClipId` currently names a range (`"skill"`), not a clip asset file. With only one clip asset this works. S04 roster-ready multi-assets may need an asset-level mapping; do not overfit S03 to Agumon path names.
- `FrameRange` has no validation method; `ClipRange::len()` can underflow if `start > end`. Validator should check invalid ranges before calling inclusive len semantics.
- `StartQte.headless_default` is schema-required today; a validator test for missing it would be a parse test, not a validation diagnostic.
- `SkillIdRef` exists but is unused in the current `Command` schema. Do not invent skill-reference checks unless a real graph field references skills.
- Existing `SkillBook` status canonicalization is private/lowercase while animation uses `StatusId("Heated")`. Avoid binding animation validator to private skill validation constants.
- `AnimationAssetPlugin` registers both `AnimGraph` and `Clip` as `RonAssetPlugin::<T>::new(&["ron"])`. This has worked in S01/S02 tests, but keep asset tests after changes to catch loader ambiguity.
- Bevy skill guidance: avoid cargo clean; use targeted cargo tests/checks for iteration.

## Sources / Evidence

- `src/animation/anim_graph.rs`: current closed graph schema and command/param/status reference surfaces.
- `src/animation/clip.rs`: current clip/range schema and inclusive range semantics.
- `src/animation/plugin.rs`: current typed asset loading/readiness pattern.
- `assets/digimon/agumon/anim_graph.ron` and `assets/digimon/agumon/clip.ron`: real valid fixture pair S03 should validate.
- `src/data/mod.rs`: aggregate skill book helpers and data plugin readiness patterns.
- `src/data/skills_ron/types.rs`: `SkillBook`, `SkillDef`, custom signals, and skill ids for adapter construction.
- `src/data/skills_ron/validation.rs`: local typed validation error style.
- `docs/M022/slices/S03/S03-PLAN.md`: original Validator §L checklist, adapted to current `src/animation` boundary.
- Memories: MEM009/D002 adapter validation boundary, MEM011 strict boot/resilient reload policy, MEM043 readiness only after typed asset readability, MEM040 strict schema patterns.
