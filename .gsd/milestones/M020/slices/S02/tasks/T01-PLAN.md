---
estimated_steps: 1
estimated_files: 14
skills_used: []
---

# T01: Rimuovere shim pub use legacy e riallineare call-site al path canonico blueprints::<name>::<Type>

Refactor compiler-driven a tre fasi in un unico task: (1) estendere i `mod.rs` dei blueprint Agumon/Patamon/Dorumon con `pub use identity::{...}` mirati così che i path canonici `combat::blueprints::<name>::<Type>` esistano e coesistano temporaneamente con gli shim; (2) rimuovere i tre `pub use blueprints::<name>::identity as <alias>` da `src/combat/mod.rs` (linee ~87-106) inclusi i doc-comment che li descrivono; (3) riallineare gli 11 call-site noti — 2 in `src/` (gabumon.rs, observability.rs) e 9 in `tests/` (twin_core_*, holy_support_*, predator_*) — sostituendo `crate::combat::twin_core::X` con `crate::combat::blueprints::agumon::X` (e analoghi per patamon/dorumon, usando `bevyrogue::` invece di `crate::` nei test). Le tre fasi non possono essere separate senza spezzare la compilazione: vanno svolte come singola unità, sfruttando il compilatore come oracle dei call-site da fixare. Simboli da re-esportare (verificati su identity.rs): agumon → `TwinCoreState, TwinCoreDesignTag, TwinCoreHook, twin_core_added_tag_transition, twin_core_design_tag, twin_core_design_tag_name, classify_twin_core_tag, apply_twin_core_transitions_system` (+ le const TAG_*); patamon → `HolySupportState, HolySupportSnapshot, HolySupportHook, HolySupportDesignTag, holy_support_added_tag_transition, holy_support_design_tag, holy_support_design_tag_name, classify_holy_support_tag, apply_holy_support_transitions_system` (+ const + il tipo `HolySupportTransition` se referenziato dai test — verificarne presenza in identity.rs: in caso di assenza alias va fixato come tipo presente sotto altro nome o aggiunto al re-export); dorumon → `PredatorLoopState, PredatorLoopSnapshot, PredatorTargetSnapshot, PredatorLoopHook, PredatorLoopDesignTag, PredatorLoopRequestKind, apply_predator_loop_transition, apply_predator_loop_transitions_system` (+ DEFAULT_*). Nota: `HolySupportTransition` referenziato dai test non appare nei `pub` di identity.rs — verificare `rg "HolySupportTransition" src/combat/blueprints/patamon/` per localizzarlo (può essere ri-export o tipo annidato). Costanti: i call-site attuali non sembrano usare le TAG_* via shim, ma includerle nel re-export non costa nulla e blinda la slice. Pitfall: non confondere `crate::` (usato in `src/`) con `bevyrogue::` (usato in `tests/` perché extern crate). Nessuna API runtime cambia; il bus reactive di S01 resta intatto.

## Inputs

- `src/combat/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/identity.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/patamon/identity.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/observability.rs`
- `tests/twin_core_mechanics.rs`
- `tests/twin_core_integration.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/predator_loop_kernel.rs`
- `.gsd/milestones/M020/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/observability.rs`
- `tests/twin_core_mechanics.rs`
- `tests/twin_core_integration.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/predator_loop_kernel.rs`

## Verification

cargo check && cargo check --features windowed && cargo test && bash -c '! rg -n "combat::twin_core|combat::holy_support|combat::predator_loop" src tests'

## Observability Impact

Nessun impatto runtime — il refactor è puramente lessicale. Compile-time: i path legacy diventano introvabili (errore E0433 se reintrodotti). Health signal: `rg` sui pattern legacy esce con codice 1. Test baseline (≥673) deve restare immutata; qualsiasi delta indica regressione semantica non attesa.
