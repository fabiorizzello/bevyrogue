# M001: Animation asset pipeline foundation

**Gathered:** 2026-05-18
**Status:** Ready for planning

## Project Description

M001 ports and adapts the existing `docs/M022/` plan into the active GSD milestone. The milestone builds the asset-pipeline foundation for animation: typed `clip.ron` and `anim_graph.ron` Bevy assets, boot-time validation, adapter-based cross-asset checks, roster-ready asset coverage, and a real `windowed` hot-reload proof.

M022 is the base for defining what needs to be implemented, but it is not a set of iron rules. It came from `MILESTONE_PORTFOLIO`; this milestone preserves the intent while adapting architecture to the current repo and the clarified requirement that the animation motore stays generic and decoupled from Digimon-specific content.

## Why This Milestone

Later visual/runtime work needs a trustworthy animation asset contract. If the schema for frame geometry or animation FSM orchestration is wrong, it should be discovered here through headless asset loading and validation, not halfway through a future render/runtime milestone. This milestone establishes the module seam and validation behavior that later animation runtime work can build on.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `cargo test` and see typed animation asset contract tests pass for `clip.ron`, `anim_graph.ron`, validator checks, and adapter-based cross-asset validation.
- Run `cargo check` and `cargo check --features windowed` successfully, preserving the headless-first project rule.
- Run `cargo run --features windowed`, edit animation asset files, and verify live hot reload without panic or corrupted world state.
- Inspect the animation module and see generic engine code separated from Digimon-specific assets/adapters.

### Entry point / environment

- Entry point: `cargo test`; `cargo check`; `cargo check --features windowed`; manual `cargo run --features windowed` for hot-reload UAT.
- Environment: local dev, headless-first. The hot-reload proof requires `windowed`.
- Live dependencies involved: Bevy `AssetServer` and file-watch hot reload; `bevy_common_assets` RON asset loading; no external services.

## Completion Class

- Contract complete means: `Clip` and `AnimGraph` schemas load typed RON assets, invalid schema cases fail with typed diagnostics, and Agumon clip geometry is proven against source atlas data.
- Integration complete means: `AnimGraph` and `Clip` validate together, cross-asset references are checked through explicit adapters into real project data, and non-Agumon assets exercise the same generic path.
- Operational complete means: a real `cargo run --features windowed` hot-reload demo has been run and documented, including no crash or world-state corruption on reload.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Typed `anim_graph.ron` and `clip.ron` assets load through one cohesive animation module boundary.
- Required validator §L checks reject broken fixtures with typed diagnostics rather than runtime surprises.
- Cross-asset checks use adapter-provided catalogs instead of direct Digimon/gameplay coupling in the animation core.
- Agumon proves the full path with real data, while non-Agumon support validates through the same generic architecture instead of one-off hardcoding.
- Manual `windowed` hot reload works in the real app environment.

## Architectural Decisions

### One cohesive animation module

**Decision:** Schema, loading, validation, orchestration, and future runtime/player behavior should live behind one cohesive animation module boundary.

**Rationale:** The user wants the whole animation FSM area in one conceptual module. Keeping this seam cohesive makes later runtime work easier to reason about while still allowing internal submodules such as `clip`, `anim_graph`, `validate`, `plugin`, and `runtime`.

**Alternatives Considered:**
- Split asset schemas into `src/data/` and runtime elsewhere — rejected as the primary design because it weakens the animation-FSM module boundary.

### Generic motore separated from Digimon specificity

**Decision:** The animation engine must stay generic; Digimon-specific behavior belongs in assets, data catalogs, and adapters, not in core animation logic.

**Rationale:** The user explicitly warned to disaccoppia motore da specificità Digimon. This matters now because early Agumon-first work could otherwise bake Agumon assumptions into the generic engine.

**Alternatives Considered:**
- Hardcode Agumon-first shortcuts to match M022 literally — rejected because M022 is a portfolio-generated scope seed, not a rigid implementation law.

### Adapter-based cross-asset validation

**Decision:** Use a quasi-pure validator that accepts explicit validation catalogs/adapters for project-specific data instead of importing all gameplay/data internals directly into animation core.

**Rationale:** This keeps validation real while preserving a clean module seam. The validator can still prove param refs, clip refs, and related cross-asset constraints, but coupling to existing game data remains explicit and localized.

**Alternatives Considered:**
- Directly couple validation to `SkillBook` and gameplay structures — simpler initially but more fragile as runtime and data evolve.
- Fully pure validation with no project data — cleaner but too weak for the M022/M001 contract.

### Strict on boot, resilient on reload

**Decision:** Bad assets at boot are typed blocking errors; bad hot-reload edits keep the last valid version, log a clear error, and do not crash or corrupt world state.

**Rationale:** This follows M022's validator-first intent while preserving a usable authoring loop during live editing.

**Alternatives Considered:**
- Warning-only invalid boot assets — rejected because it defers schema bugs to runtime.
- Crash on invalid hot reload — rejected because it makes the authoring loop brittle.

## Error Handling Strategy

Invalid animation assets at boot fail fast with typed diagnostics. Required structural and cross-asset errors are blocking. Advisory checks such as reachability or cancel coverage may remain warnings unless implementation evidence shows they should be promoted. During hot reload, invalid edits do not replace the last valid asset state; the system logs the error clearly and avoids panic or world-state corruption.

## Risks and Unknowns

- Exact module placement — the animation boundary must fit the current repo without scattering schema and runtime across unrelated modules.
- Adapter shape — cross-asset validation needs enough project data to be meaningful without coupling animation core to all gameplay internals.
- Non-Agumon architecture — the milestone must avoid careless stubs that pass tests while hiding bad roster assumptions.
- Hot reload proof — Bevy supports watching for changes, but the final operational proof still requires a real `windowed` run.

## Existing Codebase / Prior Art

- `docs/M022/M022-CONTEXT.md` — source planning context for this milestone.
- `docs/M022/M022-ROADMAP.md` — source roadmap shape, adapted into M001.
- `docs/M022/slices/S01/S01-PLAN.md` — source AnimGraph schema and loader plan.
- `docs/M022/slices/S02/S02-PLAN.md` — source Clip schema and lossless conversion plan.
- `docs/M022/slices/S03/S03-PLAN.md` — source validator §L plan.
- `docs/M022/slices/S04/S04-PLAN.md` — source hot-reload and non-Agumon plan.
- `src/data/mod.rs` — established repo pattern for `RonAssetPlugin`, `AssetServer`, `LoadedWithDependencies`, and typed data readiness.
- `src/data/error.rs` — existing typed error surface to integrate or convert animation validation errors at plugin boundaries.
- `assets/data/digimon/agumon/skills.ron` — likely source for adapter-backed cross-asset parameter validation.

## Relevant Requirements

- R001 — Establishes the generic animation module seam.
- R002 — Advances typed `anim_graph.ron` loading.
- R003 — Advances typed `clip.ron` loading and geometry parity.
- R004 — Advances strict boot-time validation.
- R005 — Advances adapter-based cross-asset validation.
- R006 — Advances manual `windowed` hot-reload proof.
- R007 — Advances roster-ready architecture.
- R008 — Preserves headless-first verification constraints.

## Scope

### In Scope

- Port and adapt the M022 asset pipeline into M001.
- Define typed animation asset schemas for `anim_graph.ron` and `clip.ron`.
- Register/load those assets through Bevy asset patterns already used by the repo.
- Prove Agumon as the first full real-data path.
- Make non-Agumon support architecturally real through the same generic loading/validation path.
- Implement validator §L-style required checks with typed diagnostics.
- Use explicit adapters for cross-asset validation.
- Prove manual hot reload in `windowed`.

### Out of Scope / Non-Goals

- Runtime animation player and `tick_fsm` execution.
- Command to gameplay/kernel runtime translation.
- Full production-complete authored animation content for every Digimon.
- Digimon-specific behavior inside the core animation engine.
- Treating M022 as a literal implementation mandate when current repo structure calls for adaptation.

## Technical Constraints

- Headless-first: tests and validation must run without `windowed`.
- No winit, wgpu, or egui dependency outside feature-gated `windowed` paths.
- Deterministic validation: no wall-clock or unseeded randomness.
- Closed schema vocabularies for animation graph concepts where applicable.
- Generic animation engine code must not encode Agumon or other Digimon-specific rules.

## Integration Points

- Bevy `AssetServer` — asset loading and hot reload.
- `bevy_common_assets::ron::RonAssetPlugin` — typed RON asset loading pattern.
- Existing data module patterns — readiness and `LoadedWithDependencies` lifecycle.
- Data/gameplay adapters — provide catalogs for cross-asset validation without contaminating animation core.
- `windowed` runtime — real hot-reload UAT only.

## Testing Requirements

- Contract tests for typed `AnimGraph` parsing and schema rejection.
- Contract tests for typed `Clip` parsing and Agumon geometry parity.
- Validator tests with broken fixtures for every required blocking check.
- Integration tests proving adapter-backed cross-asset validation against real project data.
- Headless `cargo check` and `cargo test` verification.
- `cargo check --features windowed` verification.
- Manual `cargo run --features windowed` hot-reload UAT documented in the slice summary.

## Acceptance Criteria

- S01: A cohesive animation module can load an Agumon `anim_graph.ron` as a typed asset and reject out-of-vocabulary schema values with typed errors.
- S02: A typed `clip.ron` asset loads and Agumon geometry parity against source atlas data is proven.
- S03: Validator §L required checks pass for valid assets and fail broken fixtures with typed diagnostics; cross-asset checks use explicit adapters.
- S04: Non-Agumon assets validate through the same generic path; manual `windowed` hot reload is proven without crash or world-state corruption.

## Open Questions

- Exact source file/module names for the new animation boundary should be chosen during implementation after reading nearby current modules.
- The final shape of validation adapters should be driven by the smallest real catalog needed for M001 checks.
