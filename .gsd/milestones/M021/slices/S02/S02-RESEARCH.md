# S02 — Timeline FSM + validate_timeline_refs (Research)

**Status:** ready for planner. Depth: **targeted** (architecture validated by spike `M021-timeline-fsm` 33/33; this slice ports the standalone PoC into the live crate against the S01 framework).

## Summary

S02 lifts the Timeline-FSM model from the standalone spike into `src/combat/api/`. After this slice the kernel exposes:

- `Beat`, `BeatKind`, `Presentation`, `BeatEdge`, `CompiledTimeline`, `BeatEvent` types (graph data).
- `BeatRunner` with a single-level `LoopFrame` walking `BeatKind::Loop { body, exit_when }`.
- `validate_timeline_refs` recursive validator, wired into `App::finish()` via `CombatPlugin::finish` (or a startup pass) for fail-fast boot.
- Concrete `ExtPoint::Fn` signatures replacing the S01 placeholder `fn()` for at least `HookExt`, `SelectorExt`, `PredicateExt`, `CueExt` (the four axes referenced from the timeline graph).
- `SkillCtx` extended with whatever read accessors the fixture and `chain_bolt` port need (caster-aware target selection + cast_hit_set for NoRepeat).

The slice's three demo gates (from roadmap):

1. **Fixture OnTurnStart kills target verde** — a hand-rolled `CompiledTimeline` whose `Impact` beat hook enqueues `Intent::DealDamage` of magnitude ≥ target HP; the existing S01 `intent_applier` path routes it to `apply_deal_damage`, target dies, suite asserts.
2. **`validate_timeline_refs` scopre typo** — a unit test installs an `ExtRegistries` missing one hook (or selector / predicate / cue) and asserts that the validator returns `Err` enumerating the dangling reference with axis + site (`beat <id>` or `edge from→to`).
3. **`LoopFrame` single-level su `chain_bolt` port** — the existing `chain_bolt` fixture (`src/data/skills_ron.rs:1050`, 3-hop Bounce + LowestHpPctAlive + NoRepeat + Falloff 80%) is re-expressed as a `CompiledTimeline` whose body is a single `Impact` beat plus an `exit_when` predicate combining `hop_index ≥ 3 || pool_exhausted`. `BeatRunner` walks it; the per-hop damage curve is emitted via a formula registered in `FormulaExt` (or inlined in the hook for S02; full RON→CompiledTimeline compiler is S05).

## Recommendation

Port the spike's `lib.rs` types and `BeatRunner` shape verbatim (with the Bevy-aware adjustments below) rather than re-deriving. The spike already validated 4 architecturally distinct patterns (Loop+skilltree-gate, blueprint-state, identity filter, RNG-gated edge) on 33 tests; S02 is a transcription, not a redesign.

Three deltas the port must absorb:

1. **No thread-local runtime.** The spike used a `RuntimeGuard` (raw pointers in thread-locals) to keep hook signatures flat while satisfying borrow checker. Production replaces this with `SkillCtx<'a>` carrying borrows. Per **F7** in the spike findings: the hook signature `fn(&BeatEvent, &mut SkillCtx)` survives, but `SkillCtx<'a>` gains lifetime-bound fields (`pub registries: &'a ExtRegistries`, `pub state: &'a CombatState` or equivalent query handle). The signature stored in `Registry<HookExt>::Fn` becomes `for<'a> fn(&BeatEvent, &mut SkillCtx<'a>)` — still `Copy`, still no trait object.
2. **Real `ExtPoint::Fn` signatures.** S01 left every axis as `type Fn = fn()`. S02 must refine at minimum the four axes the timeline graph references: hook, selector, predicate, cue. Formula/Tick/AI can stay placeholder if not exercised yet, or be refined to spike-style signatures opportunistically — but the **kernel grep gate** (`rg "TwinCore|BatteryLoop|…" src/combat/ --glob '!blueprints/**'` → 0 in M021 success criteria) only requires the kernel to be name-free, not to wire every axis in S02.
3. **`validate_timeline_refs` runs at `App::finish()`.** Bevy 0.18 `Plugin::finish` is the natural seam (CombatPlugin already exists as a Plugin per S01). The validator iterates all known timelines (S02 will only have the fixture + the chain_bolt port — full skill book validation arrives once skills.ron carries CompiledTimelines in S05).

## Implementation Landscape

### Where things go

| Concept | New file (suggested) | Notes |
|---|---|---|
| `Beat`, `BeatKind`, `Presentation`, `BeatEdge`, `CompiledTimeline` | `src/combat/api/timeline.rs` | Pure data types, no Bevy. Spike `lib.rs` lines ~509–605. `BeatKind::Loop { body: Vec<Beat>, exit_when: PredicateId }`. Drop `derive(PartialEq, Eq)` from `BeatKind` (Loop body is `Vec<Beat>`, not needed). |
| `BeatEvent` (with `hop_index`, `beat_targets`) | `src/combat/api/timeline.rs` | Spike lib.rs:192–206. `cast_id: CastId` (already in S01 intent.rs). |
| `BeatRunner`, `LoopFrame`, `LoopFrame` state machine | `src/combat/api/runner.rs` | Spike lib.rs:776–1089. Replace `RuntimeGuard`/thread-local accessors with `SkillCtx<'a>` borrows (F7). |
| `validate_timeline_refs` + `ValidationError` | `src/combat/api/timeline.rs` (or a `validate.rs`) | Spike lib.rs:656–746. Recursive over `Loop.body`. |
| Real `ExtPoint::Fn` signatures | `src/combat/api/registry.rs` (replace placeholders) | Hook: `for<'a> fn(&BeatEvent, &mut SkillCtx<'a>)`. Selector: `for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>`. Predicate: `for<'a> fn(&BeatEvent, &SkillCtx<'a>) -> bool`. Cue: `for<'a> fn(&CueCtx<'a>) -> CueId`. |
| `SelectorCtx`, `CueCtx` | `src/combat/api/timeline.rs` or `skill_ctx.rs` | Spike lib.rs:208–229. Pared down for S02 (caster, primary_target, state borrow). |
| `SkillCtx<'a>` extension | `src/combat/api/skill_ctx.rs` (edit) | Add `registries: &'a ExtRegistries` + a state borrow handle (`world: &'a World` is simplest for S02 fixture; production iterates ECS). For `chain_bolt` NoRepeat: add `cast_hit_set: &mut HashSet<UnitId>` borrowed from the runner. |
| `App::finish` validation hook | `src/combat/plugin.rs` (edit) | Implement `Plugin::finish(&self, app: &mut App)` that walks any registered fixtures. Initial set is empty until S05; S02 ships the hook + a unit test exercising the validator directly. |
| Fixture `OnTurnStart` timeline + test | `tests/timeline_onturnstart_kills.rs` | Headless `App`, spawns 1 unit-alive target with HP=1, drives `BeatRunner::run_to_completion`, asserts target dead via existing `Unit.hp` reads or `OnDamageDealt` event. |
| Validation test | `tests/timeline_validate_typo.rs` (or inline in `timeline.rs`) | Build a timeline that references `"missing_hook"` and assert validator returns `Err` with axis="hook". |
| `chain_bolt` CompiledTimeline port | `tests/timeline_chain_bolt_port.rs` | Build the timeline by hand (still no RON compiler), register a Falloff formula + `lowest_hp_pct` selector + NoRepeat predicate, walk runner, assert: 3 `DealDamage` Intents in stream, each lowest-alive target, no repeats, damage falloff 80% per hop. |

### Why this build order

1. **Data types first** (`Beat`, `Presentation`, `BeatEdge`, `CompiledTimeline`, `BeatEvent`): zero-dep transcription from spike. No runtime concerns. Validator depends on them.
2. **Refine `ExtPoint::Fn` signatures**: hook/selector/predicate/cue. Breaks S01 placeholder tests in `registry.rs` — update those tests to use a representative axis (or keep the `NumExt` test axis pattern intact since it's independent).
3. **`SkillCtx<'a>` extension**: add registries + state borrows + cast_hit_set. This is where the F7 promotion lands. Decide the state-borrow shape (full `&World` for simplicity in S02 vs a narrow `SystemParam`-style aggregate — recommend `&World` for S02 since `intent_applier` already uses `&mut World`; revisit in S05+).
4. **`validate_timeline_refs`**: pure function over types from step 1 + an `ExtRegistries`. Unit-testable without Bevy. Easiest place to start TDD.
5. **`BeatRunner` linear path**: walk Cast / Phase / Impact / Aftermath, no Loop yet. Drive fixture-1 (OnTurnStart kills target).
6. **`LoopFrame` + body cursor + `exit_when` evaluation**: spike lib.rs has the canonical 80-line implementation. Drive fixture-3 (chain_bolt port).
7. **`Plugin::finish` wire**: register the validator. Initially walks zero timelines; the unit test invokes `validate_timeline_refs` directly to prove the typo case. (Wiring real fixture timelines into a `Resource<TimelineLibrary>` is acceptable scope creep for S02; the planner can choose.)

### Risks and watch-outs (carried from spike)

- **F1 (gate fallback edge)**: every gated edge needs an unconditional fallback edge from the same anchor, or the runner halts when the predicate fails. The chain_bolt port has only the Loop body internal `exit_when`, so this is mostly a documentation/lint concern for now — but it's worth a doc-comment on `next_from` so future authors of the RON compiler (S05) bake the fallback into the lowering.
- **F6 (running `beat_targets`)**: the runner must carry `last_beat_targets` across beats. The spike fixed a subtle bug where gates evaluated against `base_event.beat_targets = []`. Port the fix verbatim (spike `BeatRunner::last_beat_targets` field).
- **F8 (uniform validation only for graph-referenced axes)**: hook/selector/predicate/cue are validated by traversal of beats+edges. Formula/tick/ai are referenced from inside hook bodies and validated only at lookup. Document this in the validator's doc-comment so reviewers don't expect more than what's structurally possible.
- **F15 (pool exhaustion as `exit_when` clause)**: the chain_bolt `exit_when` predicate must OR three conditions: `hop_index >= max_hops`, `pool ⊆ cast_hit_set` (NoRepeat exhaustion), and (for the spike's bouncing_fire) `!skilltree.has("...")`. S02 only needs the first two for chain_bolt; the talent-gate clause is S08+.
- **Determinism (I1)** is not exercised in S02 (no RNG predicates yet) but must not be regressed. The fixture and port should be RNG-free; lowest_hp_pct selector breaks ties by `UnitId` ascending, not random.
- **DryRun ≡ Execute (I2)** is not enforced in S02 either — the slice S03 owns it. But `SkillCtxMode` is already routed through the runner (spike lib.rs:949). Make sure `BeatRunner::step` takes `mode: SkillCtxMode` so S03 can flip it without API churn.
- **NoRepeat selector vs `cast_hit_set`**: the spike's selector reads `ctx.cast_hit_set()`. In production the runner owns the set (spike lib.rs:812: `cast_hit_set: HashSet<UnitId>` field), folds in `Intent::DealDamage.target` after each hook fires, and exposes a borrow to `SkillCtx`. Port that ownership: it's the cleanest place — `intent_applier` cannot help here because the runner needs the set *during* the cast, before the applier drains the queue.

### Don't hand-roll (use existing)

- **Don't reinvent `Intent::DealDamage` routing.** S01's `apply_deal_damage` (`src/combat/api/applier.rs:64`) already wires the existing damage formula. The fixture and chain_bolt port enqueue `Intent::DealDamage`; the applier handles the rest. No new combat math in S02.
- **Don't add a new `CastId` allocator.** `CastIdGen` (S01) issues monotonic ids; the fixture test must pull one from `world.resource_mut::<CastIdGen>().next()` so the assert on the cast-scoped event matches.
- **Don't write a RON compiler.** S05 owns that. S02 timelines are hand-built in Rust (`CompiledTimeline { beats: vec![...], edges: vec![...], ... }`).
- **Don't migrate any of the 18 active skills.** S06 owns that. The `chain_bolt` port lives in a test, not in `skills.ron`. The real `chain_bolt` SkillDef in `src/data/skills_ron.rs:1050` stays untouched.

### Files that will change

```
src/combat/api/timeline.rs           (NEW — Beat, BeatKind, Presentation, BeatEdge,
                                       CompiledTimeline, BeatEvent, SelectorCtx, CueCtx,
                                       validate_timeline_refs, ValidationError)
src/combat/api/runner.rs             (NEW — BeatRunner, LoopFrame)
src/combat/api/registry.rs           (EDIT — refine HookExt/SelectorExt/PredicateExt/CueExt Fn signatures)
src/combat/api/skill_ctx.rs          (EDIT — extend SkillCtx<'a> with registries borrow,
                                       state borrow handle, cast_hit_set borrow)
src/combat/api/mod.rs                (EDIT — pub mod timeline; pub mod runner)
src/combat/plugin.rs                 (EDIT — impl Plugin::finish to run validator over
                                       registered timelines)
tests/timeline_onturnstart_kills.rs  (NEW — fixture test #1)
tests/timeline_validate_typo.rs      (NEW — validator test #2; or inline unit test)
tests/timeline_chain_bolt_port.rs    (NEW — chain_bolt LoopFrame test #3)
```

Zero edits to `src/combat/resolution.rs`, `src/data/skills_ron.rs`, `assets/data/skills.ron`, or anything outside `src/combat/api/` + `src/combat/plugin.rs` + `tests/`. That's the kernel-discipline check (P001 holds).

## Verification (slice-level)

```bash
cargo check                                  # headless, no new warnings
cargo check --features windowed              # winit gate intact
cargo test timeline_onturnstart_kills        # demo gate 1
cargo test timeline_validate_typo            # demo gate 2
cargo test timeline_chain_bolt_port          # demo gate 3
cargo test                                   # full suite still green

# Kernel discipline
rg "use bevy::winit|use bevy::render|use bevy_egui" src/combat/api/  # 0
rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" \
   src/combat/api/                                                   # 0
```

## Skill discovery

Relevant installed skills the planner / executor should consult:

- **`bevy`** — Bevy 0.18 ECS, Plugin lifecycle (`build` vs `finish`), system ordering. Read before wiring `Plugin::finish`.
- **`rust-best-practices`** — borrow-checker patterns; relevant when promoting spike's thread-local `RuntimeGuard` to `SkillCtx<'a>` borrows.
- **`rust-testing`** — fixture style for headless Bevy integration tests in `tests/`.
- **`tdd`** — the three demo gates map cleanly to red-green-refactor passes (validator → fixture → chain_bolt port).
- **`verify-before-complete`** — slice completion gate requires fresh `cargo test` output.

No external library docs needed: the only deps are `bevy 0.18` (already used) and `std::collections::HashMap/HashSet` (already used).

## Sources

- `.gsd/workflows/spikes/M021-timeline-fsm/FINDINGS.md` — 17 findings, 4 pattern fixtures, 33/33 verde. Authoritative.
- `.gsd/workflows/spikes/M021-timeline-fsm/src/lib.rs` — canonical port source (1094 lines, ~600 of which become `src/combat/api/{timeline,runner}.rs`).
- `.gsd/workflows/spikes/M021-timeline-fsm/tests/validation.rs` — 33 tests; the four S02-relevant ones are `validation_catches_missing_hook`, `validation_catches_loop_exit_when_unregistered`, `bouncing_fire_tier1_runs_exactly_one_hop` (pattern for the chain_bolt port), and `dry_run_intent_stream_matches_live` (for the runner-takes-`mode` invariant — applied in S03).
- `.gsd/milestones/M021/slices/S01/S01-SUMMARY.md` — what S01 already shipped (Intent, CastId/Gen, ExtRegistries skeleton, SkillCtx, intent_applier canary, CombatPlugin).
- `src/data/skills_ron.rs:1050` — existing `chain_bolt` SkillDef fixture used as the port reference.
- `M021-CONTEXT.md` §F4, §F8 — timeline FSM target architecture and validation rule.

Slice S02 researched.
