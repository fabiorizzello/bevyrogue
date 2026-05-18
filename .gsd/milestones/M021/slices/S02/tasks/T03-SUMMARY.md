---
id: T03
parent: S02
milestone: M021
key_files:
  - src/combat/api/runner.rs
  - src/combat/api/mod.rs
key_decisions:
  - Arc&lt;CompiledTimeline&gt; chosen over &'static for test ergonomics — avoids Box::leak while matching spike's intent.
  - world: &mut World in public API (step/run_to_completion), passed as &World to SkillCtx::new — enables future exclusive-world use without API churn.
  - Dummy VecDeque for predicate eval — predicates take &SkillCtx and should not enqueue; discarding any stray intents is the safest default.
  - Arc::clone in next_beat to resolve borrow conflict between iterating edges and calling &mut self eval_predicate.
duration: 
verification_result: passed
completed_at: 2026-05-15T07:56:54.974Z
blocker_discovered: false
---

# T03: Implemented BeatRunner FSM engine with single-level LoopFrame, StepOutcome, and 3 passing inline unit tests.

**Implemented BeatRunner FSM engine with single-level LoopFrame, StepOutcome, and 3 passing inline unit tests.**

## What Happened

Ported `BeatRunner` from the spike (`lib.rs:776–1089`) into `src/combat/api/runner.rs`, adapting for the live kernel's API surface.

Key adaptations from spike:
- Removed the `'a` lifetime parameter — `BeatRunner` is now a plain struct. `world: &mut World` and `regs: &ExtRegistries` are passed per-call to `step` rather than stored, which avoids lifetime coupling and makes the runner own its mutable state cleanly.
- Used `Arc<CompiledTimeline>` instead of `&'a` reference — allows test construction without `Box::leak` and production use from a shared registry.
- `loop_stack: Vec<LoopFrame>` (depth ≤ 1, guarded by `debug_assert!`) replaces spike's `Option<LoopFrame>`, matching the task plan shape.
- Replaced spike's `signals: &mut SignalBus` + `Clock` with headless-only auto-advance (no `AdvanceMode::OnSignal` branch for S02).
- `StepOutcome { Advanced, LoopExited, Done, Halted }` replaces spike's `bool` return; `Halted` fires at `MAX_HOPS = 256`.
- F6 fix carried: after each hook fires, `cast_hit_set` is updated by scanning newly appended `Intent::DealDamage` entries in `pending` via range index (no extra allocation).
- F1 fallback-edge rule documented in `/// ` comment on `next_beat`: edges tested in declaration order, first passing gate wins.
- Predicate evaluation uses a dummy `VecDeque` to prevent accidental state mutation (predicates receive `&SkillCtx` not `&mut`).
- `Arc::clone` trick in `next_beat` avoids the borrow conflict between iterating `self.timeline.edges` and calling `&mut self.eval_predicate` inside the loop.

Three inline unit tests cover the task plan's required scenarios: (a) linear 2-beat Cast→Impact both fires hooks and returns Done; (b) Loop with `always_true` exits after one body pass and reaches Done; (c) Loop with `never` predicate hits MAX_HOPS circuit-breaker and returns Halted.

## Verification

cargo check clean (123 pre-existing warnings, 0 errors). cargo test --lib combat::api::runner:: — 3/3 tests pass. rg confirmed pub struct BeatRunner and LoopFrame present in runner.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 2640ms |
| 2 | `cargo test --lib combat::api::runner::` | 0 | 3 passed | 3050ms |
| 3 | `rg 'pub struct BeatRunner' src/combat/api/runner.rs && rg 'LoopFrame' src/combat/api/runner.rs` | 0 | pass | 50ms |

## Deviations

Clock/SignalBus not wired — S02 is headless-only so all beats auto-advance. Spike's AdvanceMode branch omitted intentionally; S04 will add it when Windowed support arrives.

## Known Issues

none

## Files Created/Modified

- `src/combat/api/runner.rs`
- `src/combat/api/mod.rs`
