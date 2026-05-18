---
id: T01
parent: S05
milestone: M021
key_files:
  - src/combat/api/timeline.rs
  - src/combat/api/skill_ctx.rs
  - src/combat/api/runner_common.rs
  - src/combat/api/builtins.rs
  - src/combat/api/mod.rs
  - src/combat/plugin.rs
  - src/combat/api/runner.rs
  - tests/compiled_timeline_builtin_validation.rs
key_decisions:
  - D011
duration: 
verification_result: passed
completed_at: 2026-05-15T15:07:35.205Z
blocker_discovered: false
---

# T01: Made compiled timelines asset-safe via owned id generics plus typed beat payloads, and registered kernel built-ins automatically in CombatPlugin.

**Made compiled timelines asset-safe via owned id generics plus typed beat payloads, and registered kernel built-ins automatically in CombatPlugin.**

## What Happened

Implemented the T01 foundation for asset-backed timeline compilation without string leaking. `CompiledTimeline`, `Beat`, `BeatKind`, `BeatEdge`, `Presentation`, and `TimelineLibrary` now support owned ids via a generic `Id` parameter with borrowed defaults, so existing hand-authored tests keep compiling while asset-loaded timelines can explicitly use `String`. Added `BeatPayload::DealDamage` and threaded the current beat payload through `SkillCtx`, letting built-in hooks read typed per-beat parameters without encoding them into registry ids.

Added `src/combat/api/builtins.rs` with kernel built-ins for `core/deal_damage`, `core/primary`, `core/always`, and `core/never`, and registered them from `CombatPlugin::build` alongside the existing kernel runtime setup. `CombatPlugin::finish` now validates `TimelineLibrary<String>` timelines against the registry, keeping the asset-owned path separate from the legacy borrowed test path. The new integration test file proves the plugin installs built-ins automatically, validates an owned `CompiledTimeline<String>`, exercises the built-in hook/selector/predicate functions directly, and asserts precise axis/site failures for typoed built-in ids.

I also updated the existing timeline fixtures to carry the new optional beat payload field so the core suite keeps compiling cleanly under the richer beat model.

## Verification

Verified with `cargo test --test compiled_timeline_builtin_validation` and a full `cargo test` run. The targeted test covered plugin registration, owned-timeline validation, built-in execution, and typo diagnostics; the full suite passed afterward, confirming the new payload field updates and generic timeline types did not regress existing timeline/runner behavior.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test compiled_timeline_builtin_validation` | 0 | ✅ pass | 8700ms |
| 2 | `cargo test` | 0 | ✅ pass | 18000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/timeline.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/runner_common.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `src/combat/api/runner.rs`
- `tests/compiled_timeline_builtin_validation.rs`
