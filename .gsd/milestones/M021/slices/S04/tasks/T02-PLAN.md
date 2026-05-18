---
estimated_steps: 11
estimated_files: 6
skills_used: []
---

# T02: Intent::BlueprintSignal + SetBlueprintState dispatcher in intent_applier; CombatKernelTransition::Blueprint variant

Why: The applier currently warn-and-drops BlueprintSignal (src/combat/api/applier.rs:50-52). The dispatch path must enqueue on SignalBus AND emit a kernel-transition CombatEvent for JSONL round-trip (D008 final shape, G-F). SetBlueprintState similarly needs a real write target (Resource<BlueprintState>, MEM001) since Gabumon/Dorumon/Tentomon depend on it in S07–S09 even though kitsune_grace does not.

Do:
1. Add `Blueprint { owner: &'static str, name: &'static str, payload: SignalPayload }` to `CombatKernelTransition` in src/combat/kernel.rs around line 890. Verify serde derives propagate (variant inherits parent derives).
2. Add `#[derive(Resource, Default)] pub struct BlueprintState { pub map: HashMap<(UnitId, String), i64> }` (new file src/combat/api/blueprint_state.rs OR inline in applier.rs — pick blueprint_state.rs for separation; expose via api/mod.rs). Init the Resource in plugin.rs.
3. In src/combat/api/applier.rs:
   - Replace the catch-all `other =>` arm with explicit arms for `BlueprintSignal { source, owner, name, payload, cast_id }` and `SetBlueprintState { actor, key, value, cast_id }`.
   - BlueprintSignal arm: read `world.resource::<SignalTaxonomy>()`; if `!contains(owner,name)`, `debug_assert!(false, "unregistered signal: {owner}/{name}"); log::warn!; continue;`. Else: push `Signal::Blueprint{owner,name,payload:payload.clone(),cast_id}` to `SignalBus`; emit `CombatEvent { kind: CombatEventKind::OnKernelTransition { transition: CombatKernelTransition::Blueprint{owner,name,payload} }, source, target: source, follow_up_depth: 0, cast_id }`.
   - SetBlueprintState arm: `world.resource_mut::<BlueprintState>().map.insert((actor, key), value);`. Optionally emit a CombatEvent for observability (deferred; not required for kitsune_grace).
   - Keep remaining variants under a final `other =>` warn arm (Reject, KoUnit, BeatGuard, AdvanceTurn etc still pending).
4. Add integration test tests/blueprint_signal_dispatcher.rs: build minimal App with CombatPlugin; register `("test", "sig")` in SignalTaxonomy; push `Intent::BlueprintSignal { source: UnitId(0), owner: "test", name: "sig", payload: SignalPayload::Amount(42), cast_id: CastId(1) }` to IntentQueue; run `intent_applier` once via `world.run_system_once`. Assert: SignalBus.drain() yields one matching Signal; Messages<CombatEvent> contains one OnKernelTransition::Blueprint event with payload Amount(42); serde_json::to_string then from_str on the CombatEvent round-trips structurally.

Done-when: cargo test --test blueprint_signal_dispatcher green; cargo check headless+windowed clean; no new warnings; debug_assert path is exercised by a second test that pushes an unregistered signal in a #[should_panic] block.

## Inputs

- `src/combat/api/applier.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/mod.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/plugin.rs`
- `tests/intent_applier_canary.rs`
- `.gsd/milestones/M021/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/api/applier.rs`
- `src/combat/api/blueprint_state.rs`
- `src/combat/api/mod.rs`
- `src/combat/kernel.rs`
- `src/combat/plugin.rs`
- `tests/blueprint_signal_dispatcher.rs`

## Verification

cargo test --test blueprint_signal_dispatcher && cargo check && cargo check --features windowed

## Observability Impact

Adds CombatKernelTransition::Blueprint variant; emitted via existing CombatEventKind::OnKernelTransition path. JSONL logger (src/combat/jsonl_logger.rs) automatically picks it up via serde Serialize derive. debug_assert! provides 3am-debug visibility on unregistered signal taxonomy.
