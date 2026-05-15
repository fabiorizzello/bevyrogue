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
completed_at: 2026-05-15T19:18:02.493Z
blocker_discovered: false
---

# T01: Expanded the timeline builtin surface for revive, energy, free-skill, and tempo verbs and proved looped hop ordering with bounded-exit coverage.

**Expanded the timeline builtin surface for revive, energy, free-skill, and tempo verbs and proved looped hop ordering with bounded-exit coverage.**

## What Happened

Added new BeatPayload variants for revive, grant free skill, grant energy, advance turn, and self-advance, then wired them through the compiled timeline pipeline. Reworked the kernel builtin registry to expose dedicated verb hooks alongside the legacy apply_effect dispatcher, and taught the intent applier how to execute revive, energy gain, free-skill grants, and advance-turn events while keeping read-only world inspection headless-safe. Also added a small SkillCtx query helper for caster-team inspection and extended the focused tests to cover the new verb translations plus a bounded-loop regression for the chain-bolt pattern.

## Verification

Ran the slice verification command `cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder`. It completed successfully and the new bounded-loop regression plus builtin translation checks passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder` | 0 | ✅ pass | 518ms |

## Deviations

Added dedicated verb-specific builtin hooks in addition to the legacy apply_effect path so new timeline assets can bind to stricter hooks immediately.

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
