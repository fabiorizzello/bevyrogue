---
estimated_steps: 12
estimated_files: 6
skills_used: []
---

# T04: Migrate Dorumon and Tentomon reactive passives and prove deterministic Block Reaction

Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: Dorumon's predator_loop and Tentomon's battery_loop are the slice's highest-risk passives because they bridge listener-driven passives into existing runtime resources, and Tentomon depends on the new pre-damage Block Reaction seam promised by the roadmap demo.

Do:
- Install Dorumon and Tentomon passive listeners through runtime/plugin wiring, reusing the existing Predator Loop and Battery Loop resource systems rather than forking logic.
- Implement Tentomon's BlockReady -> BlockProc reaction flow on IncomingDamage using only deterministic RNG sourced from the combat skill/runtime context.
- Ensure the passive dispatcher can arm the reaction, enqueue the mitigation intent, and emit BlockReactionTriggered in the same frame before HP mutation completes.
- Add an end-to-end integration test that covers predator/battery listener registration, deterministic block outcomes for a fixed seed, and the absence of mitigation when the guard conditions are not met.

Failure modes / negative tests:
- Unarmed Tentomon must take baseline damage.
- Battery Loop must not proc when SP/RNG/ownership predicates fail.
- Dorumon/Tentomon listeners must not consume each other's signals or mutate state outside the shared kernel/runtime seams.

Done when: the reactive-canon test proves Dorumon and Tentomon install through the runtime, Tentomon's Block Reaction reduces pre-DR damage deterministically and emits the triggered event, and the baseline path still matches when the reaction is not armed.

## Inputs

- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `src/combat/kernel.rs`
- `src/combat/api/applier.rs`
- `src/combat/events.rs`
- `docs/future_design_draft/digimon/tentomon/04_passive_battery_loop.md`
- `tests/battery_loop_kernel.rs`
- `tests/predator_loop_kernel.rs`

## Expected Output

- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `src/combat/kernel.rs`
- `tests/passive_reactive_canon.rs`

## Verification

cargo test --test passive_reactive_canon

## Observability Impact

Makes BlockReaction and reactive passive outcomes visible through canonical combat events and existing battery/predator snapshots, which is the main diagnostic seam needed for future RNG-sensitive bugs.
