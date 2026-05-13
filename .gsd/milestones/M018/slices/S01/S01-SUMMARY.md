---
id: S01
parent: M018
milestone: M018
provides:
  - AdvanceTurn(u32) primitive with ±50% cap and [0,20000] AV clamp
  - DelayTurn(u32) primitive with TempoResistance curve and floor 0
  - turn_advance_split.rs boundary test suite (6 deterministic cases)
  - combat_cli --scenario advance-delay-cap with JSONL per-application output
  - M017 Slowed pipeline migrated to DelayTurn{30} with invariant AV outcome
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - Cap enforced at emission site (pct.min(50) in extractors) not in the applicator — no accumulator-side cap needed, eliminating double-cap ambiguity.
  - Floor 0 replaces MIN_ACTION_THRESHOLD_AV — lock prevention is now structural (cap ±50 + TempoResistance + Speed accumulator), not a runtime clamp exception.
  - AV ceiling raised to 2*MAX_AV (20_000) to accommodate double-advance without overflow; is_ready() threshold unchanged at MAX_AV (10_000).
  - TempoResistance curve applied only on the delay path — AdvanceTurn is uncurved, preserving M017 semantics.
  - CLI scenario runs before Bevy ECS starts (standalone function) — zero ECS overhead, immediate output, clean exit.
  - JSONL emitted unconditionally for the scenario (no BEVYROGUE_JSONL env gate) to make cap/floor evidence visible without env setup.
patterns_established:
  - Semantic split over signed integer for directional game primitives: use separate enum variants (AdvanceTurn/DelayTurn) rather than sign-encoding direction in a single signed value.
  - Cap at emission site pattern: apply pct.min(50) in the resolver extractor before constructing the event — the event bus carries already-capped values, no secondary guard needed downstream.
  - Standalone CLI scenario function before Bevy App::run() for deterministic, ECS-free output verification.
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T15:59:51.558Z
blocker_discovered: false
---

# S01: Time-manipulation split: AdvanceTurn / DelayTurn con cap ±50% e clamp [0,200]

**Replaced Effect::TurnAdvance(i32) with semantic split AdvanceTurn(u32)/DelayTurn(u32); enforced per-call ±50% cap at emission site and AV clamp [0,20000] in applicator; all callers migrated, 554 tests green, CLI scenario produces JSONL with cap visible.**

## What Happened

S01 replaced the single signed `Effect::TurnAdvance(i32)` primitive with a semantically split pair: `Effect::AdvanceTurn(u32)` and `Effect::DelayTurn(u32)`. The work spanned five tasks.

**T01 — Pure-logic applicator:** Added `apply_advance(av, pct)` and `apply_delay(av, pct, resistance)` in `src/combat/av.rs` / `src/combat/resistance.rs`. Both apply an internal defensive cap (pct.min(50)) and clamp the resulting AV to [0, 2*MAX_AV] = [0, 20_000]. `TempoResistance` curve is applied only in the delay path. Floor 0 replaces `MIN_ACTION_THRESHOLD_AV` as the infinite-delay lock prevention mechanism, now structural rather than a runtime guard. Eight inline boundary tests in `resistance.rs` verified the pure functions.

**T02 — DSL, event bus, resolver, and all callers:** Added `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` and `DelayTurn { target, amount_pct: u32 }`. Rewrote `apply_turn_advance_system` → `apply_av_ops_system` to match the two new event kinds. Extractors `skill_advance` / `skill_delay` in `resolution.rs` apply `pct.min(50)` at the emission site — no accumulator-side cap needed. `ResolvedAction` split `turn_advance_pct: i32` into `advance_pct: u32` + `delay_pct: u32`. `Effect::TurnAdvance(i32)` and `CombatEventKind::TurnAdvance` removed completely with zero shim residue. `log.rs`, `observability.rs`, `jsonl_logger.rs`, and the windowed `combat_panel.rs` all updated to match the new variant names. Mechanical sweep of ~15 integration test files that instantiate `ResolvedAction` updated to use the new fields (default 0/0).

**T03 — M017 Slowed regression:** The Slowed pipeline branch already emitted `DelayTurn{amount_pct:30}` (done by T02). T03 updated `tests/tempo_resistance.rs` pure-logic tests: floor invariant corrected from `-MIN_ACTION_THRESHOLD_AV` to 0, advance ceiling corrected from `MAX_AV` to `2*MAX_AV`. `status_slowed_delay.rs` and `tempo_resistance.rs` both green; AV outcome (5000→2000) invariant preserved.

**T04 — Boundary test suite:** Added `tests/turn_advance_split.rs` with 6 deterministic headless cases covering: DelayTurn cap (80→50), AdvanceTurn cap (80→50), double-advance ceiling (10_000→20_000), third advance no-op past ceiling, floor (AV=2000 after 50% delay → 0), and TempoResistance curve preservation on delay path. All 6 green, deterministic across runs.

**T05 — CLI scenario and full-suite gate:** Extended `src/bin/combat_cli.rs` with `--scenario advance-delay-cap`. Runs standalone before Bevy ECS starts. Applies a scripted sequence (AdvanceTurn(50), AdvanceTurn(50), DelayTurn(80), DelayTurn(50)) to two mock units; prints step-by-step AV gauge bars and emits one JSONL line per application with fields `kind`, `target`, `amount_pct_requested`, `amount_pct_capped`, `av_pre`, `av_delta`, `av_post`. Cap enforcement (pct_requested=80 → pct_capped=50) and floor clamp (Δ=0 when AV already 0) are explicitly visible in the output. Full `cargo test`: 554 tests, 0 failures. `cargo check --features windowed`: clean.

## Verification

1. **cargo check** (headless + windowed): clean — warnings only, zero errors.
2. **rg legacy TurnAdvance sweep**: `rg -n 'Effect::TurnAdvance|CombatEventKind::TurnAdvance' src/ assets/ tests/` → CLEAN, zero occurrences. Only `TurnAdvanced` (turn-order event, unrelated) remains.
3. **cargo test full suite**: 554 tests across all integration suites, 0 failed, 0 ignored. Includes turn_advance_split (6/6), status_slowed_delay (1/1), tempo_resistance (14/14), and all 40+ prior test binaries.
4. **CLI scenario**: `cargo run --bin combat_cli -- --scenario advance-delay-cap` exit 0. JSONL output shows cap enforcement (amount_pct_requested=80 → amount_pct_capped=50 on step 3) and floor clamp (Δ=0 on step 4). All four AV transitions match expected math.
5. **M017 regression**: status_slowed_delay.rs and tempo_resistance.rs both green with updated event variant names and corrected floor/ceiling invariants; AV outcome (5000→2000) preserved.

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

T03 scope was narrower than planned: pipeline.rs, skills.ron, combat_panel.rs, and status_slowed_delay.rs were already fully migrated by T02; T03 only needed to update tempo_resistance.rs pure-logic tests (floor/ceiling invariants). Plan described those files as T03 targets.

## Known Limitations

81 compiler warnings present (pre-existing unused field/variable warnings, no errors). Not introduced by this slice.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/av.rs` — 
- `src/combat/resistance.rs` — 
- `src/data/skills_ron.rs` — 
- `src/combat/events.rs` — 
- `src/combat/resolution.rs` — 
- `src/combat/state.rs` — 
- `src/combat/turn_system/mod.rs` — 
- `src/combat/log.rs` — 
- `src/combat/observability.rs` — 
- `src/combat/jsonl_logger.rs` — 
- `src/combat/turn_system/pipeline.rs` — 
- `assets/data/skills.ron` — 
- `src/ui/combat_panel.rs` — 
- `tests/status_slowed_delay.rs` — 
- `tests/tempo_resistance.rs` — 
- `tests/turn_advance_split.rs` — 
- `src/bin/combat_cli.rs` — 
