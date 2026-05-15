---
estimated_steps: 11
estimated_files: 9
skills_used: []
---

# T01: Expand the generic timeline verb surface for looped active skills

Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.

Why: S05 only proved straight-line active execution. S06 must first cover the remaining generic active verbs and the loop-backed hop semantics that the canon roster still lacks in shipped assets.

Do:
- Extend the timeline payload and builtin execution surface only for the active verbs needed by the child-roster canon set: looped multi-hop damage sequencing, break, status, delay or advance tempo, revive, grant-free-skill, energy grant, and self-targeted tempo side effects where the active catalog actually uses them.
- Reuse the generic targeting and bounce helpers already living in `src/combat/resolution.rs` instead of re-implementing hop selection in blueprint code.
- Keep the runtime headless-safe and continue using immutable Bevy world queries from `SkillCtx` helpers via `World::try_query::<&T>()` where read-only ECS inspection is needed.
- Add or update focused tests so the generic runtime proves loop iteration order, payload-to-intent translation, and failure behavior for malformed payload or missing registry wiring before any broad asset rewrite begins.

Negative tests:
- Missing or wrong payload type on a builtin hook must still fail loudly at the beat site.
- Loop chains with exhausted targets or bounded hops must stop deterministically without hidden extra iterations.

Done when: the generic timeline runtime can express the remaining active verb surface needed by the canon assets, and the loop-focused targeted tests pass without relying on the legacy action resolver.

## Inputs

- `src/combat/api/timeline.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/resolution.rs`
- `tests/timeline_chain_bolt_port.rs`
- `tests/compiled_timeline_builtin_validation.rs`
- `tests/compiled_timeline_petit_thunder.rs`

## Expected Output

- `src/combat/api/timeline.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/resolution.rs`
- `tests/timeline_chain_bolt_port.rs`
- `tests/compiled_timeline_builtin_validation.rs`
- `tests/compiled_timeline_petit_thunder.rs`

## Verification

cargo test --test timeline_chain_bolt_port --test compiled_timeline_builtin_validation --test compiled_timeline_petit_thunder

## Observability Impact

Keeps the event-stream proof surface intact for hop order and beat lifecycle, and preserves beat-site panic or validation quality when builtins receive malformed payloads or missing registry ids.
