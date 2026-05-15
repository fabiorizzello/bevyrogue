# S03 — Research: Mode parity (DryRun ≡ Execute ≡ Preview) + Two-clock invariant

**Status:** targeted research. Tech is known (Bevy ECS, the S01/S02 kernel primitives are in place). S03 is invariant-hardening + two scope decisions the planner must resolve before decomposing.

## Summary

S03 must prove three invariants on the S02 `BeatRunner`/`SkillCtx`/`Clock` primitives:

1. **I2 / D024 — Mode parity:** `DryRun ≡ Execute ≡ Preview` produce a byte-identical `Intent` stream on a *branched* timeline.
2. **I3 / D026 — Two-clock parity:** `Clock::HeadlessAuto` and `Clock::Windowed` produce the same end-of-cast `Intent` stream; only timing differs (Windowed stalls on `Presentation::Cue`).
3. **Circuit breaker @256:** a `Loop` whose `exit_when` never fires halts at `MAX_HOPS = 256` instead of hanging.

The decisive research finding: **most of the machinery already exists but is inert.** `SkillCtxMode` (skill_ctx.rs:18-27) is plumbed through `step`/`fire_beat`/`eval_predicate`/`next_beat` but **no code branches on it** — Intent emission is already mode-independent at the runner. `MAX_HOPS = 256` + `StepOutcome::Halted` are fully implemented (runner.rs:19,121-124) with an inline unit test. `Clock` exists (clock.rs) but `BeatRunner` has **zero clock awareness** — S02 explicitly omitted the clock branch. So S03 is ~70% test authoring + ~30% targeted wiring, gated by two scope decisions below.

## Active Requirements This Slice Owns

No `REQUIREMENTS.md` was preloaded. From `M021-CONTEXT.md` the slice owns invariants **I2 (D024)**, **I3 (D026)**, and the circuit-breaker safety guarantee (F4). Constraint **I1 (determinism)** is adjacent: the parity tests double as determinism tests (same input ⇒ same Intent stream).

## Implementation Landscape

| File | Role for S03 |
|---|---|
| `src/combat/api/runner.rs` | `BeatRunner` FSM. Mode plumbed but unused; `MAX_HOPS=256`/`Halted` done. Clock branch absent. Selector ctx is world-naive (`state: &()`, runner.rs:271-275). |
| `src/combat/api/skill_ctx.rs` | `SkillCtxMode {Execute,DryRun,Preview}`; `SkillCtx` already carries `world: &World` (read-only) + `enqueue`. Predicates already get world. |
| `src/combat/api/clock.rs` | `Clock {HeadlessAuto,Windowed}` Resource. Doc asserts I3 but nothing enforces it. |
| `src/combat/api/applier.rs` | `intent_applier` always applies `DealDamage` to world; **no mode gating** — this is where no-apply semantics for DryRun/Preview must live (or be deliberately deferred). |
| `src/combat/api/timeline.rs` | `Presentation { cue_id, anim, vfx, sfx }` on `Beat`; `BeatKind` has no dedicated Cue stall point — Windowed must key off `beat.presentation.is_some()`. |
| `src/combat/api/rng.rs` | `CastRng::from_params(seed,cast,beat,hop,salt)` ready for an RNG-gated branched fixture. |
| `tests/timeline_chain_bolt_port.rs` | Canonical fixture pattern to fork: Loop + selector + hook + `cast_hit_set` + `run_to_completion`, asserting on `pending` before `intent_applier`. |
| `tests/timeline_onturnstart_kills.rs`, `tests/timeline_validate_typo.rs` | Other S02 demo-gate fixtures to mirror for naming/structure. |

`BeatRunner` is **only invoked from tests** today (`rg BeatRunner src/ --glob '!*/api/*'` → none). It is not yet wired into `turn_system`/`pipeline`. This keeps S03 fully contained to `src/combat/api/` + `tests/` — no live combat pipeline edits.

## Critical Scope Decisions (resolve before planning)

**Decision A — Two-clock: S02 deferred it to S04, but S03's roadmap demands "two-clock verde".**
`S02-SUMMARY.md` states "AdvanceMode (windowed clock step-through) not wired in BeatRunner — deferred to S04" and "S04 adds AdvanceMode branch, SignalBus, PassiveRunner". But the S03 roadmap line is *"two-clock verde"*. Resolution: split the concern. S03 owns the **invariant** (HeadlessAuto vs Windowed produce identical Intent stream), provable with a minimal clock-aware step that stalls/resumes on `beat.presentation` without the full SignalBus-driven Windowed driver. S04 owns the **full Windowed runtime** (SignalBus, PassiveRunner, real animation-completion signal). Recommend S03 implements: `BeatRunner::step` takes/reads `Clock`; under `Windowed`, a beat carrying `Presentation` yields a `StepOutcome::AwaitingCue` (new variant) and a `resume_cue()` call advances it; the parity test drives both clocks to completion and asserts equal `pending`. Flag for the decide phase — this is a D026 boundary clarification, candidate for `gsd_decision_save`.

**Decision B — DryRun/Preview no-apply semantics: enforce now or defer to C1 (S11)?**
D024 says DryRun produces the same Intent stream *without applying*. The runner already never applies (it only fills `pending`). True no-apply belongs to the **consumer** of `pending`: `intent_applier` (applier.rs) unconditionally mutates world. Options: (B1) S03 proves *Intent-stream parity only* (run runner 3× over the same world, assert `pending` byte-equal; do not drain) — minimal, matches "test … verde su chain ramificata" literally, no applier change; (B2) S03 also adds a mode gate so DryRun/Preview short-circuit `intent_applier`. Recommend **B1** for S03 (the invariant is about the *stream*; consumers using DryRun without applying is C1/S11's concern) and record the rationale. Note: `Intent` derives `Clone` but **not `PartialEq`** — stream equality assertions need a comparable projection (e.g. map to `(variant_tag, source, target, amount, …)` tuples or `format!("{:?}")`). Flag: deriving `PartialEq` on `Intent` is blocked because `BlueprintSignal.payload: u64` is fine but future S04 boxes it; safest is a test-local normalization helper, not a derive.

**Decision C — "chain ramificata" (branched chain): does branching need world-aware selectors?**
S02 follow-up says "S03 must wire real world-aware selectors". But branching in the FSM is via **edge gate predicates**, and predicates already receive a `SkillCtx` with `world: &World` — they can read live HP/team now. Selectors (`SelectorCtx`) still have no world (`state: &()`). A branched parity fixture (edge predicate reads target HP → routes to a different beat) is achievable **without** touching `SelectorCtx`. Recommend: keep `SelectorCtx` world-access **deferred** unless a fixture genuinely needs world-driven target *selection* (vs. branching). If wired, the design constraint is D031/S02: `SelectorCtx<S=()>` keeps `timeline.rs` Bevy-free; the runner supplies a concrete `S` wrapping a world snapshot — mirror how `SkillCtx` borrows `&World`. This is the single largest optional lift; recommend scoping it out of S03 and noting it as an S03→later follow-up unless Decision C says otherwise.

## Natural Seams (task candidates)

1. **Clock-aware stepping** (runner.rs): add `Clock` to `step` signature (or runner field), `StepOutcome::AwaitingCue` variant, `resume_cue()`; HeadlessAuto path = current behavior (no regression). Highest risk / biggest unblocker — **first proof**.
2. **Branched-chain fixture + mode-parity test** (`tests/timeline_mode_parity.rs`): one branched timeline (edge predicate reads world HP), run under Execute/DryRun/Preview, assert normalized `pending` equality. Depends on a stream-equality helper (test-local).
3. **Two-clock parity test** (`tests/timeline_two_clock_parity.rs`): same branched timeline with a `Presentation`-bearing beat; drive HeadlessAuto to completion vs Windowed (step + resume_cue loop); assert equal `pending`. Depends on seam 1.
4. **Circuit-breaker integration test** (`tests/timeline_circuit_breaker.rs`): realistic Loop with a never-true `exit_when`, assert `StepOutcome::Halted` at hop 256 and that `pending` is bounded. Decide whether Halt also emits an observable `Intent::Reject`/`log::warn!` (observability skill: a silent halt is a 3am-debugging hazard — recommend a `log::warn!` with `cast_id`+timeline id at minimum; `Intent::Reject` is the cleaner replay-visible signal but adds applier scope — flag).
5. **Optional (Decision C):** world-aware `SelectorCtx` — only if a fixture needs it.

Seams 2, 3, 4 are independent given seam 1; the planner can parallelize them after seam 1 lands.

## First Proof / Highest Risk

Seam 1 (clock-aware stepping) is the only change that touches the FSM control flow and risks regressing the three green S02 tests (`timeline_onturnstart_kills`, `timeline_chain_bolt_port`, `timeline_validate_typo`) and the inline runner unit tests. Build and prove it first: HeadlessAuto must be byte-identical to today before the Windowed branch is added. The `StepOutcome` enum is `#[derive(PartialEq, Eq)]` and matched exhaustively in `run_to_completion` (runner.rs:240-243) — adding `AwaitingCue` is a breaking match that must be handled there and in all test `assert_eq!(outcome, …)` sites.

## Verification

- `cargo test --test timeline_mode_parity` — branched chain, Execute≡DryRun≡Preview on normalized `pending`.
- `cargo test --test timeline_two_clock_parity` — HeadlessAuto≡Windowed end-of-cast `pending`.
- `cargo test --test timeline_circuit_breaker` — `StepOutcome::Halted` at 256, bounded output.
- `cargo test` — full suite green (regression guard on S02 fixtures + inline runner/timeline unit tests).
- `cargo check` (headless) and `cargo check --features windowed` — 0 new warnings; no `bevy::winit/render`/`bevy_egui` import enters `src/combat/api/` (grep check, per `api/mod.rs` discipline; tolerate the known doc-comment false positive).
- `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/` → 0 (P001 guard, unchanged).

## Skills / Don't Hand-Roll

- **rust-testing** (`/home/fabio/.agents/skills/rust-testing/SKILL.md`) — integration-test layout, deterministic fixtures; matches CLAUDE.md "tests deterministici, no wall-clock, no RNG senza seed".
- **observability** skill — applies to Decision/seam 4: a silent circuit-breaker halt violates the "next agent at 3am has signals" principle; emit `log::warn!` (or `Intent::Reject`) with `cast_id` + `timeline.id` + hop count.
- **grill-me** / decide-phase — Decisions A, B, C above are exactly the hand-wavy coupling that should be resolved before execution; A is a likely `gsd_decision_save` (D026 boundary: which slice owns Windowed runtime vs. the parity invariant).
- No external libraries needed — `CastRng` is in-tree; Bevy already used locally (skip `resolve_library`).

## Sources

Code-only research (S02 kernel is fully local). Files read: `src/combat/api/{runner,skill_ctx,clock,intent,applier,timeline,registry,signal,mod}.rs`, `src/combat/plugin.rs`, `tests/timeline_chain_bolt_port.rs`, `S02-SUMMARY.md`. `memory_query` for "DryRun Execute Preview mode parity two-clock" returned no prior notes — no superseding architecture memory exists for this slice.
