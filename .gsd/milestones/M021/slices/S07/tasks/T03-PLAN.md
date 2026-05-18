---
estimated_steps: 12
estimated_files: 6
skills_used: []
---

# T03: Migrate Agumon, Gabumon, Patamon, and Renamon passives onto PassiveRunner

Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: these four passives cover the main non-blocking migration paths for S07 — paired Twin Core triggers, Holy Support buildup, and Kitsune Grace ally-ult reactions — without the extra Tentomon pre-damage complexity. Landing them first proves the new listener/filter layer against multiple existing identity resources.

Do:
- Register passive listeners inside the relevant blueprint/runtime wiring so Agumon/Gabumon/Patamon/Renamon install their passive timelines automatically when the combat runtime boots.
- Reuse the existing Twin Core and Holy Support runtime hooks/resources instead of duplicating state machines in the passive layer.
- Replace one-off test scaffolding with canonical runtime coverage that proves ally-ult reactions, paired Twin Core signaling, and grace accumulation through the shared passive dispatcher.
- Ensure the signal taxonomy and passive listener setup leave no undrained bus entries after an update.

Failure modes / negative tests:
- Self ult and enemy ult must not trigger Kitsune Grace.
- Passive listeners must not re-fire indefinitely after their once-per-cycle / guard state is written.
- Twin Core and Holy Support passives must surface through existing kernel transition paths rather than private side channels.

Done when: the new integration test shows these four passives install through runtime wiring, emit the expected blueprint/kernel transitions, and settle cleanly in one update tick with no leftover passive bus work.

## Inputs

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/identity.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/patamon/identity.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/kernel.rs`
- `tests/passive_kitsune_grace.rs`
- `tests/twin_core_integration.rs`
- `tests/holy_support_mechanics.rs`

## Expected Output

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/kernel.rs`
- `tests/passive_canon_support.rs`

## Verification

cargo test --test passive_canon_support

## Observability Impact

Reuses existing Twin Core and Holy Support transition streams while moving passive activation onto canonical runtime wiring, giving future agents one passive-dispatch path instead of ad hoc per-test setup.
