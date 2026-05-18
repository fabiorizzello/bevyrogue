---
estimated_steps: 12
estimated_files: 6
skills_used: []
---

# T01: Add composite passive event routing and loop-capable PassiveRunner

Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: the current passive framework only reacts to exact blueprint owner/name pairs carried through SignalBus, and PassiveRunner still assumes linear timelines. S07 needs passive listeners that can subscribe to kernel combat events as well as blueprint signals, compose conditions, and safely drive looped passive FSMs in the same update tick.

Do:
- Introduce a typed event-filter surface for passive listeners (including composite All/Any/Not plus custom predicate hooks) without leaking Digimon-specific names into the shared API.
- Extend the signal/event bridge so passive dispatch can observe both blueprint signals and selected kernel CombatEvent-derived envelopes, while preserving existing ult_used behavior.
- Upgrade PassiveRunner to store filter-based subscriptions, support loop bodies with the same 256-hop circuit-breaker guarantees as BeatRunner, and keep same-frame cascades deterministic when listeners enqueue more intents/signals.
- Export the new API from src/combat/api/mod.rs and add a focused integration test that exercises composite matching, same-frame cascading, and loop-halting behavior.

Failure modes / negative tests:
- Unmatched filters must no-op without consuming unrelated signals.
- Malformed or self-recursing listener loops must halt via the existing circuit breaker rather than hang the update.
- Kernel-event and blueprint-signal listeners must coexist without draining each other's work out of order.

Done when: passive listeners can subscribe through the shared filter model, looped passive timelines execute deterministically, and the targeted test proves same-frame routing plus circuit-breaker safety.

## Inputs

- `src/combat/api/event_bridge.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/runner.rs`
- `src/combat/events.rs`
- `tests/passive_kitsune_grace.rs`

## Expected Output

- `src/combat/api/event_filter.rs`
- `src/combat/api/event_bridge.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/mod.rs`
- `tests/passive_event_filters.rs`

## Verification

cargo test --test passive_event_filters

## Observability Impact

Adds explicit passive-routing seams future agents can inspect through CombatEvent-derived envelopes and PassiveRunner circuit-breaker warnings instead of reverse-engineering ad hoc owner/name matches.
