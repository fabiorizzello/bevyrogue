---
estimated_steps: 1
estimated_files: 9
skills_used: []
---

# T01: Expand the generic timeline verb surface for looped active skills

Extend the timeline payload and builtin execution surface for the remaining active verbs needed by the child-roster canon set, including looped multi-hop damage sequencing, break, status, delay/advance tempo, revive, grant-free-skill, energy grant, and self-targeted tempo side effects. Reuse generic targeting and bounce helpers in src/combat/resolution.rs, keep runtime headless-safe, and add focused tests for loop iteration order, payload-to-intent translation, and malformed payload or missing registry wiring failure behavior.

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
