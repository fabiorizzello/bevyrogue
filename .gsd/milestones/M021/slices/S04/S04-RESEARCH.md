# S04 — Research: SignalBus + PassiveRunner + Ult instant + `Intent::BlueprintSignal` dispatcher

## Summary

S04 lifts four interlocking primitives out of the placeholder zone into a working reactive layer:

1. **`SignalBus` resource** (today: empty `Resource` placeholder at `src/combat/api/signal.rs:1-15`) — gains a closed `Signal` enum + `VecDeque<Signal>` queue + `App::finish()` taxonomy validation (D028 / I8).
2. **`PassiveRunner`** — new sibling of `BeatRunner` that consumes `Signal`s from the bus and drives a listener-shaped `CompiledTimeline`, producing the same `Intent` stream shape (D025, F16).
3. **`Intent::BlueprintSignal` dispatcher inside `intent_applier`** — today logs `warn!` and drops (`src/combat/api/applier.rs:50-52`). S04 must route to (a) `SignalBus::enqueue(Signal::Blueprint{owner, payload})` AND (b) emit `CombatEvent::OnKernelTransition { transition: CombatKernelTransition::Blueprint { owner, payload } }` for JSONL round-trip (D008 final shape, G-F).
4. **Ult instant cast separation** (D010 precondition) — Ultimate casts must not invoke the normal turn-advance pathway, so the post-ult turn order is determined exclusively by `Intent::AdvanceTurn`-emitting listeners (e.g. Renamon `kitsune_grace` triggering `Self_, +10%`). This is the prerequisite the canonical kitsune_grace fixture needs to be observable end-to-end.

Demo target (per ROADMAP): **Renamon kitsune_grace verde + JSONL `Blueprint` round-trip + `debug_assert` mismatch on unregistered signal**. The fixture is canonically reactive (no cast of its own), so it is the cleanest first proof for both `SignalBus` and `PassiveRunner`. Risk is **high** because four new pieces interlock and the `CombatKernelTransition::Blueprint` variant doesn't exist yet in `src/combat/kernel.rs:889-902` — adding it is a kernel surface change.

This slice does **not** migrate any of the 5 existing Digimon-specific transition variants (`TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`); that is S07–S10 work. S04 only *adds* the `Blueprint` variant and the dispatch path; the old per-blueprint `dispatch()` fns in `src/combat/blueprints/<x>.rs` keep working unchanged.

## Implementation Landscape

### Existing scaffolding that S04 builds on

| Asset | File / location | What S04 uses it for |
|---|---|---|
| `SignalBus` empty Resource | `src/combat/api/signal.rs:1-15` | Becomes the typed `Signal` queue. Module already wired into `CombatPlugin` (`plugin.rs:9,28`). |
| `Intent::BlueprintSignal { owner, payload: u64, cast_id }` | `src/combat/api/intent.rs:142-146` | Already the correct shape (D008-aligned). S04 replaces the placeholder `u64` payload note in the doc comment with a real closed enum, then routes it through `intent_applier`. |
| `intent_applier` warn-and-drop branch | `src/combat/api/applier.rs:50-52` | The `BlueprintSignal` match arm becomes a real dispatcher writing to `SignalBus` + `CombatEvent`. |
| `BeatRunner` clock-aware step | `src/combat/api/runner.rs:62-453` | `PassiveRunner` borrows the same fire-hook + edge-resolve helpers; the only structural difference is the driving event source (Signal vs cursor advance) and the lifecycle (persistent per-listener, not per-cast). |
| `CombatEventKind::OnKernelTransition { transition: CombatKernelTransition }` | `src/combat/events.rs:121-124` | Already emits kernel transitions to the bus; S04 only adds a new variant `CombatKernelTransition::Blueprint` and emits it from the dispatcher. |
| `CombatKernelTransition` enum | `src/combat/kernel.rs:889-902` | S04 adds `Blueprint { owner: &'static str, payload: SignalPayload }`. Serde derive already in place. Co-exists with the 5 Digimon-specific variants; they migrate in S07–S10. |
| S03 AwaitingCue / resume_cue (D005) | `src/combat/api/runner.rs:39-54, 285-292` | Stable seam: S04 replaces the auto-resume short-circuit with a real `SignalBus`-driven cue-completion path (eventually). For the kitsune_grace fixture, the passive itself has no Presentation, so the stall path is not exercised in S04 — but the design must remain compatible. |
| Existing dispatch pattern | `src/combat/blueprints/mod.rs:99-150` (`DispatchFn`, `dispatch_custom_signal`) | **Don't touch**. The new `Intent::BlueprintSignal` path is parallel; the old `SkillCustomSignal → CombatKernelTransition` path through `resolve_action` is the legacy migration target for S07–S10. |

### What is new

```
src/combat/api/
  signal.rs        ← rewrite from placeholder
  passive_runner.rs ← new file
```

Plus surgical edits to:
- `src/combat/api/intent.rs` — change `BlueprintSignal::payload: u64` → typed payload (see "Signal taxonomy" below).
- `src/combat/api/applier.rs` — implement `BlueprintSignal` arm.
- `src/combat/kernel.rs` — add `CombatKernelTransition::Blueprint { owner, payload }` variant + constructor.
- `src/combat/plugin.rs` — wire `PassiveRunner` registration and `App::finish()` taxonomy validation.
- `src/combat/api/mod.rs` — `pub mod passive_runner;` + re-exports.

### Signal taxonomy (D028 / I8)

Today `Intent::BlueprintSignal::payload` is `u64` (a stub). Per `M021-RESEARCH.md §1.1` the production shape is `Box<dyn Any + Send + Sync>`, but **trait-object payloads serialize poorly** and would block the JSONL round-trip demo. Recommended landing shape for S04:

```rust
// src/combat/api/signal.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Generic cross-blueprint signal. Variant discriminator is `(owner, name)`.
    Blueprint { owner: &'static str, name: &'static str, payload: SignalPayload, cast_id: CastId },
    // S07+ may add framework-level variants (UltimateUsed, OnKill) once the
    // CombatEvent → Signal bridge is built. For S04, Blueprint is sufficient.
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalPayload {
    Empty,
    Amount(i64),
    UnitTarget(UnitId),
}
```

This matches the existing `CustomSignalPayload::{Empty, Amount}` shape in `src/data/skills_ron.rs` (used by the legacy dispatcher), so the JSONL output is structurally familiar and S07's migration can copy values directly. Adding a `UnitTarget` variant covers kitsune_grace's `actor: UnitId` payload need without forcing `Box<dyn Any>`. **The trait-object shape from RESEARCH.md is explicitly demoted for S04**; if/when a third payload kind becomes necessary that doesn't fit a closed enum, revisit (it likely won't — surveyed roster fits these three variants).

Taxonomy registration at `App::finish()`:

```rust
#[derive(Resource, Default)]
pub struct SignalTaxonomy {
    registered: HashSet<(&'static str /*owner*/, &'static str /*name*/)>,
}
// Blueprint `register()` calls `taxonomy.register("renamon", "ult_used_by_ally")`.
// CombatPlugin::finish() panics if any timeline references an unregistered (owner,name).
// (Mirror of validate_timeline_refs at src/combat/api/timeline.rs:118-141.)
```

The `debug_assert mismatch` line in the ROADMAP refers to this: a `Signal::Blueprint{owner, name, …}` sent through `intent_applier` for an unregistered `(owner, name)` triggers `debug_assert!(taxonomy.contains(...), "unregistered signal …")` in `intent_applier`. Release builds drop silently with a `warn!` (matching the existing degraded behavior for unknown intents). This is intentionally less strict than timeline validation because signal emitters might come from data-driven sources where the static-string requirement is bent.

### `PassiveRunner` shape (resolves Gap G-B)

Three options for lifecycle/dispatch were surveyed in `M021-RESEARCH.md §7 G-B`. The literature here is the FINDINGS.md F16 callout and the kitsune_grace canon doc. Recommendation:

**Persistent per-unit-per-FSM, driven by signal-match.**

```rust
// src/combat/api/passive_runner.rs (sketch)
pub struct PassiveRunner {
    timeline: Arc<CompiledTimeline>,
    /// The unit owning this passive (caster for the inner Intent stream).
    owner: UnitId,
    /// Triggering signal predicates: which (owner,name) wakes this runner?
    triggers: Vec<(&'static str, &'static str)>,
    /// Cursor is None when the FSM is in the listening state (entry beat).
    /// When a matching Signal arrives, cursor = Some(entry), then run_to_completion
    /// drains the timeline (emits Intents), then cursor resets to None.
    cursor: Option<BeatId>,
    cast_hit_set: HashSet<UnitId>,
    last_beat_targets: Vec<UnitId>,
}
```

- **One PassiveRunner per (unit, passive timeline).** A Renamon spawn registers one runner for `kitsune_grace`; Gabumon dual-path spawns two (`fur_cloak_fsm` + `twin_core_ice_fsm`) (F17).
- **Driving event:** the runner's `step()` is replaced by `react(&Signal, ...)` which checks the trigger predicate, allocates a fresh `cast_id` (via `CastIdGen`), seeds `caster = self.owner` and (typically) `primary_target = signal.payload as UnitId` for kitsune_grace, then drains the timeline through the **same** `fire_beat` / `next_beat` machinery as `BeatRunner`. Code reuse is high enough that the shared private helpers should move to a `mod common` inside `api/`.
- **No persistent cursor across signals.** Each signal arrival is a fresh cast through a listener-shaped graph: entry beat → optional guards → effect beat(s) → done. The "FSM 3+ nodes" mandate from the kitsune_grace canon (`Dormant → Proc → Resolve → Dormant`) is satisfied by edges, not by holding cursor state between signals. This sidesteps the "should cursor survive across signals" question that complicates the simpler PassiveRunner sketches in the spike — for the v0 roster (6 passives), all reactions are atomic per signal.
- **Bevy system entrypoint:** a new `passive_dispatch_system` runs after `intent_applier` and before the next pipeline step. It drains `SignalBus`'s `VecDeque<Signal>`, iterates registered `PassiveRunner`s, and calls `react()` for each one whose trigger matches. New Intents from passives land back in `IntentQueue`, so the next `intent_applier` tick may itself emit further signals (recursion bounded by depth or the `MAX_HOPS=256` circuit-breaker, see "Risk" below).
- **Storage:** `Resource<PassiveListeners> { runners: Vec<PassiveRunner> }`. Per-unit lookup is by `runner.owner`; the runner itself doesn't need to be a Bevy `Component` because passives are not queried by other systems — only by the dispatcher. This avoids Bevy 0.18 archetype churn when a passive triggers.

### `Intent::BlueprintSignal` dispatcher

The applier match arm becomes:

```rust
// src/combat/api/applier.rs, replacing the warn! branch for BlueprintSignal:
Intent::BlueprintSignal { owner, name, payload, cast_id } => {
    // I8: panic in debug, warn+drop in release if unregistered.
    let taxonomy = world.resource::<SignalTaxonomy>();
    if !taxonomy.contains(owner, name) {
        debug_assert!(false, "unregistered signal: {owner}/{name}");
        warn!("intent_applier: dropping unregistered signal {owner}/{name}");
        continue;
    }
    // D008 final shape (G-F): enqueue on bus AND emit CombatKernelTransition for JSONL.
    world.resource_mut::<SignalBus>().push(Signal::Blueprint {
        owner, name, payload: payload.clone(), cast_id,
    });
    world.resource_mut::<Messages<CombatEvent>>().write(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, payload },
        },
        source: /* derive from cast_id originator */, target: /* same */,
        follow_up_depth: 0,
        cast_id,
    });
}
```

`SetBlueprintState` should land in the same task: it currently falls through to the warn-and-drop branch. The per-unit-per-key state map is `Resource<BlueprintState> { map: HashMap<(UnitId, String), i64> }` (matches MEM001 — the canon write-path). `SkillCtx::blueprint_state(actor, key) -> i64` reads from this map; writes go via `SetBlueprintState` through this same applier. **Don't punt this** — it's load-bearing for the kitsune_grace fixture (the canon's "no state" claim is accurate for that specific passive, but the runner machinery must support it because Gabumon/Dorumon/Tentomon depend on it in S07–S09).

### Ult instant cast (D010)

Today an Ultimate goes through the normal turn pipeline (`src/combat/turn_system/pipeline.rs:570-665`): the attacker's turn ends, `UltimateUsed` is emitted, then `advance_turn_system` picks the next unit. Per D010 and the M021-CONTEXT.md P3 precondition, ult cast must be **separable from the turn advancement** so that downstream listeners (kitsune_grace AdvanceTurn +10%) can modify turn order *after* the ult resolves but *before* the next turn is chosen.

Concrete change scope for S04:

- The existing pipeline already emits `CombatEventKind::UltimateUsed { unit_id }` at three sites (`pipeline.rs:666, 1209, 1508, 1972`). S04 needs to bridge `CombatEventKind::UltimateUsed` → `Signal::Blueprint { owner: "kernel", name: "ult_used", payload: UnitTarget(unit_id) }` so the kitsune_grace `PassiveRunner` can subscribe. This bridge lives in a small `combat_event_to_signal_system` system that runs once per frame, reads CombatEvent, and writes Signals.
- Actual "instant cast" (no turn-end side-effect from the ult itself) is enforced by ensuring the listener's `Intent::AdvanceTurn` lands in the same pipeline tick as the ult's resolution. With the existing pipeline that's already true — there's no separate `TacticalCyclePhase::UltInstant` enum value to add; D010's "precondition" is the bridge from CombatEvent to Signal, not a phase reshape. **Confirm with the planner** before allocating effort to phase enum work. The existing `TacticalCyclePhase` (`src/combat/kernel.rs:9-14`) has 4 variants (Declared/PreApp/Impact/Applied); adding `UltInstant` would be a kernel surface change that I3/D026's "stream parity" doesn't require.

**Recommendation:** treat D010 in S04 as scope-limited to the CombatEvent→Signal bridge for `UltimateUsed`. Defer any `TacticalCyclePhase::UltInstant` enum variant unless a fixture explicitly demonstrates the existing pipeline can't deliver listener `AdvanceTurn` before the next turn-pick. If the kitsune_grace fixture proves clean (passive's `Intent::AdvanceTurn` modifies AV before the next `advance_turn_system` tick), the phase enum stays untouched and the SUMMARY records that D010's intent is satisfied by the bridge, not by phase reshape.

## Recommendation

Decompose into 5 tasks, in this order:

### T01 — Signal taxonomy + `SignalBus::push/drain` + `SignalTaxonomy` registration

**Files:** `src/combat/api/signal.rs` (rewrite), `src/combat/api/mod.rs` (re-exports), `src/combat/plugin.rs` (taxonomy resource + `App::finish` validation), `src/combat/api/intent.rs` (change `BlueprintSignal::payload` type).

Define the closed `Signal` + `SignalPayload` enums. Replace `SignalBus::_pending: u32` with `VecDeque<Signal>` + `push/drain`. Add `SignalTaxonomy: Resource` with `register(owner, name)`. `CombatPlugin::finish` does **not** panic on missing taxonomy entries by itself — the validation runs in the `intent_applier` (debug_assert) when a signal is emitted, mirroring the existing timeline reference validation pattern. Inline unit tests cover: push/drain order; taxonomy register/contains; payload round-trip serialize.

Risk: changing `Intent::BlueprintSignal::payload` from `u64` to `SignalPayload` may ripple into other code. **Verify** by `rg "BlueprintSignal" src/ tests/` — current count is 1 hit (the definition only), so the change is safe.

### T02 — `Intent::BlueprintSignal` dispatcher in `intent_applier` + `CombatKernelTransition::Blueprint`

**Files:** `src/combat/api/applier.rs` (new match arm), `src/combat/kernel.rs` (add `Blueprint { owner, name, payload }` to `CombatKernelTransition`), `src/combat/events.rs` (re-export new variant if needed).

Implement the dispatcher writing to `SignalBus` AND emitting `CombatEventKind::OnKernelTransition { Blueprint{...} }`. `SetBlueprintState` lands the per-unit-per-key write via a new `Resource<BlueprintState>`. The dispatcher source/target field requires a small fixup: BlueprintSignal carries `cast_id` but not source/target — use the **caster** of the originating cast (look it up via a `Resource<CastOriginators>` populated by the BeatRunner at cast entry, OR — simpler — change `Intent::BlueprintSignal` to carry `source: UnitId` explicitly). The latter is cleaner; do that.

Integration test: a hook enqueues `Intent::BlueprintSignal`, `intent_applier` drains, assert (a) `SignalBus` has the entry; (b) `CombatEvent` was written with `OnKernelTransition::Blueprint`; (c) `serde_json::to_string` of the event round-trips through `serde_json::from_str` (JSONL round-trip — the demo line in the roadmap). Pattern: clone `tests/timeline_chain_bolt_port.rs` setup.

### T03 — `PassiveRunner` core + `passive_dispatch_system` Bevy wiring

**Files:** `src/combat/api/passive_runner.rs` (new), `src/combat/api/mod.rs` (mod + re-exports), `src/combat/plugin.rs` (system registration after `intent_applier`).

Build the `PassiveRunner` struct with `react(&Signal, world, regs, mode, pending) -> StepOutcome`. Refactor `BeatRunner::fire_beat` and `BeatRunner::next_beat` into shared `pub(crate) fn` helpers in `api/runner.rs` (or a new `api/runner_common.rs`); both runners call them. The dispatcher system drains `SignalBus`, iterates registered runners, fires those whose triggers match. Bounded recursion: enforce the same `MAX_HOPS=256` circuit-breaker on signal cascades (a passive whose timeline emits another `BlueprintSignal` that re-triggers itself must halt). Reuse `bevy::log::warn!` from S03's circuit-breaker pattern.

Inline tests: trigger predicate matches; non-matching signal is ignored; circuit-breaker fires on signal loop (mirror `runner.rs::loop_with_never_exit_halts_at_circuit_breaker`).

### T04 — Ult-instant bridge: `CombatEvent::UltimateUsed → Signal`

**Files:** new `src/combat/api/event_bridge.rs` (or fold into `passive_runner.rs`), `src/combat/plugin.rs` (system registration before `passive_dispatch_system`).

A small system reads `CombatEvent` looking for `OnSkillCast { … }` / `UltimateUsed { … }` and writes a `Signal::Blueprint { owner: "kernel", name: "ult_used", payload: UnitTarget(unit_id), cast_id: ROOT }` to the bus. Register `("kernel", "ult_used")` in the taxonomy at plugin build.

This is the cheapest path to make the kitsune_grace fixture trigger without restructuring the existing turn pipeline. **Do not touch `TacticalCyclePhase`** in S04 unless a follow-up grill from the planner shows the existing pipeline can't deliver listener AdvanceTurn before the next turn-pick (the `flush_ult_gain_system` pattern at `src/combat/ultimate.rs:148-156` shows the precedent for inter-system event staging that should work).

### T05 — Integration test: Renamon `kitsune_grace` end-to-end + JSONL round-trip

**Files:** `tests/passive_kitsune_grace.rs` (new), possibly a tiny `tests/jsonl_blueprint_roundtrip.rs` if T02's inline JSONL assertion is preferred elsewhere.

Spawn Renamon (Ally) + Patamon (Ally) + Enemy. Manually inject `CombatEvent::UltimateUsed { unit_id: patamon_id }` (or drive Patamon to actually cast its ult — the manual injection is faster and equally diagnostic). Run one `app.update()`. Assert:
- `kitsune_grace` PassiveRunner fired (observable via a `BlueprintState` counter increment or a recorded test-only sentinel).
- Renamon's `ActionValue` advanced by 10% of MAX_AV (the canonical effect).
- Self-cast guard: inject a second `UltimateUsed { unit_id: renamon_id }`, assert no AV gain.
- Cross-team guard: inject `UltimateUsed { unit_id: enemy_id }`, assert no AV gain.
- JSONL: the `OnKernelTransition::Blueprint` event round-trips through `serde_json` to itself (BEVYROGUE_JSONL behavior — `src/combat/jsonl_logger.rs:7-18`).

Use the canonical fixture pattern (`tests/timeline_chain_bolt_port.rs:35-57` for setup_app/spawn_unit). Spawn the Renamon with a registered PassiveRunner via a small `register_renamon_passives(&mut app)` helper in the test file — the real blueprint module's `register()` doesn't exist yet (that's S08–S10).

## Implementation order and seams

1. **T01 first** — pure data layer, no system wiring. Unblocks everything downstream. Risk: `Intent::BlueprintSignal::payload` type change ripples (verified single hit; should be safe).
2. **T02 in parallel with T03** — independent files (`applier.rs` vs `passive_runner.rs`). T02 lands the JSONL round-trip demo; T03 lands the runner. Either can ship first.
3. **T04 needs T01 + T03** — bridges existing event bus into the new signal layer.
4. **T05 is the integration capstone** — needs all of the above.

## Don't Hand-Roll

- **Timeline validation pattern** — copy `validate_timeline_refs` (`src/combat/api/timeline.rs:118-141`). Signal taxonomy validation mirrors the same error-collection + panic-at-finish shape, just keyed on `(owner, name)` tuples.
- **Test scaffolding** — clone `tests/timeline_chain_bolt_port.rs:35-57`'s `setup_app` and `spawn_unit`. The pattern is already canon for S02/S03.
- **Circuit-breaker log signal** — reuse the S03 pattern at `src/combat/api/runner.rs:158-164` for signal cascades.
- **JSONL output** — `src/combat/jsonl_logger.rs:7-18` already serializes `CombatEvent`; the new `CombatKernelTransition::Blueprint` variant flows through for free via `#[derive(Serialize)]`.

## Risks and watch-outs

- **`Intent::BlueprintSignal` lacks a `source` field today** (`intent.rs:142-146`). Add it in T02; the alternative (looking up source by cast_id) requires a new `Resource<CastOriginators>` map that S04 doesn't need otherwise. Verify the constructor sites — there are none yet because `BlueprintSignal` isn't enqueued anywhere — so adding the field is structurally safe.
- **Signal cascade depth.** A passive emitting `BlueprintSignal` that triggers another passive that re-triggers the first → infinite loop. Use `MAX_HOPS=256` on the dispatcher's per-frame drain count, log a `warn!` with the loop's signal chain. Test it.
- **`Box<dyn Any>` is explicitly rejected** for payloads (RESEARCH.md §1.1 vs serialization needs). Use the closed `SignalPayload` enum. If a future passive needs a payload kind that doesn't fit `{Empty, Amount, UnitTarget}`, that's an additive variant — not a redesign.
- **D010 phase reshape is *not* in S04 scope** unless the planner can demonstrate via fixture that the existing pipeline can't deliver listener `AdvanceTurn` before the next turn-pick. Avoid speculative kernel-surface changes; let the kitsune_grace fixture force the issue if it exists.
- **`debug_assert! mismatch` semantics:** the ROADMAP demo bullet refers to taxonomy validation, not to a `panic!`. Use `debug_assert!` so release builds degrade gracefully (warn + drop), matching the existing `intent_applier` warn-and-drop pattern for unknown intents (`applier.rs:50-52`).
- **Avoid touching the legacy `src/combat/blueprints/<x>.rs` dispatchers.** The new `Intent::BlueprintSignal` path is parallel to the existing `SkillCustomSignal → dispatch_custom_signal` path (`blueprints/mod.rs:137-150`). Migration of the 5 Digimon-specific transition variants to the new path is S07–S10 scope.
- **`SignalBus` Resource conflict with `Resource<PassiveListeners>`** — keep them separate Resources. The bus is the queue; the listeners are the registry. Co-locating them creates a borrow conflict because `passive_dispatch_system` reads listeners while draining the bus.

## Skill discovery

Relevant pre-installed skills (no install needed):

- `bevy` — Bevy 0.18 ECS conventions; consult when wiring the `passive_dispatch_system` between `intent_applier` and the next pipeline step. Specifically: Bevy 0.18 `Messages<T>` (not `Events<T>`) — the codebase already migrated, so follow the existing pattern in `applier.rs`.
- `rust-best-practices` — `Arc<CompiledTimeline>` lifecycle decisions; `&'static str` vs `String` interning trade-offs (RESEARCH.md G-D defers interning to S11 — confirm).
- `rust-testing` — TDD pattern for the 5-test kitsune_grace integration suite.
- `tdd` — particularly useful for T05 where the 4 canonical guards (ally non-self / self / enemy / dead owner from the spike sketch) are the natural red-green ladder.
- `verify-before-complete` — mandatory before marking T05 done given the high risk; freshly run `cargo test --test passive_kitsune_grace` and `cargo test` output must be in the SUMMARY.

## Active requirements affecting delivery

(Skipped — REQUIREMENTS.md was not preloaded as Active by the dispatch, and the milestone-level constraints in M021-CONTEXT are already captured above as P001 / I1 / I2 / I3 / I5 / I6 / I8.)

## Sources

- `src/combat/api/signal.rs` (placeholder), `src/combat/api/intent.rs:142-155`, `src/combat/api/applier.rs:33-55`, `src/combat/api/runner.rs:62-453`, `src/combat/api/timeline.rs:118-141`, `src/combat/plugin.rs`, `src/combat/events.rs:121-124`, `src/combat/kernel.rs:889-902`, `src/combat/blueprints/mod.rs:99-150`, `src/combat/blueprints/renamon.rs`, `src/combat/jsonl_logger.rs`, `src/combat/turn_system/pipeline.rs:570-665`, `src/combat/ultimate.rs:148-156`
- `.gsd/milestones/M021/M021-CONTEXT.md` (F5, P3, D008, D010, D028, I8, G-B, G-F)
- `.gsd/milestones/M021/M021-RESEARCH.md` §1.1 SignalBus, §3 Passive line, §7 G-B/G-F
- `.gsd/milestones/M021/slices/S03/S03-SUMMARY.md` (AwaitingCue/resume_cue seam from D005)
- `.gsd/workflows/spikes/M021-timeline-fsm/FINDINGS.md` F3 (D028 load-bearing), F16 (PassiveRunner shape), F17 (multi-FSM blueprint)
- `.gsd/spikes/spike-blueprint-api/sketches/kitsune_grace.rs` (canonical reactive shape)
- `docs/future_design_draft/digimon/renamon/04_passive_kitsune_grace.md` §1.5 (FSM topology), §2 (Blueprint contract)
- `.gsd/DECISIONS.md` D005 (S03/S04 clock seam), D008 (Blueprint variant final shape)
- MEM001 (D034 canonical write-path — `Intent::SetBlueprintState`)

Slice S04 researched.