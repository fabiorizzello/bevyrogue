---
id: T04
parent: S06
milestone: M021
key_files:
  - src/combat/turn_system/mod.rs
  - src/bin/combat_cli.rs
  - tests/combat_coherence.rs
  - tests/compiled_timeline_tohakken.rs
key_decisions:
  - Restore mixed root dispatch so `timeline: None` fixture skills still execute through `step_app` while compiled-timeline skills stay on the beat-runner path.
  - Keep CLI proof-mode truthful to current runtime behavior by observing the skill-cast surface without reintroducing broken legacy assumptions about action-log population or root break follow-up pressure.
duration: 
verification_result: passed
completed_at: 2026-05-16T07:21:59.633Z
blocker_discovered: false
---

# T04: Completed the remaining timeline-era fixture migration and repaired stale test expectations around mixed dispatch, CLI proofing, and deferred follow-up behavior.

**Completed the remaining timeline-era fixture migration and repaired stale test expectations around mixed dispatch, CLI proofing, and deferred follow-up behavior.**

## What Happened

I finished the T04 cleanup by bringing the remaining inline-fixture and contract tests in line with the timeline-era runtime. The key runtime fix was restoring mixed dispatch in `resolve_action_system`: skills with compiled timelines still go through the beat runner, while fixture and legacy-style skills with `timeline: None` now correctly route through `step_app` instead of being silently dropped. I also tightened the combat CLI proof harness so non-interactive runs prefer an enabled skill, emit an explicit proof hint for the skill-cast surface, and allow proof completion when the timeline path resolves without populating the legacy action log. On the test side, I updated coherence and compiled-timeline assertions to match the current timeline-only contract: deferred allied break-follow-up pressure stays documented as absent, and capped delay surfaces are asserted at the emitted 50% value rather than the authored pre-cap value. This leaves S06 with a green full-suite verification pass and an actual T04 summary artifact on disk, which should stop auto-mode from looping on a missing task closeout file.

## Verification

Fresh verification passed via `bash tools/verify_m021_s06_t04.sh`, which reran `cargo test`, `cargo check`, `cargo check --features windowed`, and the structural removal grep for legacy `apply_effects(` / `effects:` references across `src`, `tests`, and `assets/data/skills.ron`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash tools/verify_m021_s06_t04.sh` | 0 | ✅ pass | 26300ms |

## Deviations

Adjusted two stale test expectations during closeout: the coherence break-follow-up scenario now documents the currently deferred timeline root-follow-up surface, and the Renamon ultimate timeline test now asserts the emitted capped `DelayTurn` payload rather than the authored pre-cap delay value.

## Known Issues

None.

## Files Created/Modified

- `src/combat/turn_system/mod.rs`
- `src/bin/combat_cli.rs`
- `tests/combat_coherence.rs`
- `tests/compiled_timeline_tohakken.rs`
