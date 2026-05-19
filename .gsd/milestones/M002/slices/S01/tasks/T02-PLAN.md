---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation

Why: the D001/MEM021 mandate — anim graphs must author zero gameplay numbers — must become an executable invariant; the M001 mul:18 (agumon) and mul:16 (renamon) duplicates must be removed behind it. Do: In src/animation/validation/types.rs add AnimationValidationCheck::GameplayCommandForbidden and AnimationValidationReason::GameplayCommandInAnimGraph (follow the CommandParam precedent for diagnostic plumbing). In src/animation/validation/graph.rs (validate_graph_nodes) and/or validation/command.rs, add a check that any Command::EmitDamage | EmitStatus | EmitHeal appearing in node.on_enter OR in node.cues (walk cues too — FrameCue command from T01) produces an Error diagnostic. Keep existing param/status validation working for non-graph use if shared. Remove the EmitDamage block from assets/digimon/agumon/anim_graph.ron (the mul:18 dup) and from assets/digimon/renamon/anim_graph.ron (mul:16) — keep SpawnParticle (presentation, allowed). Fix the now-broken structural assertions in tests/anim_graph_asset.rs and tests/anim_validation.rs that asserted the EmitDamage mul:18 / FrameOutsideNamedClipRange-adjacent shape. Add an executable anti-DRY test (new tests/anim_gameplay_command_forbidden.rs): assert the live loaded agumon graph contains zero gameplay commands; assert a synthetic graph with EmitDamage in on_enter fails with GameplayCommandForbidden; assert a synthetic graph with EmitDamage inside a FrameCue cue also fails. Done when: cargo test green, both production graphs gameplay-command-free, the new test enforces D001. Decisions: D042 (renamon remediated identically). Q7 negatives: EmitStatus and EmitHeal in a graph also rejected, not just EmitDamage.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/validation/types.rs`
- `src/animation/validation/command.rs`
- `src/animation/validation/graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_validation.rs`

## Expected Output

- `src/animation/validation/types.rs`
- `src/animation/validation/command.rs`
- `src/animation/validation/graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_validation.rs`
- `tests/anim_gameplay_command_forbidden.rs`

## Verification

cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation
