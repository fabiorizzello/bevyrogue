---
estimated_steps: 13
estimated_files: 4
skills_used: []
---

# T01: Signal/SignalPayload enums + SignalBus push/drain + SignalTaxonomy + BlueprintSignal payload type change

Why: SignalBus is a placeholder Resource (src/combat/api/signal.rs); the typed Signal enum + queue + taxonomy is the data-layer foundation every downstream task depends on. Without it, the dispatcher (T02), runner (T03), and bridge (T04) cannot compile.

Do:
1. Rewrite src/combat/api/signal.rs:
   - Define `pub enum Signal { Blueprint { owner: &'static str, name: &'static str, payload: SignalPayload, cast_id: CastId } }` with derives `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` (import serde via existing crate usage in kernel.rs).
   - Define `pub enum SignalPayload { Empty, Amount(i64), UnitTarget(UnitId) }` with the same derives.
   - Replace `SignalBus { _pending: u32 }` with `SignalBus { queue: VecDeque<Signal> }` and impl `pub fn push(&mut self, sig: Signal)` and `pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, Signal>`.
   - Define `#[derive(Resource, Default)] pub struct SignalTaxonomy { registered: HashSet<(&'static str, &'static str)> }` with `pub fn register(&mut self, owner, name)` and `pub fn contains(&self, owner, name) -> bool`.
2. Update src/combat/api/intent.rs:
   - Change `Intent::BlueprintSignal { owner: UnitId, payload: u64, cast_id }` → `Intent::BlueprintSignal { source: UnitId, owner: &'static str, name: &'static str, payload: SignalPayload, cast_id }` (per research risk note: add explicit `source` to avoid CastOriginators map; rename old `owner: UnitId` to `source` since the new `owner` is the blueprint-static-string).
3. Update src/combat/api/mod.rs: add `pub use signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy};`.
4. Update src/combat/plugin.rs: `.init_resource::<SignalTaxonomy>()`.
5. Inline unit tests in signal.rs (in a `#[cfg(test)] mod tests` block): push_drain_order, taxonomy_register_contains, signal_payload_round_trip via serde_json::to_string/from_str.

Done-when: `cargo check` (headless + windowed) clean; `cargo test --lib combat::api::signal` (or the workspace `cargo test`) shows the 3 new unit tests green; `rg "BlueprintSignal" src/ tests/` is the expected single usage site (intent.rs definition) — if other call sites surface, repair them.

## Inputs

- `src/combat/api/signal.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `src/combat/api/timeline.rs`
- `src/combat/kernel.rs`
- `.gsd/milestones/M021/slices/S04/S04-RESEARCH.md`

## Expected Output

- `src/combat/api/signal.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`

## Verification

cargo check && cargo check --features windowed && cargo test --lib -- combat::api::signal && rg "BlueprintSignal" src/ tests/

## Observability Impact

No runtime observability changes; data-layer only. Serde derives on Signal/SignalPayload enable JSONL serialization downstream (T02).
