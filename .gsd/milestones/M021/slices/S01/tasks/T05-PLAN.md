---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Slice verification — grep gates + full suite + decision update

Verifica finale slice. Esegue tutti i gate cargo + grep, valida che le test esistenti reggono. Cargo check headless + windowed, cargo test full suite (~74 + canary T02 + propagation T03), grep verifiers (no bevy::winit/render/bevy_egui in src/combat/ ex-blueprints, CombatEvent {} con cast_id, pub mod api, CombatPlugin in lib.rs, no register_combat_kernel_runtime in main.rs), smoke run headless + windowed (skip se DISPLAY mancante). Se emerse decisioni non in DECISIONS.md (shape IntentQueue, default seed RNG), appendi via gsd_decision_save.

## Inputs

- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `src/main.rs`
- `tests/intent_applier_canary.rs`
- `tests/cast_id_propagation.rs`

## Expected Output

- `.gsd/DECISIONS.md`

## Verification

Tutti i gate verdi. cargo test 0 fail. cargo check --features windowed 0 warning nuovi. rg verifiers come step 3 di T05-PLAN.

## Observability Impact

Nessuno — verification-only task.
