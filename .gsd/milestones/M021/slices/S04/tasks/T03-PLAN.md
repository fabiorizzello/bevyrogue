---
estimated_steps: 16
estimated_files: 4
skills_used: []
---

# T03: PassiveRunner + passive_dispatch_system; shared fire_beat/next_beat helpers; signal-cascade circuit-breaker

Why: kitsune_grace and the 5 other passives are reactive timelines (no cast of their own); they need a runner whose driving event is `Signal` arrival, not cursor advance. Code reuse with BeatRunner is high enough that the shared `fire_beat`/`next_beat`/edge-resolve helpers should be lifted to `pub(crate)` visibility on a shared module.

Do:
1. Create src/combat/api/passive_runner.rs:
   - `pub struct PassiveRunner { timeline: Arc<CompiledTimeline>, owner: UnitId, triggers: Vec<(&'static str, &'static str)>, cast_hit_set: HashSet<UnitId>, last_beat_targets: Vec<UnitId> }` (no persistent cursor across signals — each signal arrival is a fresh atomic cast through the listener-shaped graph; see RESEARCH §PassiveRunner shape).
   - `pub fn new(timeline: Arc<CompiledTimeline>, owner: UnitId, triggers: Vec<(&'static str,&'static str)>) -> Self`.
   - `pub fn react(&mut self, signal: &Signal, world: &World, regs: &ExtRegistries, mode: SkillCtxMode, pending: &mut VecDeque<Intent>, cast_id_gen: &mut CastIdGen)`: check trigger predicate match; if match, allocate fresh CastId, seed caster=self.owner and primary_target derived from payload (UnitTarget→that unit, else self.owner), then drive timeline to completion via the shared helpers (mirroring BeatRunner::run_to_completion structure).
   - Reuse fire_beat/next_beat: refactor in src/combat/api/runner.rs to expose them as `pub(crate) fn` (or extract to a new module `src/combat/api/runner_common.rs`). Tests in runner.rs must still pass unchanged.
2. `#[derive(Resource, Default)] pub struct PassiveListeners { pub runners: Vec<PassiveRunner> }` (kept as a sibling Resource to SignalBus to avoid borrow conflicts during dispatch).
3. `pub fn passive_dispatch_system(world: &mut World)` — exclusive system:
   - Drain SignalBus into a local Vec<Signal>.
   - For each signal: iterate PassiveListeners.runners and call react() on those whose triggers match. New Intents land in IntentQueue (which intent_applier will drain next).
   - Enforce MAX_HOPS=256 on per-frame react() invocations to bound signal cascades; on hit emit `log::warn!` with the offending signal chain (mirror src/combat/api/runner.rs:158-164 pattern).
4. Wire in src/combat/plugin.rs: `.init_resource::<PassiveListeners>().add_systems(Update, passive_dispatch_system.after(intent_applier))`.
5. Update src/combat/api/mod.rs: `pub mod passive_runner; pub use passive_runner::{PassiveRunner, PassiveListeners, passive_dispatch_system};`.
6. Inline unit tests in passive_runner.rs: trigger predicate matches; non-matching signal ignored; circuit-breaker halts on signal loop (timeline emits BlueprintSignal that re-triggers itself).

Done-when: cargo test --lib -- combat::api::passive_runner green; cargo check headless+windowed clean. All S02/S03 BeatRunner tests still pass (verify with `cargo test --test timeline_mode_parity --test timeline_two_clock_parity --test timeline_circuit_breaker`).

## Inputs

- `src/combat/api/runner.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `.gsd/milestones/M021/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/api/passive_runner.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`

## Verification

cargo test --lib -- combat::api::passive_runner && cargo test --test timeline_mode_parity --test timeline_two_clock_parity --test timeline_circuit_breaker && cargo check && cargo check --features windowed

## Observability Impact

MAX_HOPS=256 signal-cascade circuit-breaker emits bevy::log::warn! with the signal chain — closes a 3am-debug hazard for infinite passive-loops. Reuses the S03 pattern.
