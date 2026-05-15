---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Circuit-breaker integration test: Loop with never-true exit_when halts at MAX_HOPS

WHY: F4 — an infinite Loop must halt deterministically with a bounded Intent stream and an observable signal, not hang. DO: create tests/timeline_circuit_breaker.rs. Build a realistic-ish Loop CompiledTimeline whose body has an Impact beat with a hook that enqueues one Intent::DealDamage per hop, and an exit_when predicate registered to always return false (mirror the inline `never` predicate but as an integration fixture with a real spawned world and intent_applier wiring like chain_bolt_port). Run via run_to_completion with max_steps comfortably above MAX_HOPS (e.g. 1000). Assert: outcome == StepOutcome::Halted; pending length is bounded (<= MAX_HOPS, i.e. the breaker stopped accumulation — assert exactly MAX_HOPS or <= MAX_HOPS+1 depending on observed fire-before-halt ordering, document the exact expected count from the runner semantics); run_to_completion did NOT panic (proves the breaker fired before max_steps). Note in a comment that the Halt also emits bevy::log::warn! (added in T01) — capturing log output is out of scope, the StepOutcome::Halted assertion is the contract. DONE-WHEN: cargo test --test timeline_circuit_breaker passes.

## Inputs

- `src/combat/api/runner.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `tests/timeline_chain_bolt_port.rs`

## Expected Output

- `tests/timeline_circuit_breaker.rs`

## Verification

cargo test --test timeline_circuit_breaker 2>&1 | tail -5 (passes: Halted at MAX_HOPS, bounded pending, no panic)
