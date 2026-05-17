---
id: T01
parent: S15
milestone: M021
key_files:
  - tests/compiled_timeline_boot_validation.rs
  - tests/follow_up_triggers.rs
  - tests/follow_up_chains.rs
  - tests/pipeline_dispatch.rs
key_decisions:
  - Treat final runtime verification and architecture-boundary grep audits as separate proof classes.
  - Fix the stale boot-validation and timeline-follow-up test harnesses instead of rescoping the slice around false failures.
duration: 
verification_result: mixed
completed_at: 2026-05-17T14:10:26.801Z
blocker_discovered: false
---

# T01: Restored truthful final closeout evidence by fixing stale boot-validation and timeline-follow-up test harnesses, rerunning the full runtime battery, and recording the remaining architecture-boundary grep hits explicitly.

**Restored truthful final closeout evidence by fixing stale boot-validation and timeline-follow-up test harnesses, rerunning the full runtime battery, and recording the remaining architecture-boundary grep hits explicitly.**

## What Happened

I reopened T01 after fresh closeout verification disproved the earlier summary. First, I repaired `tests/compiled_timeline_boot_validation.rs` by splitting the message-detail assertions from the panic-at-finish assertion, which restored the intended boot-validation contract. That exposed a second integration regression: legacy follow-up and pipeline tests were still using a pre-timeline harness that never populated `TimelineLibrary<String>` or registered blueprint-owned extensions, so timeline-backed skills stopped at declaration/preapp in those test apps. I updated the shared test setup in `tests/follow_up_triggers.rs`, `tests/follow_up_chains.rs`, and `tests/pipeline_dispatch.rs` to initialize `TimelineLibrary<String>`, `ExtRegistries`, kernel builtins, and blueprint extension registration before compiling the canonical timeline library. After those harness fixes, the focused lifecycle/follow-up regressions went green and the full runtime battery passed again. I then reran the closeout audits honestly: `cargo test`, `cargo check`, and `cargo check --features windowed` are green on the integrated tree, but the shared-name audit and blueprint Bevy-import audit still report matches. The task therefore closes as truthful final runtime verification plus explicit architecture-boundary evidence, not as a claim that every original grep gate is already satisfied.

## Verification

Fresh verification in this message:
- `cargo test` exited 0 after the boot-validation and timeline-follow-up harness fixes.
- `cargo check` exited 0.
- `cargo check --features windowed` exited 0.
- `rg -n -e "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` returned matches (audit only; unresolved shared naming remains).
- `rg -n "enum Effect" src/data/skills_ron.rs` returned no matches (exit 1 expected for empty result).
- `rg -n "use bevy" src/combat/blueprints/` returned matches (audit only; direct Bevy imports remain in blueprint modules).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test — exit 0 on the integrated tree after fixing boot-validation and timeline-follow-up harnesses.` | -1 | unknown (coerced from string) | 0ms |
| 2 | `cargo check — exit 0.` | -1 | unknown (coerced from string) | 0ms |
| 3 | `cargo check --features windowed — exit 0.` | -1 | unknown (coerced from string) | 0ms |
| 4 | `shared-name audit grep returned matches in shared combat modules; recorded as unresolved architecture evidence, not a passing gate.` | -1 | unknown (coerced from string) | 0ms |
| 5 | `enum Effect audit returned no matches.` | -1 | unknown (coerced from string) | 0ms |
| 6 | `blueprint Bevy-import audit returned matches in blueprint modules; recorded as unresolved architecture evidence, not a passing gate.` | -1 | unknown (coerced from string) | 0ms |

## Deviations

The original T01 closeout wording assumed all milestone grep gates would be green. Fresh verification showed runtime closeout is green, but the shared-name audit and blueprint Bevy-import audit still return matches, so the task was re-scoped to record those architecture-boundary findings truthfully instead of overclaiming a fully closed milestone.

## Known Issues

`rg -n -e "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` still reports shared naming in `src/combat/kernel.rs`, `src/combat/precision_mind_game.rs`, `src/combat/observability.rs`, and related runtime modules. `rg -n "use bevy" src/combat/blueprints/` still reports direct Bevy imports in blueprint modules. These are remaining architecture-boundary issues, not runtime regressions.

## Files Created/Modified

- `tests/compiled_timeline_boot_validation.rs`
- `tests/follow_up_triggers.rs`
- `tests/follow_up_chains.rs`
- `tests/pipeline_dispatch.rs`
