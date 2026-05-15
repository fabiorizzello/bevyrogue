# S04: SignalBus + PassiveRunner + Ult instant + Intent::BlueprintSignal dispatcher

**Goal:** Lift SignalBus + PassiveRunner + Intent::BlueprintSignal dispatcher out of placeholder into a working reactive layer, and bridge CombatEvent::UltimateUsed → Signal. Demo: Renamon kitsune_grace passive triggers on ally ult, advancing AV by +10%; JSONL CombatKernelTransition::Blueprint round-trip; debug_assert! mismatch on unregistered signal.
**Demo:** Renamon kitsune_grace verde; JSONL Blueprint round-trip; debug_assert mismatch.

## Must-Haves

- Signal + SignalPayload closed enums defined; SignalBus carries VecDeque<Signal> with push/drain.
- SignalTaxonomy resource registers (owner,name) tuples; intent_applier debug_asserts on unregistered emission.
- Intent::BlueprintSignal arm in intent_applier enqueues on SignalBus AND emits CombatEvent::OnKernelTransition { CombatKernelTransition::Blueprint{owner,name,payload} } with cast_id propagated.
- Intent::SetBlueprintState arm writes Resource<BlueprintState> (HashMap<(UnitId,String),i64>).
- PassiveRunner reacts to matching Signals via shared fire_beat/next_beat helpers; circuit-breaker MAX_HOPS=256 on signal cascade.
- passive_dispatch_system runs after intent_applier; combat_event_to_signal_system bridges UltimateUsed → Signal::Blueprint{owner:"kernel",name:"ult_used",payload:UnitTarget(unit_id)}.
- Integration test tests/passive_kitsune_grace.rs proves the demo end-to-end (ally non-self triggers; self-ult and enemy-ult do not; AV advances 10% MAX_AV).
- JSONL round-trip: serde_json::to_string then from_str on the OnKernelTransition::Blueprint event reproduces it.
- cargo check (headless + windowed) and cargo test green; no new warnings.
- P001 guard intact: rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/ → 0 hits.

## Proof Level

- This slice proves: High — multiple new interlocking primitives (SignalBus + PassiveRunner + dispatcher + bridge); demonstrated by a 4-assertion integration test (kitsune_grace) plus JSONL round-trip plus inline unit tests covering taxonomy register/contains, push/drain order, debug_assert mismatch (debug-only), and signal-cascade circuit-breaker.

## Integration Closure

Single integration capstone test (tests/passive_kitsune_grace.rs) drives the full pipeline: spawn Renamon/Patamon/Enemy → inject CombatEvent::UltimateUsed → app.update() → assert (a) SignalBus drained, (b) PassiveRunner fired (BlueprintState counter or sentinel intent), (c) Renamon AV advanced by 10% of MAX_AV, (d) negative guards (self-ult, enemy-ult, no fire), (e) serde_json round-trip on the emitted OnKernelTransition event. Inline T02 test (tests/blueprint_signal_dispatcher.rs or inline in applier.rs unit tests) covers the dispatcher-only path (SignalBus enqueue + CombatEvent emission) without the runner, isolating the JSONL round-trip demo.

## Verification

- Reuse S03 circuit-breaker pattern (bevy::log::warn! with cast_id + signal chain) for signal-cascade halt.
- intent_applier emits CombatEvent::OnKernelTransition for every dispatched BlueprintSignal; JSONL logger picks it up via existing serde Serialize derive (no logger changes).
- debug_assert! on unregistered (owner,name) fires panic in debug builds; release degrades to warn!+drop matching existing applier policy.
- No new tracing spans; all visibility flows through CombatEvent (P001-aligned).

## Tasks

- [x] **T01: Signal/SignalPayload enums + SignalBus push/drain + SignalTaxonomy + BlueprintSignal payload type change** `est:M`
  Why: SignalBus is a placeholder Resource (src/combat/api/signal.rs); the typed Signal enum + queue + taxonomy is the data-layer foundation every downstream task depends on. Without it, the dispatcher (T02), runner (T03), and bridge (T04) cannot compile.
  - Files: `src/combat/api/signal.rs`, `src/combat/api/intent.rs`, `src/combat/api/mod.rs`, `src/combat/plugin.rs`
  - Verify: cargo check && cargo check --features windowed && cargo test --lib -- combat::api::signal && rg "BlueprintSignal" src/ tests/

- [x] **T02: Intent::BlueprintSignal + SetBlueprintState dispatcher in intent_applier; CombatKernelTransition::Blueprint variant** `est:L`
  Why: The applier currently warn-and-drops BlueprintSignal (src/combat/api/applier.rs:50-52). The dispatch path must enqueue on SignalBus AND emit a kernel-transition CombatEvent for JSONL round-trip (D008 final shape, G-F). SetBlueprintState similarly needs a real write target (Resource<BlueprintState>, MEM001) since Gabumon/Dorumon/Tentomon depend on it in S07–S09 even though kitsune_grace does not.
  - Files: `src/combat/api/applier.rs`, `src/combat/api/blueprint_state.rs`, `src/combat/api/mod.rs`, `src/combat/kernel.rs`, `src/combat/plugin.rs`, `tests/blueprint_signal_dispatcher.rs`
  - Verify: cargo test --test blueprint_signal_dispatcher && cargo check && cargo check --features windowed

- [x] **T03: PassiveRunner + passive_dispatch_system; shared fire_beat/next_beat helpers; signal-cascade circuit-breaker** `est:L`
  Why: kitsune_grace and the 5 other passives are reactive timelines (no cast of their own); they need a runner whose driving event is `Signal` arrival, not cursor advance. Code reuse with BeatRunner is high enough that the shared `fire_beat`/`next_beat`/edge-resolve helpers should be lifted to `pub(crate)` visibility on a shared module.
  - Files: `src/combat/api/passive_runner.rs`, `src/combat/api/runner.rs`, `src/combat/api/mod.rs`, `src/combat/plugin.rs`
  - Verify: cargo test --lib -- combat::api::passive_runner && cargo test --test timeline_mode_parity --test timeline_two_clock_parity --test timeline_circuit_breaker && cargo check && cargo check --features windowed

- [x] **T04: CombatEvent → Signal bridge for UltimateUsed (D010 precondition, scope-limited)** `est:S`
  Why: Per D010, ult cast must be observable to listener passives (e.g. kitsune_grace) without restructuring the existing turn pipeline. The cheapest path is a small system that reads CombatEvent::UltimateUsed and writes Signal::Blueprint{owner:"kernel",name:"ult_used",payload:UnitTarget(unit_id)} to SignalBus. Research recommendation explicitly scope-limits D010 to this bridge — do NOT touch TacticalCyclePhase.
  - Files: `src/combat/api/event_bridge.rs`, `src/combat/api/mod.rs`, `src/combat/plugin.rs`
  - Verify: cargo test --lib -- combat::api::event_bridge && cargo check && cargo check --features windowed && rg "TacticalCyclePhase::UltInstant" src/

- [x] **T05: Integration capstone: Renamon kitsune_grace passive end-to-end + JSONL round-trip + negative guards** `est:L`
  Why: This is the slice demo gate (per ROADMAP: 'Renamon kitsune_grace verde + JSONL Blueprint round-trip + debug_assert mismatch'). It exercises T01–T04 together via the canonical reactive passive — the cleanest first proof for the whole stack.
  - Files: `tests/passive_kitsune_grace.rs`
  - Verify: cargo test --test passive_kitsune_grace && cargo test && cargo check --features windowed && rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/

## Files Likely Touched

- src/combat/api/signal.rs
- src/combat/api/intent.rs
- src/combat/api/mod.rs
- src/combat/plugin.rs
- src/combat/api/applier.rs
- src/combat/api/blueprint_state.rs
- src/combat/kernel.rs
- tests/blueprint_signal_dispatcher.rs
- src/combat/api/passive_runner.rs
- src/combat/api/runner.rs
- src/combat/api/event_bridge.rs
- tests/passive_kitsune_grace.rs
