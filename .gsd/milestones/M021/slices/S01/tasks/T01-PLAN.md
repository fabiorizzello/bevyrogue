---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Create src/combat/api/ skeleton — primitive types (Intent, Registry, SignalBus, Clock, RNG)

Introdurre il modulo src/combat/api/ con i tipi base del framework, niente wiring Bevy (solo Resource markers). Definisce CastId(NonZeroU32)+ROOT, Intent enum chiuso ~18 variant (incl. BlueprintSignal/SetBlueprintState/Reject), trait ExtPoint + Registry<E> + ExtRegistries Resource (7 assi placeholder), SignalBus Resource scaffold, Clock enum, CombatRng SplitMix64 deterministico. Vincolo: nessun import bevy::winit/render/bevy_egui in src/combat/api/. Unit test brevi inline per Registry lookup (hit/miss) e RNG determinism.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/M021-ROADMAP.md`
- `.gsd/DECISIONS.md`
- `src/combat/mod.rs`

## Expected Output

- `src/combat/api/mod.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/clock.rs`
- `src/combat/api/rng.rs`
- `src/combat/mod.rs`

## Verification

cargo check (headless) + cargo check --features windowed puliti. rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ → 0. cargo test --lib api::registry::tests e cargo test --lib api::rng::tests verdi. rg 'pub mod api' src/combat/mod.rs → 1.

## Observability Impact

Nessun impact runtime. Tipi additivi.
