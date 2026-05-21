---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Drain Energy su UltEffect::Reset per energy-backed

In resolution/apply.rs e turn_system/pipeline/timeline_exec.rs (finalize path), quando l'attacker e energy-backed (metadata = energy) azzerare anche Energy.current oltre a attacker_ult.current. Mantenere UltimateCharge=0 per back-compat finche legacy non viene smantellato.

## Inputs

- `src/combat/ult_gauge.rs`
- `src/combat/mechanics/energy.rs`
- `src/combat/action_query/legality/action.rs`

## Expected Output

- `Cast ult Agumon -> Energy.current = 0 e UltimateCharge.current = 0`
- `Cast ult Digimon legacy -> solo UltimateCharge.current = 0`

## Verification

cargo test --features windowed --test damage_resolution --test windowed_only
