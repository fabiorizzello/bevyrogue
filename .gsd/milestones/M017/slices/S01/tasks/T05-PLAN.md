---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T05: Cascade rename tests/* (7 file)

Aggiornare le referenze nei file: tests/status_effect_apply.rs (6), status_effect_integration.rs (11), status_effect_turn_tick.rs (11), combat_coherence.rs (8), status_accuracy.rs (6), follow_up_chains.rs (1), form_identity.rs (2). Stessa mappa T03/T04. Per i test 'status_effect_*' che assertano semantica per-status (DoT, skip turn, ecc): aggiornare i nomi, lasciare la semantica TODO con #[ignore] se i nuovi varianti non hanno ancora behavior (entra in S03-S05). Documentare ogni #[ignore] con commento '// S03 — Heated DoT amp%' style.

## Inputs

- `Cascade src completo da T04`

## Expected Output

- `Test suite verde`
- `Lista #[ignore] tracciata per pickup in S03-S05`

## Verification

cargo test --no-fail-fast: tutti i test non-ignored verdi, count ignored ≤ N documentato nel summary slice.
