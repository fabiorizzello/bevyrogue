---
id: T03
parent: S02
milestone: M002
key_files:
  - assets/data/digimon/agumon/skills.ron
  - assets/data/digimon/agumon/unit.ron
  - tests/agumon_sharp_claws_asset.rs
  - tests/compiled_timeline_boot_validation.rs
  - tests/compiled_timeline_active_canon.rs
  - tests/twin_core_integration.rs
  - tests/snapshots/follow_up_triggers__agumon_break_follow_up_uses_real_pilot_config.snap
key_decisions:
  - Kept Sharp Claws’ legacy damage/break/custom-signal values aligned with its timeline payload so headless follow-up and twin-core behavior stayed stable while Basic rerouted to the new cue-gated animation path.
duration: 
verification_result: passed
completed_at: 2026-05-19T21:12:34.471Z
blocker_discovered: false
---

# T03: Routed Agumon Basic to a real `sharp_claws` timeline-backed skill and added cue-barrier asset coverage for the new data path.

**Routed Agumon Basic to a real `sharp_claws` timeline-backed skill and added cue-barrier asset coverage for the new data path.**

## What Happened

Added a real `sharp_claws` Agumon `SkillDef` in `assets/data/digimon/agumon/skills.ron` with single-target targeting, zero SP cost, preserved legacy damage/break/custom-signal behavior, and a compiled timeline shaped as cast → windup → impact_damage → impact_break → recovery. The impact beat now carries the `core/deal_damage` payload plus presentation metadata (`cue_id: agumon/sharp_claws/impact`, `anim: sharp_claws_strike`) so the windowed cue barrier can release on the authored strike node without putting gameplay commands into the animation graph. Routed `assets/data/digimon/agumon/unit.ron` so Agumon Basic resolves `SkillId("sharp_claws")` while Baby Flame remains in `skill_ids` for later slices. Added `tests/agumon_sharp_claws_asset.rs` to assert the canonical roster points Basic at Sharp Claws, Baby Flame still parses, Sharp Claws compiles into the expected beat/presentation shape, missing builtins and bad selector ids fail through the existing compile-error path, and the Agumon anim graph still has no gameplay-command violations. Updated canon-facing tests/snapshot that encode the compiled timeline count or Agumon basic skill id so the new routing stays coherent across asset-validation coverage.

## Verification

Passed the exact task-plan verification command (`cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden`) and an expanded affected-surface regression run covering the touched canon tests (`compiled_timeline_boot_validation`, `compiled_timeline_active_canon`, `twin_core_integration`, `follow_up_triggers`).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden --test compiled_timeline_boot_validation --test compiled_timeline_active_canon --test twin_core_integration --test follow_up_triggers` | 0 | ✅ pass | 3938ms |
| 2 | `cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden` | 0 | ✅ pass | 160ms |

## Deviations

Updated existing canon tests/snapshot that pin Agumon’s basic skill id and compiled-timeline count, in addition to the requested asset files/test, so the routing change stayed green.

## Known Issues

None.

## Files Created/Modified

- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/agumon_sharp_claws_asset.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `tests/compiled_timeline_active_canon.rs`
- `tests/twin_core_integration.rs`
- `tests/snapshots/follow_up_triggers__agumon_break_follow_up_uses_real_pilot_config.snap`
