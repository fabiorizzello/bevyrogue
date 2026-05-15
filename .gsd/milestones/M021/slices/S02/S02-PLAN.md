# S02: Timeline FSM + validate_timeline_refs

**Goal:** Port the Timeline-FSM spike (33/33 verde) into the live kernel: introduce `src/combat/api/timeline.rs` (Beat/BeatKind/Presentation/BeatEdge/CompiledTimeline/BeatEvent/SelectorCtx/CueCtx + `validate_timeline_refs`) and `src/combat/api/runner.rs` (`BeatRunner` with single-level `LoopFrame`), refine `HookExt`/`SelectorExt`/`PredicateExt`/`CueExt` `ExtPoint::Fn` signatures from `fn()` placeholders to real `for<'a> fn(...)` shapes, extend `SkillCtx<'a>` with the borrows the runner needs (`registries: &'a ExtRegistries`, a state borrow handle, `cast_hit_set: &mut HashSet<UnitId>`), wire validation via `Plugin::finish` in `CombatPlugin`, and demonstrate end-to-end with three new headless integration tests: (1) fixture OnTurnStart kills target via `Intent::DealDamage` through S01's `intent_applier`, (2) validator reports a dangling-reference typo with axis + site, (3) the existing `chain_bolt` 3-hop Bounce/LowestHpPctAlive/NoRepeat/Falloff-80% pattern expressed as a `CompiledTimeline` with a single-level Loop body, driven by `BeatRunner`, producing exactly 3 `Intent::DealDamage`s on the lowest-HP-alive non-repeating targets with the right per-hop falloff.
**Demo:** Fixture OnTurnStart kills target verde; validate_timeline_refs scopre typo; LoopFrame single-level su chain_bolt port.

## Must-Haves

- `cargo check` headless: exit 0, no new warnings.
- `cargo check --features windowed`: exit 0, no new warnings.
- `cargo test timeline_onturnstart_kills` green.
- `cargo test timeline_validate_typo` green.
- `cargo test timeline_chain_bolt_port` green (3 hops, lowest-HP-alive selection, no repeats, 100/80/64 damage curve or equivalent integer ladder).
- `cargo test` full suite: 0 failures.
- `rg "use bevy::winit|use bevy::render|use bevy_egui" src/combat/api/` → 0 (kernel discipline).
- `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/` → 0.
- `rg "pub fn validate_timeline_refs" src/combat/api/timeline.rs` → 1.
- `rg "pub struct BeatRunner" src/combat/api/runner.rs` → 1.
- `CombatPlugin` implements `Plugin::finish` and calls the validator over any registered timelines (initially empty in S02; the validator test exercises the pure function directly).
- Zero edits to `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `assets/data/skills.ron`, or any non-`api/` combat module beyond `src/combat/plugin.rs` (kernel-discipline check P001 holds).

## Proof Level

- This slice proves: demo

## Integration Closure

Three new integration tests under `tests/` provide closure: `timeline_onturnstart_kills.rs` proves the Intent path (S01 applier) is reached by a runner-driven hook; `timeline_validate_typo.rs` proves the validator's contract on graph-referenced axes (hook/selector/predicate/cue) with axis + site reporting; `timeline_chain_bolt_port.rs` proves `LoopFrame` semantics (body cursor, exit_when, running `beat_targets`/`cast_hit_set`) against an existing well-understood pattern. `CombatPlugin::finish` wires the validator into `App::finish()` so S05+ can register real timelines and fail-fast at boot. No edits to S01 surfaces beyond additive borrows on `SkillCtx<'a>` and additive refinement of four `ExtPoint::Fn` types — S01 inline tests (registry/RNG/applier canary/cast_id propagation) remain green.

## Verification

- None new in S02. The runner emits `Intent` values into the existing S01 queue; CastId propagation through `BeatEvent.cast_id` reuses the S01 invariant. JSONL `Blueprint` round-trip is S04's responsibility.

## Tasks

- [x] **T01: Timeline data types + validate_timeline_refs + validator unit test** `est:120m`
  Why: the graph types and their validator are pure data, Bevy-agnostic, and unblock every later task. Porting them first keeps the surface small and lets the validator test (demo gate 2) drive TDD without a runner.
  - Files: `src/combat/api/timeline.rs`, `src/combat/api/mod.rs`, `tests/timeline_validate_typo.rs`
  - Verify: cargo check && cargo test --test timeline_validate_typo && cargo test --lib combat::api::timeline:: && rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs

- [x] **T02: Refine ExtPoint::Fn signatures (Hook/Selector/Predicate/Cue) + extend SkillCtx<'a>** `est:90m`
  Why: T01's `BeatEvent` and timeline types need real callable signatures on the four graph-referenced axes; T03's `BeatRunner` needs `SkillCtx<'a>` to carry the borrows the spike used thread-locals for (F7). This task does the surgical signature promotion and adds the borrows — nothing more.
  - Files: `src/combat/api/registry.rs`, `src/combat/api/skill_ctx.rs`
  - Verify: cargo check && cargo check --features windowed && cargo test --lib combat::api:: && cargo test --test intent_applier_canary && cargo test --test cast_id_propagation

- [x] **T03: BeatRunner with single-level LoopFrame (no test yet, must compile and be unit-tested)** `est:180m`
  Why: `BeatRunner` is the FSM engine — the slice's largest mechanical port from the spike. Keeping it in its own task isolates the borrow/lifetime work (F7) and lets T04 focus on the integration scenarios.
  - Files: `src/combat/api/runner.rs`, `src/combat/api/mod.rs`
  - Verify: cargo check && cargo test --lib combat::api::runner:: && rg 'pub struct BeatRunner' src/combat/api/runner.rs && rg 'LoopFrame' src/combat/api/runner.rs

- [x] **T04: Demo gates 1 & 3: fixture OnTurnStart kills target + chain_bolt CompiledTimeline port** `est:210m`
  Why: the two runner-driven demo gates from the roadmap. Keeping them in one task pays off because both build a hand-rolled `CompiledTimeline`, both wire a hook fn, and both drive `BeatRunner::run_to_completion` over an existing Bevy `App` — duplication of setup helpers is highest here.
  - Files: `tests/timeline_onturnstart_kills.rs`, `tests/timeline_chain_bolt_port.rs`
  - Verify: cargo test --test timeline_onturnstart_kills && cargo test --test timeline_chain_bolt_port && cargo test

- [x] **T05: CombatPlugin::finish validator hook + slice verification (grep gates, headless+windowed, full suite)** `est:90m`
  Why: closes the slice with the `App::finish()` seam and the full demo-closure verification. The validator hook makes S05+ fail-fast on dangling references; the verification confirms every roadmap success criterion for S02 is green now.
  - Files: `src/combat/plugin.rs`, `src/combat/api/timeline.rs`
  - Verify: cargo check && cargo check --features windowed && cargo test && rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ ; rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat/api/ ; rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs && rg 'pub struct BeatRunner' src/combat/api/runner.rs && rg 'fn finish' src/combat/plugin.rs

## Files Likely Touched

- src/combat/api/timeline.rs
- src/combat/api/mod.rs
- tests/timeline_validate_typo.rs
- src/combat/api/registry.rs
- src/combat/api/skill_ctx.rs
- src/combat/api/runner.rs
- tests/timeline_onturnstart_kills.rs
- tests/timeline_chain_bolt_port.rs
- src/combat/plugin.rs
