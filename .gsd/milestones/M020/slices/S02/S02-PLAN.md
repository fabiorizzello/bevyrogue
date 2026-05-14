# S02: Rimozione shim pub use legacy (twin_core / holy_support / predator_loop)

**Goal:** Eliminare i tre `pub use` shim Digimon-specific (`twin_core`, `holy_support`, `predator_loop`) da `src/combat/mod.rs` riallineando ogni call-site al path canonico `combat::blueprints::<name>::<Type>`, lasciando il combat module privo di alias legacy prima dell'inizio di M021.
**Demo:** cargo test passa; grep ricorsivo mostra zero occorrenze di combat::twin_core, combat::holy_support, combat::predator_loop fuori dai file blueprint stessi

## Must-Haves

- Zero `pub use` shim residui in `src/combat/mod.rs` (twin_core/holy_support/predator_loop rimossi insieme ai loro doc-comment).
- I tre `blueprints/<name>/mod.rs` riesportano i tipi previamente shimmati via `pub use identity::{...}` mirati.
- Tutti gli 11 call-site noti (2 in `src/`, 9 in `tests/`) compilano con il path canonico `combat::blueprints::<name>::<Type>`, senza esporre il segmento `identity::`.
- `cargo check` headless e `cargo check --features windowed` chiudono senza nuovi warning.
- `cargo test` ≥673 verde (baseline post-S01).
- `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` non restituisce match (eventuali occorrenze in commenti dentro `identity.rs` documentate come stale-but-harmless o ripulite).

## Proof Level

- This slice proves: final-assembly: il combat module risulta concretamente privo dei tre shim e tutti i call-site reali girano sul path canonico — verificato compilando entrambe le feature configuration e l'intera suite di integrazione.

## Integration Closure

Upstream consumati: i tre `blueprints/<name>/identity.rs` (sorgente canonica dei simboli) e `combat/mod.rs` (sede degli shim da rimuovere). Wiring nuovo: re-export espliciti `pub use identity::{...}` nei tre `blueprints/<name>/mod.rs`. Dopo questa slice, M020 è chiuso end-to-end: bus reactive completo (S01) + combat namespace pulito (S02) — il kernel è pronto per M021 senza alias legacy né emit gap.

## Verification

- Nessun impatto runtime: refactor puramente lessicale. Il bus eventi e i logger JSONL restano invariati; nessun nuovo signal o sink. Il segnale di salute è statico (compile-time + grep): il successo è osservabile come exit-1 di `rg` sui pattern legacy e come baseline test count immutata.

## Tasks

- [x] **T01: Rimuovere shim pub use legacy e riallineare call-site al path canonico blueprints::<name>::<Type>** `est:1h`
  Refactor compiler-driven a tre fasi in un unico task: (1) estendere i `mod.rs` dei blueprint Agumon/Patamon/Dorumon con `pub use identity::{...}` mirati così che i path canonici `combat::blueprints::<name>::<Type>` esistano e coesistano temporaneamente con gli shim; (2) rimuovere i tre `pub use blueprints::<name>::identity as <alias>` da `src/combat/mod.rs` (linee ~87-106) inclusi i doc-comment che li descrivono; (3) riallineare gli 11 call-site noti — 2 in `src/` (gabumon.rs, observability.rs) e 9 in `tests/` (twin_core_*, holy_support_*, predator_*) — sostituendo `crate::combat::twin_core::X` con `crate::combat::blueprints::agumon::X` (e analoghi per patamon/dorumon, usando `bevyrogue::` invece di `crate::` nei test). Le tre fasi non possono essere separate senza spezzare la compilazione: vanno svolte come singola unità, sfruttando il compilatore come oracle dei call-site da fixare. Simboli da re-esportare (verificati su identity.rs): agumon → `TwinCoreState, TwinCoreDesignTag, TwinCoreHook, twin_core_added_tag_transition, twin_core_design_tag, twin_core_design_tag_name, classify_twin_core_tag, apply_twin_core_transitions_system` (+ le const TAG_*); patamon → `HolySupportState, HolySupportSnapshot, HolySupportHook, HolySupportDesignTag, holy_support_added_tag_transition, holy_support_design_tag, holy_support_design_tag_name, classify_holy_support_tag, apply_holy_support_transitions_system` (+ const + il tipo `HolySupportTransition` se referenziato dai test — verificarne presenza in identity.rs: in caso di assenza alias va fixato come tipo presente sotto altro nome o aggiunto al re-export); dorumon → `PredatorLoopState, PredatorLoopSnapshot, PredatorTargetSnapshot, PredatorLoopHook, PredatorLoopDesignTag, PredatorLoopRequestKind, apply_predator_loop_transition, apply_predator_loop_transitions_system` (+ DEFAULT_*). Nota: `HolySupportTransition` referenziato dai test non appare nei `pub` di identity.rs — verificare `rg "HolySupportTransition" src/combat/blueprints/patamon/` per localizzarlo (può essere ri-export o tipo annidato). Costanti: i call-site attuali non sembrano usare le TAG_* via shim, ma includerle nel re-export non costa nulla e blinda la slice. Pitfall: non confondere `crate::` (usato in `src/`) con `bevyrogue::` (usato in `tests/` perché extern crate). Nessuna API runtime cambia; il bus reactive di S01 resta intatto.
  - Files: `src/combat/mod.rs`, `src/combat/blueprints/agumon/mod.rs`, `src/combat/blueprints/patamon/mod.rs`, `src/combat/blueprints/dorumon/mod.rs`, `src/combat/blueprints/gabumon.rs`, `src/combat/observability.rs`, `tests/twin_core_mechanics.rs`, `tests/twin_core_integration.rs`, `tests/holy_support_affordance.rs`, `tests/holy_support_mechanics.rs`, `tests/holy_support_resolution.rs`, `tests/patamon_blueprint_seam.rs`, `tests/dorumon_predator_runtime.rs`, `tests/predator_loop_kernel.rs`
  - Verify: cargo check && cargo check --features windowed && cargo test && bash -c '! rg -n "combat::twin_core|combat::holy_support|combat::predator_loop" src tests'

## Files Likely Touched

- src/combat/mod.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/blueprints/patamon/mod.rs
- src/combat/blueprints/dorumon/mod.rs
- src/combat/blueprints/gabumon.rs
- src/combat/observability.rs
- tests/twin_core_mechanics.rs
- tests/twin_core_integration.rs
- tests/holy_support_affordance.rs
- tests/holy_support_mechanics.rs
- tests/holy_support_resolution.rs
- tests/patamon_blueprint_seam.rs
- tests/dorumon_predator_runtime.rs
- tests/predator_loop_kernel.rs
