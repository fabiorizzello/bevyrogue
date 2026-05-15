---
id: S03
parent: M021
milestone: M021
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - AwaitingCue latch uses two fields (awaiting_cue: Option<BeatId> + cue_just_resumed: bool) so the loop-body path (body_cursor advance) is handled uniformly with the linear path without duplication
  - Global stall gate placed before loop_stack branch — one check covers both linear and loop-body step paths
  - run_to_completion auto-resumes AwaitingCue so S02 batch-drive semantics are preserved unchanged under Windowed clock
  - World::try_query::<&T>() (takes &self) used in predicate closures for immutable component reads — World::query takes &mut self and is unusable with &World from SkillCtx
  - normalize() strips cast_id from DealDamage format!() string to enable structural cross-mode comparison (each mode run uses a distinct CastId)
  - Exact circuit-breaker pending count = MAX_HOPS (256): breaker fires at hop_index==256 before the 257th body, so body ran for hop_index 0..=255
patterns_established:
  - Manual Windowed step loop pattern for testing clock-aware BeatRunner: step() in bounded loop, on AwaitingCue increment counter + record beat id + resume_cue(), stop on Done/Halted
  - Beat amount encoding (distinct integers per beat) for deterministic Intent stream identity without world state reads
  - normalize(pending) = iter().map(|i| format!('{:?}', i)).collect::<Vec<_>>() for cross-run stream comparison
observability_surfaces:
  - bevy::log::warn! on StepOutcome::Halted carrying cast_id, timeline id, and hop count — enables 3am diagnosis of infinite-loop timelines
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-15T09:05:57.939Z
blocker_discovered: false
---

# S03: Mode parity (DryRun ≡ Execute ≡ Preview) + Two-clock invariant

**Proved three S03 invariants on BeatRunner/SkillCtx/Clock: DryRun≡Execute≡Preview stream parity on branched timelines, HeadlessAuto≡Windowed end-of-cast parity with real Windowed stall, and circuit-breaker Halt at MAX_HOPS=256 with bounded Intent stream.**

## What Happened

S03 extended the S02 BeatRunner/SkillCtx/Clock primitives to prove three runtime invariants without touching the intent_applier or live pipeline.

**T01 — Clock-aware BeatRunner (src/combat/api/runner.rs):** Added `clock: Clock` field with a `with_clock` builder keeping `new()` unchanged (S02 tests unaffected). Added `StepOutcome::AwaitingCue` variant and a two-field latch (`awaiting_cue: Option<BeatId>` + `cue_just_resumed: bool`) so a Presentation-bearing beat under `Clock::Windowed` stalls exactly once before re-evaluating edges. A global stall-gate placed before the loop_stack branch handles both linear and loop-body paths uniformly. `run_to_completion` auto-resumes AwaitingCue preserving S02 batch-drive semantics. The circuit-breaker `hop_index >= MAX_HOPS` path now emits `bevy::log::warn!` with cast_id, timeline id, and hop count before returning `StepOutcome::Halted`, closing a 3am-debugging hazard. Six inline unit tests (3 pre-existing + 3 new: HeadlessAuto regression, Windowed stall, HeadlessAuto≡Windowed pending parity) all pass.

**T02 — Mode-parity integration test (tests/timeline_mode_parity.rs):** Built a branched CompiledTimeline: Impact/Cast beat (DealDamage 50) with two outgoing edges — edge A gated by a world-reading predicate (`target hp_current < threshold` via `World::try_query::<&Unit>()`) routing to a finisher beat (DealDamage 200), edge B unconditional routing to a normal beat (DealDamage 100). Two test functions spawn the world in each branch condition (deterministic, no RNG). Each runs three BeatRunners — Execute, DryRun, Preview — via `run_to_completion` without draining through intent_applier (world HP unchanged across mode runs). A `normalize` helper strips cast_id from DealDamage format for structural cross-mode comparison. Both test functions pass, confirming the predicate is live (routes both ways).

**T03 — Two-clock parity integration test (tests/timeline_two_clock_parity.rs):** Two-beat timeline (Cast with Presentation → Impact without). Hooks encode beat identity in DealDamage amounts (7, 13) for deterministic distinguishability. HeadlessAuto run via `run_to_completion`; Windowed run via a manual step loop bounded at 64 iterations — on AwaitingCue it increments a counter, records the stall beat id, calls `resume_cue()`, then continues. Assertions: outcome==Done, `awaiting_cue_count >= 1` (stall real), last stall at "cast" beat, and `format!("{:?}")` of all intents equal across both clocks (I3/D026 verified).

**T04 — Circuit-breaker integration test (tests/timeline_circuit_breaker.rs):** Loop timeline with a body Impact beat that enqueues one DealDamage per hop (`cb/one_damage_per_hop`) and an `exit_when` predicate (`cb/never`) that always returns false. Run via `run_to_completion` with `max_steps=1000`. Assertions: outcome==StepOutcome::Halted; pending contains exactly 256 DealDamage intents (breaker fires at `hop_index==256` before the 257th body execution, body ran for hop_index 0..=255); draining through intent_applier yields 256 OnDamageDealt events with no panic.

## Verification

1. `cargo test --test timeline_mode_parity` → 2 passed (finisher_branch + normal_branch, live predicate confirmed both routes). 2. `cargo test --test timeline_two_clock_parity` → 1 passed (AwaitingCue stall observed, HeadlessAuto≡Windowed pending streams equal). 3. `cargo test --test timeline_circuit_breaker` → 1 passed (Halted at MAX_HOPS=256, exactly 256 DealDamage intents, no panic). 4. `cargo test` full suite → exit 0, all tests green (S01+S02+S03 fixtures + inline unit tests, no regressions). 5. `cargo check` headless → exit 0, no new warnings. 6. `cargo check --features windowed` → exit 0, no new warnings. 7. P001 guard: `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/` → 0 matches.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/api/runner.rs` — Added Clock field, with_clock builder, StepOutcome::AwaitingCue, two-field latch, global stall-gate, auto-resume in run_to_completion, circuit-breaker warn! on Halt
- `tests/timeline_mode_parity.rs` — New: Execute≡DryRun≡Preview integration test on branched timeline with live world-reading predicate (two test functions cover both branch routes)
- `tests/timeline_two_clock_parity.rs` — New: HeadlessAuto≡Windowed end-of-cast Intent stream parity integration test with manual Windowed step loop and AwaitingCue count assertion
- `tests/timeline_circuit_breaker.rs` — New: Loop never-exit circuit-breaker integration test: StepOutcome::Halted at MAX_HOPS=256, exactly 256 DealDamage intents, no panic
