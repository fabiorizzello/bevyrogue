---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T01: Plumbing UltGaugeMetadata + Energy in snapshot/query

Estendere ResolveActorsQuery/UnitQuerySnapshot per esporre Option<UltGaugeMetadata> e Option<Energy>; aggiornare build_snapshot_from_ecs_with_sp e tutti i callsite units_data (UI combat panel, CLI bootstrap/dashboard, preview cache, follow_up). Non cambia ancora il comportamento - solo plumbing.

## Inputs

- `src/combat/ult_gauge.rs`
- `src/combat/mechanics/energy.rs`

## Expected Output

- `UnitQuerySnapshot espone gauge_meta e energy`
- `Tutti i call sites compilano senza cambi di comportamento`
- `Test esistenti restano verdi`

## Verification

cargo check --features windowed && cargo test --features windowed --test action_query --test windowed_only
