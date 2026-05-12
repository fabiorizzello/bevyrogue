---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T04: Cascade rename src/combat/* (8 file)

Sostituire occorrenze legacy in: src/combat/speed.rs (1), battery_loop.rs (1), rng.rs (1), observability.rs (1), kernel.rs (1), turn_system/mod.rs (7), turn_system/tests.rs (11). Mappare ogni token a Heated/Chilled/Paralyzed/Slowed coerentemente con la mappa di T03. Lasciare commenti '// canon §H.1' su siti non triviali (es. switch su StatusKind). Niente cambio logica.

## Inputs

- `StatusKind da T01`

## Expected Output

- `Zero referenze legacy non-reserved in src/combat/*`

## Verification

cargo check (default + windowed) verde dopo questo task.
