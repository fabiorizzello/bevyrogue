---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Enum rewrite + apply/refresh/tick skeleton

Riscrivere StatusKind in src/combat/status_effect.rs sostituendo Burn/Freeze/Shock/DeepFreeze con Heated, Chilled, Paralyzed, Slowed, Blessed. Aggiungere Burn e Shock come reserved §H.1 (varianti dichiarate ma senza effetto attivo, documentate inline). Mantenere apply/refresh/tick come scheletro coerente: apply inserisce (target,kind) single-instance, re-apply applica refresh_max_dur (max(old.dur, new.dur)), tick decrementa e drop a 0. Nessuna semantica per-status (amp%, skip, delay, +Ult) — quelle in S03-S05. Aggiornare BuffKind se serve. Eventuali helper di pattern match aggiornati. Niente shim legacy.

## Inputs

- `docs/future_design_draft/02-08_effect_cascade.md §H.1, §H.4, §H.5`
- `.gsd/DECISIONS.md D004 + D009`

## Expected Output

- `StatusKind enum riscritto`
- `apply/refresh/tick scheletro compatibile con S02 policy work`

## Verification

cargo check (default + windowed) compila. Lo step T05 garantirà cargo test verde.
