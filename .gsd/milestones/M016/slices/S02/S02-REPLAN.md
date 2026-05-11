# S02 Replan

**Milestone:** M016
**Slice:** S02
**Blocker Task:** T04
**Created:** 2026-05-09T13:55:26.346Z

## Blocker Description

Completed blocker assessment T04 confirmed that user-captured design feedback invalidates the current S02 plan: adding a Dorumon-specific `SkillCustomSignal::Dorumon` enum variant in `src/data/skills_ron.rs` would make Digimon signal ownership too static, and continuing to place Predator Loop as a shared `src/combat/predator_loop.rs` mechanic conflicts with the desired one-file/plugin removal boundary for each Digimon.

## What Changed

Replaced the enum-based Dorumon signal migration with a plugin-oriented blueprint seam. The revised pending tasks first introduce a generic custom-signal envelope plus blueprint registry, then make the Dorumon blueprint/plugin own Predator Loop signal decoding and domain transitions, and finally prove runtime diagnostics/docs/audit surfaces without adding character-specific shared-system branches. Completed blocker assessment T04 is preserved unchanged as the historical reason for this replan.
