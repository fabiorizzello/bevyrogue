# S05 Research — Built-in extension fns + RON → CompiledTimeline compiler

## Summary

Targeted research. S05 is the first bridge from the hand-built M021 timeline framework to real skill data. The framework exists (`src/combat/api/{timeline,runner,registry,skill_ctx}.rs`) and is validated by integration tests, but there is no production compiler path from `assets/data/skills.ron` into `CompiledTimeline`, no built-in extension registration, and most non-damage `Intent` variants are still no-op in the new `intent_applier`.

The safest first proof is **one implemented single-target skill (`petit_thunder`) compiled from RON into a `CompiledTimeline`, executed by `BeatRunner`, and producing the same core intent/event surface as the legacy path**. `tohakken` is more complex and currently not present in live `assets/data/skills.ron` under that id; the current Renamon ult is `renamon_ult` / `Fox Drive` with single-target damage+toughness plus precision custom signals. The design doc for `tohakken` describes AoE enemy damage + AoE enemy delay + team `Blessed`, so implementing it will also require applier support beyond current DealDamage.

## Requirements owned / supported

No root `REQUIREMENTS.md` was preloaded. From M021 success criteria, S05 directly supports:

- Boot-time strict validation for timeline refs: typo in RON must fail before runtime.
- Built-in extension fns registered by the kernel before active-skill migration.
- RON v2 skill graph loaded/compiled into `CompiledTimeline`.
- S05 roadmap gate: `Tohakken + Petit Thunder via CompiledTimeline; typo→errore boot`.

## Skill discovery

Relevant installed skills from the prompt:

- `bevy` — directly relevant for resource/plugin lifecycle, `App::finish`, systems, and headless-first constraints.
- `rust-best-practices` — relevant for typed compiler errors, ownership, and small module boundaries.
- `rust-testing` / `tdd` — relevant for compiler fixtures and integration tests.

External skill discovery performed for RON/serde: `npx skills find "serde ron"`.
Promising but not installed:

- `npx skills add udapy/rust-agentic-skills@ron-specialist` — 17 installs; directly relevant to RON schema/compiler work.
- `npx skills add existential-birds/beagle@serde-code-review` — 20 installs; broader serde review, less targeted than the RON specialist.

## Implementation Landscape

### Existing framework pieces

- `src/combat/api/timeline.rs`
  - Defines `TimelineLibrary`, `CompiledTimeline`, `Beat`, `BeatKind`, `BeatEdge`, `Presentation`, `BeatEvent`, `SelectorCtx`, `CueCtx`, and `validate_timeline_refs`.
  - `TimelineLibrary` is already a Bevy `Resource`, but it only holds `Vec<CompiledTimeline>` and is currently empty unless tests construct timelines manually.
  - `validate_timeline_refs` checks hook, selector, and predicate ids, recursing into `BeatKind::Loop` bodies. It does **not** currently validate formula or cue ids because formulas are not structurally referenced and presentation cue ids are data-only strings.

- `src/combat/api/registry.rs`
  - Defines generic `Registry<E: ExtPoint>` and `ExtRegistries` with seven axes.
  - Hook, selector, predicate, and cue signatures are usable.
  - `FormulaExt`, `TickExt`, and `AiUtilityExt` still have placeholder `fn()` signatures. S05 only needs formula if the compiler models damage amount through formulas; the lower-risk path is hook ids that encode fixed operations from compiled skill params.

- `src/combat/api/runner.rs` and `runner_common.rs`
  - `BeatRunner` can execute timelines, including single-level loops and Windowed cue stalls.
  - `runner_common::fire_beat` resolves selectors only for `BeatKind::Impact`, invokes hooks, and folds newly enqueued `DealDamage` targets into `cast_hit_set`.
  - Selector functions receive `SelectorCtx { caster, primary_target, state: &() }`, with no `World`; real all-enemy/all-ally selectors cannot be implemented as selectors yet unless `SelectorCtx` is extended.
  - Hooks and predicates receive `SkillCtx`, which has read-only `world`, `caster`, `primary_target`, `cast_id`, `mode`, `registries`, `cast_hit_set`, and `enqueue`.

- `src/combat/plugin.rs`
  - `CombatPlugin::build` initializes `ExtRegistries`, `TimelineLibrary`, `IntentQueue`, `SignalBus`, etc.
  - `CombatPlugin::finish` validates every timeline in `TimelineLibrary` against `ExtRegistries` and panics with axis/site details on dangling references.
  - There is no built-in `register_builtin_extensions` call yet; currently tests register local ids manually.

### Existing data pieces

- `src/data/skills_ron.rs`
  - Current schema is still v1: `SkillDef { id, name, damage_tag, sp_cost, targeting, implementation, effects, custom_signals, ... }` with `enum Effect`.
  - `validate_skill_book` validates semantic consistency for current `Effect`/`TargetShape` fields.
  - There is no `timeline` field or compiler helper.
  - `SkillBook` loads through Bevy assets; `DataPlugin` validates on `AssetEvent::LoadedWithDependencies`, i.e. after plugin finish, not during `App::finish`.

- `assets/data/skills.ron`
  - `petit_thunder` exists: `Damage(16, Single)`, `ToughnessHit(8)`, `ApplyStatus(Paralyzed, 1)`, custom signal `tentomon/build_static_charge` amount 1.
  - `tohakken` does not exist in live skills. Current Renamon ult is `SkillId("renamon_ult")`, name `Fox Drive`, target `Single`, effects `Damage(48), ToughnessHit(30)`, custom precision signals.
  - Future docs define `tohakken` as Renamon ult: AoE Holy damage to all enemies, `DelayTurn(all enemies, 30%)`, and `Blessed` to all allies for 2 turns.

### Existing execution pieces

- `src/combat/api/applier.rs`
  - New kernel `intent_applier` currently implements only `DealDamage`, `BlueprintSignal`, and `SetBlueprintState`.
  - `ApplyStatus`, `BreakToughness`, `DelayTurn`, `ApplyBuff`, `AdvanceTurn`, `HealHp`, resource intents, etc. log `unimplemented intent variant` and no-op.
  - Therefore `petit_thunder` can only be partially true via CompiledTimeline until `ApplyStatus`/`BreakToughness` are implemented, and `tohakken` cannot be faithful until `DelayTurn` and `ApplyBuff` are implemented.

- Legacy execution still lives in `src/combat/resolution.rs` and `src/combat/turn_system/pipeline.rs`.
  - Useful pure helpers: `resolve_targets`, `select_bounce_hop`, `compute_hop_damage`, extraction helpers from `Effect`.
  - But direct reuse inside new hooks must preserve M021 invariant: hooks enqueue `Intent`; state mutation only in `intent_applier`.

## Key constraints and surprises

1. **`tohakken` id mismatch is a real planning risk.** The roadmap says Tohakken, but live assets have `renamon_ult`/`Fox Drive`. A task must decide whether S05 renames/adds `tohakken`, aliases `renamon_ult` to Tohakken semantics, or implements the future design under existing id. This should be explicit in the plan.

2. **Boot-time validation has two possible meanings.** `CombatPlugin::finish` can validate timelines already inserted into `TimelineLibrary`, but asset-loaded `SkillBook` arrives later through `DataPlugin`. A RON typo can still fail on load via `validate_skill_book_on_load`; it will not naturally be caught by `CombatPlugin::finish` unless timelines are compiled synchronously before finish or a startup/load system populates and validates `TimelineLibrary` immediately after asset load.

3. **Selectors are not world-aware.** `SelectorCtx` cannot read teams/hp/alive state. For S05, either:
   - keep selectors simple (`primary`) and implement multi-target inside hooks using `ctx.world`, or
   - extend `SelectorCtx` to include `&World` and possibly caster/primary team data.
   The first path is lower-risk for S05 but less architecturally pure; the second is better for S06 migration.

4. **Formula axis is not ready.** The design calls for formula ids, but `FormulaExt` is still `fn()`. S05 can defer formula sophistication by compiling concrete hook ids with embedded skill params stored in generated timelines/side tables, but S06 will need a real `FormulaCtx` if damage/heal math becomes registry-driven.

5. **Intent applier coverage blocks truthful end-to-end tests.** A compiler can enqueue `ApplyStatus`, `BreakToughness`, `DelayTurn`, and `ApplyBuff` today, but only `DealDamage` mutates/emits. Planner should include applier tasks for the exact effects used by Petit Thunder and Tohakken, or scope tests to intent-stream equivalence and explicitly defer mutation. The roadmap wording “via CompiledTimeline” suggests at least intent stream + boot validation, but “Tohakken” likely implies real Delay/Blessed behavior.

6. **Use `World::try_query::<&T>()` inside predicates/hooks that only have `&World`.** Project memory notes Bevy 0.18 borrow rules: `World::query` needs `&mut self`; use `try_query` for immutable component reads from `SkillCtx::world`.

## Recommended approach

### Data model

Add a v2 timeline field to `SkillDef` while keeping v1 effects for compatibility during migration:

- `timeline: Option<SkillTimelineRon>` or equivalent, default `None`.
- `SkillTimelineRon` should be `#[serde(deny_unknown_fields)]` and close to existing `CompiledTimeline` shape:
  - `entry: String`
  - `beats: Vec<BeatRon>`
  - `edges: Vec<EdgeRon>`
  - `BeatRon { id, kind, hook, selector, presentation }`
  - loop kind with `body` and `exit_when` if needed later.

Compile `String` ids to `'static` ids only if the code deliberately leaks/boxes strings; otherwise prefer changing `CompiledTimeline` ids from `&'static str` to `String`/`Arc<str>` for asset-sourced data. This is a critical type choice. Current hand-built tests use `&'static str`; asset RON cannot naturally borrow `'static` unless using `Box::leak`, which is undesirable in reload paths. Recommendation: migrate timeline id fields and registry lookup sites to `String`/`Arc<str>` for compiled data, while `Registry` keys can remain `&'static str` and accept lookup by `&str`.

### Compiler module seam

Create a small compiler module rather than embedding logic in `skills_ron.rs`:

- `src/data/skill_timeline.rs` or `src/data/skills_compiler.rs`
  - `compile_skill_timeline(skill: &SkillDef) -> Result<Option<CompiledTimeline>, SkillTimelineCompileError>`
  - `compile_skill_book_timelines(book: &SkillBook) -> Result<Vec<CompiledTimeline>, SkillTimelineCompileError>`
  - `validate_compiled_timelines(timelines: &[CompiledTimeline], regs: &ExtRegistries) -> Result<(), ...>` optional adapter around `validate_timeline_refs`.

Keep `skills_ron.rs` focused on serde/schema and semantic validation.

### Built-in extension module seam

Create `src/combat/api/builtins.rs` or `src/combat/api/builtin_ext.rs` with:

- `pub fn register_builtin_extensions(regs: &mut ExtRegistries)`.
- Selectors initially: `core/primary` (returns primary); optionally `core/no_targets`.
- Predicates initially: `core/always`, `core/never`, maybe loop predicates only if needed.
- Hooks initially for exact needed operations:
  - `core/deal_damage_fixed` only works if amount/tag are discoverable. Since hook signature has no beat params, either use per-skill generated hook ids (not possible dynamically) or add static params to `Beat`/`BeatKind::Impact`.

This reveals a design gap: hooks have only id, no per-beat parameter payload. RON can reference `hook: "core/deal_damage"`, but the hook cannot know amount/status/duration unless it reads a side table. Options:

1. Add `params` to `Beat` (e.g. serde-compatible `BTreeMap<String, RonValueLike>` or a typed `BeatParams` enum). Built-in hooks read `ctx.timeline_params(ev.beat_id)` or params included in `BeatEvent`.
2. Generate one Rust hook per concrete effect (not feasible for RON-defined values).
3. Encode params into hook id strings (`core/deal_damage/16/electric`) and parse at runtime (bad for validation and allocations).

Recommendation: add typed, minimal beat params. For S05, a `BeatPayload`/`BeatEffect` enum can cover `DealDamage { amount, tag }`, `BreakToughness { amount, tag }`, `ApplyStatus { kind, duration }`, `DelayTurn { pct }`, `ApplyBuff { kind, duration }`, and `BlueprintSignal { owner, name, payload }`. Then built-in hooks can be generic. This is temporarily similar to `Effect`, but scoped to compiled timeline beat payloads rather than the source RON DSL. If that violates the “drop enum Effect” end-state, use a `BTreeMap<String, ScalarParam>` plus strongly typed extraction helpers.

### First proof task

Implement `petit_thunder` first:

- Add a RON timeline for `petit_thunder` with one or more impact beats using `core/primary`.
- Compile it into `CompiledTimeline`.
- Register built-ins.
- Run `BeatRunner` over a fixture and assert pending intents include, in order:
  - `DealDamage { amount: 16, tag: Electric }`
  - `BreakToughness { amount: 8, tag: Electric }` or equivalent toughness intent
  - `ApplyStatus { kind: Paralyzed, duration_turns: 1 }`
  - `BlueprintSignal { owner: tentomon, name: build_static_charge, payload Amount(1) }`
- If applier work is included, drain through `intent_applier` and verify HP/status/toughness/signal event. If not, keep this as an intent-stream test and explicitly document mutation as follow-up.

### Tohakken task shape

Before implementing, align id and semantics:

- Either update live `assets/data/skills.ron` Renamon ult to `SkillId("tohakken")` and update `assets/data/units.ron`, or keep `SkillId("renamon_ult")` but name/timeline it as Tohakken.
- Expected from docs: all alive enemies receive Holy/Light damage and DelayTurn 30%; all alive allies receive `Blessed` 2 turns. Live `DamageTag` has `Light`, not necessarily `Holy`.
- This probably requires world-aware target expansion in hook or selector.

## Natural seams for planner

1. **Schema + compiler only**
   - Files: `src/data/skills_ron.rs`, new `src/data/skill_timeline.rs`, `src/data/mod.rs`.
   - Outputs: parse RON timeline, compile to `CompiledTimeline`, unit tests for valid/invalid refs.

2. **Built-in extension registration**
   - Files: new `src/combat/api/builtins.rs`, `src/combat/api/mod.rs`, `src/combat/plugin.rs`.
   - Outputs: `register_builtin_extensions` called during plugin build before finish validation; tests assert ids exist.

3. **Runtime execution proof for Petit Thunder**
   - Files: `assets/data/skills.ron`, new `tests/compiled_timeline_petit_thunder.rs`.
   - Outputs: compiled timeline produces expected intent stream; optional applier state assertions.

4. **Applier expansion for effects used in S05**
   - Files: `src/combat/api/applier.rs`, maybe status/toughness/turn-order helpers.
   - Outputs: implement `ApplyStatus`, `BreakToughness`, `DelayTurn`, `ApplyBuff` only as needed.
   - This seam can be split further; `ApplyStatus` is independent from `DelayTurn`/`ApplyBuff`.

5. **Tohakken asset/semantics alignment**
   - Files: `assets/data/skills.ron`, `assets/data/units.ron`, new test for compiled Tohakken.
   - Outputs: id choice made, compiled multi-target timeline, intent stream order damage → delay → buff.

6. **Boot typo failure**
   - Files: compiler tests and/or `src/data/mod.rs`, `src/combat/plugin.rs`.
   - Outputs: a test that a bad hook/selector/predicate id in a RON timeline yields a deterministic validation error before gameplay execution. If using asset load, test the `validate_skill_book_on_load`/compiler path; if using `TimelineLibrary`, test `App::finish` panic.

## Suggested verification

Minimum targeted commands after implementation:

```bash
cargo test --test compiled_timeline_petit_thunder
cargo test --test compiled_timeline_tohakken
cargo test --test timeline_validate_typo
cargo test --lib data::skills_ron
cargo check
cargo check --features windowed
```

Full gate before slice completion:

```bash
cargo test
cargo check
cargo check --features windowed
rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'
```

## Watch-outs for executors

- Do not remove `enum Effect` in S05 unless planner intentionally expands scope; S06 owns full active migration + drop.
- Do not put Digimon-specific ids or logic in `src/combat/api/*`; `tohakken`/`petit_thunder` should live in assets/tests, while built-ins stay generic.
- Avoid `Box::leak` for asset string ids if hot reload remains possible; prefer owned id types in compiled timelines.
- Keep headless-first: no windowed-only imports in `src/combat/api` or compiler modules.
- Preserve DryRun/Execute/Preview intent-stream parity; tests comparing streams should normalize `cast_id` if runs allocate different ids.
- Loop circuit-breaker expectation remains exactly `MAX_HOPS = 256` intents for one intent per hop, not `MAX_HOPS + 1`.

## Sources inspected

- `src/combat/api/timeline.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/runner_common.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/plugin.rs`
- `src/data/skills_ron.rs`
- `src/data/mod.rs`
- `src/combat/resolution.rs`
- `tests/timeline_chain_bolt_port.rs`
- `tests/timeline_validate_typo.rs`
- `assets/data/skills.ron`
- `assets/data/units.ron`
- `docs/future_design_draft/digimon/renamon/03_ult_tohakken.md`
