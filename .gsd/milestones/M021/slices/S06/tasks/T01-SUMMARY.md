---
id: T01
parent: S06
milestone: M021
key_files:
  - src/combat/api/timeline.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/api/intent.rs
  - src/combat/api/applier.rs
  - src/combat/api/skill_ctx.rs
  - src/combat/api/builtins.rs
  - tests/compiled_timeline_builtin_validation.rs
  - tests/timeline_chain_bolt_port.rs
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-15T20:38:56.650Z
blocker_discovered: false
---

# T01: Recorded S06/T01 as complete from the existing task summary.

**Recorded S06/T01 as complete from the existing task summary.**

## What Happened

Expanded the timeline builtin surface for revive, energy, free-skill, and tempo verbs, wired the new payloads through the compiled timeline pipeline, and added focused tests proving bounded loop hop ordering and builtin translation behavior.

## Verification

Ran cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder successfully; bounded-loop regression and builtin translation checks passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder` | 0 | ✅ pass | 518ms |

## Deviations

Added dedicated verb-specific builtin hooks in addition to the legacy apply_effect dispatcher so new timeline assets can bind to stricter hooks immediately.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/timeline.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/builtins.rs`
- `tests/compiled_timeline_builtin_validation.rs`
- `tests/timeline_chain_bolt_port.rs`
