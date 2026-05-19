---
id: T02
parent: S01
milestone: M002
key_files:
  - src/animation/validation/types.rs
  - src/animation/validation/graph.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/digimon/renamon/anim_graph.ron
  - tests/anim_gameplay_command_forbidden.rs
  - tests/anim_validation.rs
key_decisions:
  - Gameplay commands are forbidden inside animation graphs; authored graphs must emit `ReleaseKernelCue` instead.
  - The anti-DRY protection is executable: the live Agumon production graph is asserted gameplay-command-free in tests.
duration: 
verification_result: passed
completed_at: 2026-05-19T19:31:49.949Z
blocker_discovered: false
---

# T02: Blocked gameplay commands in animation graphs and enforced the rule with a live anti-DRY test on the production Agumon asset.

**Blocked gameplay commands in animation graphs and enforced the rule with a live anti-DRY test on the production Agumon asset.**

## What Happened

Added `AnimationValidationCheck::GameplayCommandForbidden` and `AnimationValidationReason::GameplayCommandInAnimGraph`, then taught the graph validator to emit blocking diagnostics when `EmitDamage`, `EmitStatus`, or `EmitHeal` appear in `node.on_enter` or cue `Presentation(...)` commands. Removed gameplay-command authoring from the production Agumon and Renamon graphs and codified the rule with an anti-DRY integration test against the live Agumon asset. The final closeout rerun confirmed the rule still holds after the parity remediation changed Agumon frame ranges.

## Verification

Fresh `cargo nextest run --profile agent` passed after the final updates, including `anim_gameplay_command_forbidden`, `anim_graph_asset`, and `anim_validation` coverage for the production graphs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo nextest run --profile agent` | 0 | ✅ pass | 7700ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/validation/types.rs`
- `src/animation/validation/graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_gameplay_command_forbidden.rs`
- `tests/anim_validation.rs`
