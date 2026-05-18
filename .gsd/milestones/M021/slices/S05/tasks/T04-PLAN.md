---
estimated_steps: 4
estimated_files: 3
skills_used: []
---

# T04: Port Petit Thunder and Renamon ult data to timeline-backed canon tests

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: the slice closes only when real canon data proves the compiler/runtime path. The current roadmap names Tohakken, but the live asset id is `renamon_ult`; this task should encode Tohakken semantics under that live id and document the mapping in test names/assertions.

Do: add the real timeline data for `petit_thunder` and `renamon_ult` inside `assets/data/skills.ron`, using built-in hooks/selectors/payloads rather than bespoke test hooks. Write asset-backed integration tests that prove Petit Thunder's compiled path (including toughness/status/signal sequencing) and Renamon ult's compiled path (damage, enemy delay, ally Blessed application) end to end. Finish with broad verification on the touched code path.

Done when: both canon skills run via compiled data from `skills.ron`, the tests clearly prove the roadmap demo under current live ids, and the touched suite/checks pass without adding a second ad-hoc code path.

## Inputs

- `assets/data/skills.ron`
- `src/data/skills_ron.rs`
- `src/data/skill_timeline.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/applier.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`

## Expected Output

- `assets/data/skills.ron`
- `tests/compiled_timeline_petit_thunder.rs`
- `tests/compiled_timeline_tohakken.rs`

## Verification

cargo test --test compiled_timeline_petit_thunder --test compiled_timeline_tohakken

## Observability Impact

The new asset-backed tests become the standing proof that compiler, runtime dispatch, and applier semantics all agree on the same canon data.
