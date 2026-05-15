---
estimated_steps: 14
estimated_files: 2
skills_used: []
---

# T03: BeatRunner with single-level LoopFrame (no test yet, must compile and be unit-tested)

Why: `BeatRunner` is the FSM engine — the slice's largest mechanical port from the spike. Keeping it in its own task isolates the borrow/lifetime work (F7) and lets T04 focus on the integration scenarios.

Do:
1. Create `src/combat/api/runner.rs`. Port from spike `lib.rs:776–1089`:
   - `LoopFrame { body_index: usize, hop_index: u32 }` plus stack semantics (S02 only needs depth ≤ 1 — assert in debug if a Loop body itself contains a Loop until S05 lifts the restriction).
   - `BeatRunner { timeline: &'static CompiledTimeline (or Arc — match spike), cursor: BeatId, loop_stack: Vec<LoopFrame>, cast_hit_set: HashSet<UnitId>, last_beat_targets: Vec<UnitId> }`.
   - `BeatRunner::new(timeline, cast_id, caster, primary_target)`.
   - `BeatRunner::step(&mut self, world: &mut World, regs: &ExtRegistries, mode: SkillCtxMode, pending: &mut VecDeque<Intent>) -> StepOutcome` where `StepOutcome` is `Advanced | LoopExited | Done | Halted`.
   - `BeatRunner::run_to_completion(...)` driver.
   - Edge selection: `next_from(cursor, &regs.predicates, &event, &ctx)` picks the first edge whose gate (a) is None, or (b) resolves to true. Document the F1 fallback-edge rule in a `///` comment.
   - Loop entry/exit: on entering `BeatKind::Loop { body, exit_when }`, push a `LoopFrame { body_index: 0, hop_index: 0 }`; after each body completion, evaluate `regs.predicates.get(exit_when)` and either bump `hop_index` & reset `body_index` to walk the body again, or pop the frame and advance from the Loop beat. Carry F6 fix: `last_beat_targets` updated after every hook fires so gate predicates see running targets, not the empty `BeatEvent` default.
   - On every hook fire, also fold any `Intent::DealDamage { target, .. }` enqueued into `cast_hit_set` (read back from `pending` tail or via a thin helper — match spike's approach).
2. Add `pub mod runner;` to `src/combat/api/mod.rs` and re-export `BeatRunner`, `LoopFrame`, `StepOutcome`.
3. Inline unit tests in `runner.rs`: (a) linear 2-beat timeline (Cast→Impact) runs Cast, then Impact, then Done; (b) Loop with `exit_when="always_true"` exits after one body pass; (c) Loop with `exit_when="never"` halts at the circuit-breaker @256 (per I-spec; for S02 a smaller `MAX_HOPS = 256` constant is sufficient — match spike).

Done-when: `cargo check` clean; `cargo test --lib combat::api::runner::` green; no Loop-in-Loop allowed (debug_assert acceptable); `BeatRunner::step` accepts `SkillCtxMode` so S03 can flip DryRun without API churn.

## Inputs

- `.gsd/workflows/spikes/M021-timeline-fsm/src/lib.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/intent.rs`

## Expected Output

- `src/combat/api/runner.rs`
- `src/combat/api/mod.rs`

## Verification

cargo check && cargo test --lib combat::api::runner:: && rg 'pub struct BeatRunner' src/combat/api/runner.rs && rg 'LoopFrame' src/combat/api/runner.rs
