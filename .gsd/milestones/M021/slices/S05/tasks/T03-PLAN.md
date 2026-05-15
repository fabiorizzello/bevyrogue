---
estimated_steps: 4
estimated_files: 6
skills_used: []
---

# T03: Route timeline-backed actions through BeatRunner and cover required intent variants

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: compiling timelines is not enough unless production action dispatch can actually consume them, and S05's proof skills require more than `DealDamage` from the kernel applier.

Do: extend the resolved-action/runtime path so timeline-backed skills dispatch through `BeatRunner` using the compiled library while non-migrated skills continue to use the legacy effect resolver. In the same increment, implement the exact applier coverage this slice needs: `BreakToughness`, `ApplyStatus`, `DelayTurn`, `ApplyBuff`, and any sequencing needed to preserve canonical event ordering for compiled skills. Reuse existing combat subsystems and event semantics instead of inventing a second rules path.

Done when: a production-like integration test can execute a timeline-backed action through the normal turn pipeline, required effects mutate/emit through kernel primitives, and unmigrated skills still have a legacy fallback.

## Inputs

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/runner.rs`
- `src/data/skill_timeline.rs`
- `tests/turn_advance_split.rs`
- `tests/status_slowed_delay.rs`

## Expected Output

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/runner.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`

## Verification

cargo test --test compiled_timeline_runtime_dispatch

## Observability Impact

Keeps compiled-skill execution inspectable through the existing combat event stream, which is the main future-agent diagnostic surface when timeline dispatch or applier routing regresses.
