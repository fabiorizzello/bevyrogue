---
id: T05
parent: S01
milestone: M018
key_files:
  - src/bin/combat_cli.rs
key_decisions:
  - Scenario runs standalone before Bevy starts — no ECS overhead, exits immediately after printing
  - JSONL emitted to stdout unconditionally (no env gate) for the scenario — differs from jsonl_logger_system which gates on BEVYROGUE_JSONL env var
  - Cap enforcement (80→50) and floor clamp visible in step 3 and 4 respectively, confirming both safety invariants work end-to-end
duration: 
verification_result: passed
completed_at: 2026-05-13T15:51:52.309Z
blocker_discovered: false
---

# T05: Added --scenario advance-delay-cap to combat_cli: step-by-step AV gauge + JSONL per application; full test suite green (0 failures)

**Added --scenario advance-delay-cap to combat_cli: step-by-step AV gauge + JSONL per application; full test suite green (0 failures)**

## What Happened

Extended `src/bin/combat_cli.rs` with `--scenario advance-delay-cap`. Added a standalone `run_advance_delay_cap_scenario()` function that runs before Bevy starts. The scenario creates two mock units (Agumon at AV=0, Gabumon at AV=5000 with TempoResistance), then applies a scripted sequence: AdvanceTurn(50), AdvanceTurn(50), DelayTurn(80), DelayTurn(50). Each step prints a human-readable AV gauge (bar chart) and emits a JSONL line with fields: kind, target, amount_pct_requested, amount_pct_capped, av_pre, av_delta, av_post. The cap enforcement (80→50) and floor clamp (Δ=0 when AV already at 0) are visible in the output. Added imports for `apply_advance`, `apply_delay`, `ActionValue`, `MAX_AV`, `TempoResistance` from the lib. `cargo test` full suite: all bins passed, 0 failures. `cargo check --features windowed` also clean.

## Verification

cargo run --bin combat_cli -- --scenario advance-delay-cap (exit 0, JSONL with cap visible); cargo test full suite (all test results ok, 0 failed); cargo check --features windowed (clean)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo run --bin combat_cli -- --scenario advance-delay-cap` | 0 | pass | 2000ms |
| 2 | `cargo test 2>&1 | grep -E 'FAILED|error\['` | 0 | pass — no failures | 15000ms |
| 3 | `cargo check --features windowed` | 0 | pass | 1000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/bin/combat_cli.rs`
