---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Test integrazione Agumon energy-loop

Nuovo test in tests/digimon_kits/ che esercita il loop completo headless: spawn Agumon vs dummy, lancia sharp_claws N volte, verifica energy sale, ult resta locked finche energy<max, ult si abilita a energy=max, lancia ult, verifica energy.current==0 dopo cast. Usa harness esistenti e seeded RNG (R004).

## Inputs

- `src/combat/action_query/legality/action.rs`
- `src/combat/resolution/apply.rs`

## Expected Output

- `Test verde che dimostra il loop end-to-end per Agumon`
- `Regression guard per le tre semantiche: fills/locks/drains`

## Verification

cargo test --features windowed --test digimon_kits agumon_energy_gauge
