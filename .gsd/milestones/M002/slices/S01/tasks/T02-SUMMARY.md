---
id: T02
parent: S01
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-21T17:53:26.371Z
blocker_discovered: false
---

# T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation from production anim graphs

**GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation from production anim graphs**

## What Happened

Added AnimationValidationCheck::GameplayCommandForbidden and AnimationValidationReason::GameplayCommandInAnimGraph. Added graph.rs check that EmitDamage/EmitStatus/EmitHeal in node.on_enter or node.cues produces Error diagnostic. Removed EmitDamage block from production anim_graph.ron files. Added live-loaded test asserting agumon graph contains zero gameplay commands.

## Verification

cargo test green; validation check fires on broken test fixture

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 0ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
