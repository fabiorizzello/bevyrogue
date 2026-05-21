---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T05: Fixture sweep UnitQuerySnapshot callsite nei test

Aggiornare le fixture e i test esistenti che costruiscono UnitQuerySnapshot a mano o hard-codano la tuple units_data per il nuovo shape (gauge_meta + energy opzionali). Mantenere None/None nelle fixture legacy per non opt-in nessun comportamento nuovo.

## Inputs

- `src/combat/action_query/types.rs`

## Expected Output

- `Tutti i test integration verdi senza cambi di semantica`
- `Nessuna regressione su Digimon legacy`

## Verification

cargo test --features windowed
