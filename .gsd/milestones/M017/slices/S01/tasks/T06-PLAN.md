---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T06: Grep guard + smoke run + summary

Eseguire grep -rE '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/ assets/ e verificare che gli unici match residui siano (a) la variant reserved Burn/Shock in status_effect.rs con commento '// reserved §H.1' (b) eventuali docs/ esclusi. cargo run --bin combat_cli smoke headless. Documentare nel SUMMARY.md la lista ignored test con la slice S0N target.

## Inputs

- `T01-T05 chiusi`

## Expected Output

- `Slice DoD soddisfatto: zero legacy refs, suite verde, ignored test triaged`

## Verification

grep guard ok, cargo check + cargo test full passa, smoke CLI runs.
