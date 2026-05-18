---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T04: CombatPlugin extract — Bevy Plugin wrapper + Resource init + lib re-export

Spostare logica di register_combat_kernel_runtime in impl Plugin for CombatPlugin, montare Resource framework (ExtRegistries, SignalBus, Clock, CombatRng seed 0xDEADBEEF, IntentQueue, CastIdGen), registrare intent_applier exclusive, esporre CombatPlugin da lib.rs, aggiornare main.rs + bin/combat_cli.rs. Verifica rg import vietati e sposta dietro #[cfg(feature="windowed")] o in src/windowed.rs se trovati.

## Inputs

- `src/combat/kernel.rs`
- `src/main.rs`
- `src/combat/api/applier.rs`

## Expected Output

- `src/combat/plugin.rs`
- `src/combat/mod.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/bin/combat_cli.rs`

## Verification

rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/ --glob '!blueprints/**' → 0. cargo check (headless + windowed) puliti. cargo run headless boot OK. rg 'CombatPlugin' src/lib.rs → ≥1. rg 'add_plugins.*CombatPlugin' src/main.rs → 1. rg 'register_combat_kernel_runtime' src/main.rs → 0.

## Observability Impact

Nessun cambio observable; ordine system invariato vs register_combat_kernel_runtime.
