---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T03: Prove missing skill graph fallback and hot reload next spawn

Harden animation graph registry/player behavior so a missing skill graph is strict where M002 canonical assets are expected at boot, but runtime lookup failure degrades to a deterministic instant graph/player path with a structured diagnostic. Add a hot-reload test proving modified graph assets update registry state only for newly spawned players while an in-flight player keeps its current graph identity/state.

## Inputs

- `src/animation/registry.rs`
- `src/animation/player.rs`
- `tests/animation.rs`

## Expected Output

- `tests/animation/anim_registry_failure_visibility.rs`

## Verification

cargo test --test animation anim_registry_failure_visibility

## Observability Impact

Adds/validates diagnostic message/state for missing graph fallback and hot-reload next-spawn behavior.
