---
estimated_steps: 13
estimated_files: 3
skills_used: []
---

# T04: CombatEvent → Signal bridge for UltimateUsed (D010 precondition, scope-limited)

Why: Per D010, ult cast must be observable to listener passives (e.g. kitsune_grace) without restructuring the existing turn pipeline. The cheapest path is a small system that reads CombatEvent::UltimateUsed and writes Signal::Blueprint{owner:"kernel",name:"ult_used",payload:UnitTarget(unit_id)} to SignalBus. Research recommendation explicitly scope-limits D010 to this bridge — do NOT touch TacticalCyclePhase.

Do:
1. Create src/combat/api/event_bridge.rs:
   - `pub fn combat_event_to_signal_system(mut events: MessageReader<CombatEvent>, mut bus: ResMut<SignalBus>)`.
   - For each CombatEvent: match on kind:
     - `CombatEventKind::UltimateUsed { unit_id }` → push `Signal::Blueprint { owner: "kernel", name: "ult_used", payload: SignalPayload::UnitTarget(*unit_id), cast_id: cast_id_from_event }` (use the event's existing cast_id field).
     - (S07+ may add OnKill, OnSkillCast etc.)
2. Wire in src/combat/plugin.rs:
   - `.add_systems(Update, combat_event_to_signal_system.after(intent_applier).before(passive_dispatch_system))`.
   - At plugin build, register kernel-side signals in SignalTaxonomy: `taxonomy.register("kernel", "ult_used")`.
3. Update src/combat/api/mod.rs: `pub mod event_bridge; pub use event_bridge::combat_event_to_signal_system;`.
4. Inline unit test in event_bridge.rs: build App with CombatPlugin; manually write CombatEvent::UltimateUsed; run one update; assert SignalBus contains exactly one Signal::Blueprint with owner="kernel", name="ult_used", payload=UnitTarget(expected).

Done-when: cargo test --lib -- combat::api::event_bridge green; cargo check headless+windowed clean. Verify via `rg "TacticalCyclePhase::UltInstant" src/` returns 0 hits (we did NOT introduce a new phase variant — D010 satisfied by bridge only).

## Inputs

- `src/combat/events.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `src/combat/ultimate.rs`
- `.gsd/milestones/M021/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/api/event_bridge.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`

## Verification

cargo test --lib -- combat::api::event_bridge && cargo check && cargo check --features windowed && rg "TacticalCyclePhase::UltInstant" src/

## Observability Impact

Adds one Bevy system reading CombatEvent in Update schedule; no new event types. Signals emitted by the bridge flow through the same SignalBus → PassiveRunner → IntentQueue path as blueprint-emitted signals, so JSONL coverage from T02 transitively covers ult-driven passive activations.
