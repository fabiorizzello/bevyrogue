---
estimated_steps: 4
estimated_files: 4
skills_used: []
---

# T02: Refresh windowed damage preview through a cached preview-stream bridge

Skills used: bevy, frontend-design, bevy, rust-best-practices, verify-before-complete.

Why: `combat_panel` is a typed egui system and cannot safely run a world-backed preview query inline; the UI still needs a live damage estimate derived from the shared preview stream.

Do: add a windowed-facing preview cache/resource and an exclusive refresh system that watches the active actor, pending action, and previewable target state, calls `query_skill_preview`, folds `Intent::DealDamage` into a numeric summary, and stores the result for the egui panel; register that bridge in `src/windowed.rs`; update `src/ui/combat_panel.rs` to consume the cached preview in skill/target hover text or pending-action labels without reintroducing bespoke damage prediction logic; add a focused windowed-feature integration test that proves cache refresh is driven by the shared preview stream and stays stable when no preview is available.

Done when: the windowed combat panel reads a cached preview summary sourced from preview intents, and the bridge test plus windowed compile prove the UI path is wired without making the panel exclusive.

## Inputs

- `src/combat/preview.rs`
- `src/ui/combat_panel.rs`
- `src/windowed.rs`
- `src/combat/action_query.rs`
- `src/data/mod.rs`
- `tests/action_affordance_consumers.rs`

## Expected Output

- `src/combat/preview.rs`
- `src/ui/combat_panel.rs`
- `src/windowed.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache

## Observability Impact

Adds a concrete `PreviewCache`-style inspection surface so preview failures can be distinguished from egui rendering problems during windowed debugging.
