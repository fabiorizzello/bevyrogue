# S04 Research — Baby Burner reactive detonate + flash VFX

## Summary

Depth: targeted-to-deep. The slice is not hard because of Bevy APIs; it is hard because the current combat/animation seams intentionally buffer timeline effects until after release, while the Baby Burner draft describes a mid-skill reactive `UnitDied -> ReactiveDetonate` branch.

Recommendation: implement S04 as a Rust-side, post-application reactive detonate/flash proof, not as a new RON/editor/FSM feature. Keep shared combat code Digimon-free by adding or reusing a generic extension seam, then register Agumon-specific Baby Burner logic under `src/combat/blueprints/agumon/`. Drive the visible flash from generic combat events / blueprint transition envelopes and keep the windowed rendering behind `feature = "windowed"`.

The highest-risk first proof is a deterministic headless test where Agumon casts `agumon_ult` on a Heated target that dies, adjacent enemies take deterministic detonate damage exactly once, no detonate happens on non-lethal / non-Baby-Burner kills, and existing timeline barrier tests remain green.

## Active Requirements / Constraints

- R002 headless-first: the reactive rule must be provable without a window. Windowed flash should be a projection of combat output, not the source of truth.
- R004 determinism: no timers/RNG for combat. If the flash needs a lifetime, use a deterministic frame counter for presentation only; do not gate damage on wall-clock or render time.
- R005 feature-gating: egui/wgpu/window-only flash UI belongs under `src/windowed/` or `src/ui/` windowed systems; combat and blueprint logic must not depend on egui/winit/wgpu.
- R003 clip-atlas parity: if any Agumon graph/frame ranges are changed, keep `tests/clip_atlas_parity.rs` and hardcoded animation tests green. Prefer no atlas/clip changes for this slice.
- R006 hygiene: add fresh verification evidence and document any environment limitation for live window smoke.

## Skills Discovered

Core technologies are Rust, Bevy ECS, RON assets, and the local combat timeline/runtime. Relevant installed skills are already present in the prompt: `bevy`, `rust-best-practices`, and `rust-testing`. No external skill install is needed; Bevy is already the project’s primary framework and the work is local-code integration rather than new library usage.

## Existing Implementation Landscape

### Baby Burner data today

- `assets/data/digimon/agumon/skills.ron`
  - `baby_flame` and `sharp_claws` have `timeline: Some(...)`.
  - `agumon_ult` / Baby Burner is still legacy-only: `legacy_ops: [Damage(50), ToughnessHit(30)]`, `custom_signals: [(owner: "agumon", signal: "apply_thermal_spark", payload: Amount(3))]`, and no compiled timeline.
  - Because it lacks `timeline`, `resolve_action_system` routes it through the legacy single-target pipeline, not `run_timeline_backed_action`.

### Current two-clock/timeline seam

- `src/combat/runtime/runner.rs`
  - `BeatRunner` fires a beat hook, buffers `Intent`s, then in `Clock::Windowed` returns `StepOutcome::AwaitingCue` when the beat has `presentation`.
  - Effects are not applied to the world beat-by-beat; they are buffered until finalization.
- `src/combat/turn_system/pipeline/timeline_exec.rs`
  - On `StepOutcome::AwaitingCue`, it stores `SuspendedTimelineState` and keeps `CombatPhase::Resolving`.
  - On release, `finalize_timeline_action` extends `IntentQueue`, calls `intent_applier`, then emits `OnSkillCast`, `UltimateUsed`, `OnActionApplied`, and `OnActionResolved`.
  - Implication: a same-timeline `UnitDied -> ReactiveDetonate` branch cannot observe damage from an earlier beat without changing the timeline executor to commit/react between beats.

### Current animation/player limits

- `src/animation/anim_graph.rs`
  - Schema contains `Predicate::KernelEvent(UnitDied, UltimateUsed, ...)`, `FrameCueCommand::Presentation(Command)`, and `ReleaseKernel`.
- `src/animation/player.rs`
  - Runtime currently evaluates only `TimeInNode`, `Always`, and `KernelCue`; other predicates return false.
  - Implication: an AnimGraph `KernelEvent(UnitDied) -> ReactiveDetonate` branch from the design draft is schema-shaped but not executable yet.
- `src/windowed/render.rs`
  - The windowed bridge only detects `FrameCueCommand::ReleaseKernel` and only has explicit mode support for Sharp Claws.
  - It does not execute `FrameCueCommand::Presentation(SpawnParticle(...))` or render a particle system.

### Reactive/event infrastructure available

- `src/combat/observability/events.rs`
  - `CombatEventKind::UnitDied { status_remaining, heated_remaining }` already exists.
- `src/combat/runtime/event_bridge.rs`
  - Mirrors all `CombatEvent`s to `SignalBus` as `Signal::CombatEvent(event.clone())`.
  - Also emits legacy `kernel/ult_used` blueprint signal for `UltimateUsed`.
- `src/combat/runtime/passive_runner.rs`
  - Passive runners can react to bridged events and enqueue intents, then `intent_applier` flushes them deterministically in the same update loop.
- `src/combat/runtime/event_filter.rs`
  - Supports `EventFilter::combat(...)` and custom filters over `Signal`.
- `src/combat/blueprints/agumon/mod.rs`
  - Already registers Agumon blueprint hooks/predicates/selectors for Twin Core passive and Bouncing Fire.
  - Natural home for `BabyBurner` Rust-only reactive logic.

### UnitDied payload caveat

- Legacy resolution builds `UnitDied` payload from defender `StatusBag` via `ko_payload`; `tests/unit_died_payload.rs` covers this.
- Timeline `intent_applier` currently emits `UnitDied { status_remaining: vec![], heated_remaining: 0 }` in `src/combat/runtime/applier/effects/damage.rs`.
- If S04 keeps Baby Burner on the legacy path, Heated detonate can use the existing payload. If S04 migrates Baby Burner to timeline first, fixing timeline-applier KO payload parity becomes mandatory before detonate can be correct.

### Twin Core / custom signal caveat

- Legacy `custom_signals` route through `blueprints::transitions_for_action`, where `agumon::apply_thermal_spark` becomes a Twin Core tag transition.
- Timeline `BeatPayload::BlueprintSignal` currently emits `OnKernelTransition::Blueprint { owner, name, payload }` directly; it does not run through the custom-signal dispatcher that converts Agumon signals into Twin Core tag transitions.
- Do not assume a timeline `BlueprintSignal(owner: "agumon", name: "apply_thermal_spark")` updates `TwinCoreState` unless that bridge is deliberately added or the timeline emits the lower-level `twin_core` signal/transition intentionally.

## Design Draft Context

`docs/future_design_draft/digimon/agumon/03_ult_baby_burner.md` describes the full future Baby Burner as:

- main Fire hit on primary;
- optional splash to adjacent targets;
- `OnKill -> Detonate(Heated)` using `UnitDied.heated_remaining`;
- per-adjacent `heated_detonate` flash VFX, not on the dead primary;
- future QTE and event-payload param sources.

For M002 S04, do not import the whole draft. The roadmap says “Rust code, no RON/editor,” so the safe subset is: lethal Baby Burner reads `heated_remaining`, deterministically damages adjacent alive enemies, and emits a presentation-observable flash signal. QTE, event-payload ParamRef schema, editor support, and full reactive AnimGraph branching should stay out of scope.

## Recommended Implementation Direction

### Preferred path: generic post-application reaction seam + Agumon registration

1. Add a generic extension seam if none is sufficient:
   - Context should include at least `World`, `CombatEvent`/`UnitDied` payload, `InFlightAction` or `skill_id`, `source`, `primary_target`, and `cast_id`.
   - It should return/enqueue generic `Intent`s and/or generic presentation/blueprint transition events.
   - Keep this seam owner-neutral; no `agumon_ult`, `Heated`, or `BabyBurner` strings in shared pipeline code.
2. Register Agumon Baby Burner logic in `src/combat/blueprints/agumon/`:
   - Trigger only when the successful action skill is `SkillId("agumon_ult")` and a `UnitDied` occurred for the primary target.
   - Require `heated_remaining > 0` for detonate damage, unless design chooses flash-only on zero stacks; tests should lock the chosen behavior.
   - Resolve adjacent alive enemies using existing `TargetShape::Blast` / slot-index semantics and exclude the dead primary.
   - Enqueue deterministic `Intent::DealDamage` for each adjacent target. Suggested first-pass scalar from draft: `8 * heated_remaining`, but planner should confirm desired amount before executor bakes it in.
   - Emit a generic blueprint/presentation transition such as `OnKernelTransition::Blueprint { owner: "agumon", name: "baby_burner_detonate", payload: Amount(heated_remaining) }` for observability and windowed flash.
3. Windowed flash should listen to the generic event/transition and render presentation only:
   - A small `BabyBurnerFlash`/`FlashVfx` resource with deterministic `frames_left` is enough for S04.
   - Draw as an egui chip/overlay or simple Bevy component under `feature = "windowed"`.
   - Tests can cover the event-to-flash resource/helper without requiring a display.

### Avoid for S04

- Do not implement full AnimGraph `KernelEvent(UnitDied)` branching yet; `AnimGraphPlayer` does not evaluate those predicates.
- Do not implement QTE / `EventPayload` ParamRef / expression DSL.
- Do not make the kernel timeline commit effects beat-by-beat unless the planner deliberately expands the slice; that would risk R004/R002 and many timeline tests.
- Do not put Agumon-specific strings in shared combat pipeline branches. If shared code must call out, use an extension registry or generic event hook.

## Natural Seams / Work Units

1. **Headless reactive rule**
   - Files: `src/combat/runtime/registry.rs`, `src/combat/turn_system/pipeline/...`, `src/combat/blueprints/agumon/mod.rs` or new `src/combat/blueprints/agumon/baby_burner.rs`.
   - Purpose: provide owner-neutral post-KO/action reaction seam and Agumon-specific detonate registration.
   - Independent after the trigger shape is decided.

2. **Adjacent target + damage helper**
   - Files: likely `src/combat/blueprints/agumon/...`; optionally reuse `src/combat/resolution/types.rs::resolve_targets` and `TargetShape::Blast`.
   - Purpose: isolate deterministic target resolution and `heated_remaining * scalar` calculation for simple unit tests.

3. **UnitDied payload parity if timeline path is chosen**
   - Files: `src/combat/runtime/applier/effects/damage.rs`, tests near `tests/unit_died_payload.rs` or a new timeline-applier test.
   - Purpose: make timeline damage KO payload match legacy `ko_payload` before Baby Burner moves to a timeline.
   - This is only mandatory if `agumon_ult` becomes timeline-backed in S04.

4. **Windowed flash projection**
   - Files: `src/windowed/mod.rs`, `src/windowed/render.rs` or `src/ui/combat_panel/*`, plus `tests/windowed_preview_cache.rs` or new feature-gated test.
   - Purpose: map detonate event/transition to a visible short-lived flash indicator without mutating combat state.

5. **Asset/graph updates only if needed**
   - Files: `assets/digimon/agumon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/agumon_sharp_claws_asset.rs`.
   - Purpose: add Baby Burner nodes only if executor is explicitly asked to animate Baby Burner this slice.
   - Watch MEM032/MEM037: keep `clip: "all"`; keep atlas parity tests green.

## First Proof

Implement the smallest deterministic headless proof before any windowed work:

- Spawn Agumon with `ultimate_skill = agumon_ult`, ultimate charge ready, primary enemy at lethal HP with `StatusBag::Heated` duration 2, and two adjacent alive enemies with `SlotIndex` around the primary.
- Cast `ActionIntent::Ultimate`.
- Assert:
  - primary receives normal Baby Burner damage and emits `UnitDied { heated_remaining: 2 }`;
  - each adjacent alive target receives exactly one detonate damage application;
  - no detonate on non-lethal Baby Burner;
  - no detonate on a different skill that kills a Heated target;
  - no duplicate detonate on duplicate/no-op events or repeated update ticks;
  - a presentation-observable detonate transition/flash event is emitted once.

This proof finds the core design bug early: without access to skill/action context, a passive `UnitDied` listener cannot safely know whether the kill came from Baby Burner.

## Verification Plan

Required targeted checks after implementation:

```bash
cargo test --test unit_died_payload
cargo test --test timeline_cue_barrier_pipeline
cargo test --test timeline_two_clock_parity
cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity
cargo test --features windowed --test windowed_preview_cache
cargo test --lib
cargo build --no-default-features
cargo build --features windowed
```

Add one new focused test file if possible, e.g. `tests/agumon_baby_burner_reactive.rs`, covering lethal/non-lethal/non-Baby-Burner/no-duplicate behavior. If a windowed flash helper is added, either extend `tests/windowed_preview_cache.rs` or create a small `#[cfg(feature = "windowed")]` test that drives the event into the flash resource without opening a window.

Optional live smoke remains environment-dependent:

```bash
BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue
```

Use the explicit `--bin bevyrogue` form; S02 noted bare `cargo run --features windowed` is a multi-binary gotcha.

## Risks / Watch-outs

- **Same-timeline reactive branch is not currently viable.** Timeline effects are buffered; AnimGraph `KernelEvent` predicates are schema-only at runtime. Trying to implement the full draft branch now risks a broad executor/timeline rewrite.
- **Skill context is essential.** A pure passive `UnitDied` listener sees event source/target/cast but not necessarily the owning skill unless a new seam supplies it or a cast tracker is reliable. Avoid accidental “any Agumon kill detonates Heated.”
- **Event order is tricky.** In legacy single-target flow, `UnitDied` occurs before `UltimateUsed`; in timeline flow, damage events occur during `intent_applier` before final `OnSkillCast`/`UltimateUsed`. Do not depend on later events to arm the same cast’s detonate.
- **Timeline KO payload currently loses Heated.** If Baby Burner is migrated to timeline, fix `runtime/applier/effects/damage.rs` first or detonate will always see `heated_remaining = 0`.
- **Presentation commands are not rendered.** Adding `SpawnParticle("heated_detonate")` to RON will parse/validate only if catalogs are updated, but it will not visibly flash until a windowed consumer exists.
- **Existing tests touch `agumon_ult`.** `tests/follow_up_chains.rs`, `tests/twin_core.rs`, unit roundtrip tests, and ultimate/follow-up tests reference it. Prefer additive tests and keep event streams stable where possible.

## Key Files

- `assets/data/digimon/agumon/skills.ron` — Baby Burner legacy definition; Sharp Claws/Baby Flame timeline examples.
- `src/combat/blueprints/agumon/mod.rs` — Agumon extension registration; likely home for detonate hook/registration.
- `src/combat/blueprints/agumon/signals.rs` — legacy Agumon custom signal mapping to Twin Core tags.
- `src/combat/runtime/registry.rs` — extension registry axes; candidate place for a generic post-action/post-KO reaction axis.
- `src/combat/runtime/event_bridge.rs`, `src/combat/runtime/passive_runner.rs`, `src/combat/runtime/event_filter.rs` — existing reactive bus infrastructure.
- `src/combat/runtime/applier/effects/damage.rs` — timeline-path KO event currently emits empty status/zero heated payload.
- `src/combat/turn_system/pipeline/paths/single_target.rs` — legacy Baby Burner execution path today; has `InFlightAction` context and KO events in hand.
- `src/combat/turn_system/pipeline/timeline_exec.rs` — timeline execution/finalization; important if converting Baby Burner to timeline.
- `src/windowed/render.rs` — current windowed presentation bridge; Sharp Claws-only release handling.
- `src/ui/combat_panel/labels.rs`, `src/ui/combat_panel/render.rs` — pure UI helper pattern for displayable diagnostics.
- `tests/unit_died_payload.rs` — legacy KO payload proof.
- `tests/timeline_cue_barrier_pipeline.rs`, `tests/timeline_two_clock_parity.rs` — R004/R002 regression gates.
- `tests/windowed_preview_cache.rs` — feature-gated pure windowed/UI test pattern.

## Sources

- Local code scans via `rg`/`gsd_exec` for Baby Burner, timeline barriers, event bridge, and Twin Core.
- `docs/future_design_draft/digimon/agumon/03_ult_baby_burner.md` for intended Baby Burner detonate/VFX semantics.
- S02 summary for established two-clock barrier and windowed telegraph-chip patterns.
