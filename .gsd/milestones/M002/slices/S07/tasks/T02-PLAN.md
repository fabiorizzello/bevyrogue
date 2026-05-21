---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T02: Ult readiness via effective_ult_gauge

In legality/action.rs, quando snapshot.gauge_meta indica energy-backed, calcolare readiness ult da effective_ult_gauge(meta, energy, ult) invece di leggere actor.ultimate_ready. Aggiornare anche resources.rs per ResourceKind::Ultimate details (current/max/ready). Path legacy invariato quando metadata vuota.

## Inputs

- `src/combat/action_query/types.rs`
- `src/combat/ult_gauge.rs`

## Expected Output

- `Ult per Agumon enabled SOLO quando energy.current >= energy.max`
- `Ult per Digimon legacy invariato (gate su UltimateCharge.current >= trigger)`

## Verification

cargo test --features windowed --test action_query
