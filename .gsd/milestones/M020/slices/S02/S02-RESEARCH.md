# S02: Rimozione shim pub use legacy (twin_core / holy_support / predator_loop) — Research

**Date:** 2026-05-14

## Summary

Refactor lineare e a basso rischio: rimuovere tre `pub use blueprints::<name>::identity as <alias>` da `src/combat/mod.rs` e ribilanciare gli 11 call-site (2 in `src/`, 9 in `tests/`) sui path canonici. La decisione architetturale è già fissata in M020-CONTEXT: ogni blueprint mod (`agumon/mod.rs`, `patamon/mod.rs`, `dorumon/mod.rs`) espone i tipi previamente shimmati via `pub use identity::<Type>` al livello del modulo blueprint, così che i call-site usino `combat::blueprints::<name>::<Type>` (path corto, niente `identity::` esposto). Nessuna API runtime cambia; il bus reactive ottenuto in S01 resta intatto.

Build order ovvio: prima estendere i `mod.rs` dei tre blueprint con i `pub use identity::*` mirati, poi togliere i tre `pub use` shim da `combat/mod.rs`, poi fix dei call-site (compilatore-driven). Verifica: `cargo check` headless + windowed, `cargo test` (≥673), e `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` deve restituire zero match fuori dai file blueprint stessi.

## Recommendation

**Approccio compiler-driven a tre fasi**, in un singolo task:

1. **Espandi i blueprint mod**: aggiungi a `agumon/mod.rs` `pub use identity::{TwinCoreState, TwinCoreDesignTag, twin_core_added_tag_transition, twin_core_design_tag}`; a `patamon/mod.rs` `pub use identity::{HolySupportState, HolySupportTransition}`; a `dorumon/mod.rs` `pub use identity::{PredatorLoopState, PredatorLoopSnapshot, PredatorTargetSnapshot}`. Verifica i nomi esatti prima della modifica (vedi Key Files).
2. **Rimuovi i tre `pub use` shim** da `src/combat/mod.rs` (linee 87–105 circa, inclusi i comment block che li descrivono).
3. **Riallinea gli 11 call-site** ai path canonici `crate::combat::blueprints::<name>::<Type>` (in `src/`) e `bevyrogue::combat::blueprints::<name>::<Type>` (in `tests/`).

Si tratta di un'unica unità di lavoro: la rimozione degli shim e il fix dei call-site non possono essere separati senza spezzare la compilazione. Un solo task T01 ("Rimozione shim e riallineamento call-site") è la decomposizione corretta. Nessun seam intermedio giustifica due task.

## Implementation Landscape

### Key Files

- `src/combat/mod.rs` — sede dei tre `pub use` shim (linee 87, 95, 102). Vanno rimossi insieme ai comment block che li descrivono (linee 86–105 nel complesso). I `pub mod battery_loop;` e `pub mod precision_mind_game;` interleavati restano.
- `src/combat/blueprints/agumon/mod.rs` — aggiungere `pub use identity::{...}` per i tipi previamente esposti via `twin_core`. Da verificare prima della modifica: `rg "^(pub )?(fn|struct|enum|const) " src/combat/blueprints/agumon/identity.rs` per la lista completa.
- `src/combat/blueprints/patamon/mod.rs` — analogo per Holy Support (`HolySupportState`, `HolySupportTransition`).
- `src/combat/blueprints/dorumon/mod.rs` — analogo per Predator Loop (`PredatorLoopState`, `PredatorLoopSnapshot`, `PredatorTargetSnapshot`).
- `src/combat/observability.rs` — 3 call-site: `:300` (TwinCoreState), `:303` (HolySupportState), `:441` (PredatorTargetSnapshot).
- `src/combat/blueprints/gabumon.rs` — 1 call-site: `:3` importa `TwinCoreDesignTag` + `twin_core_added_tag_transition`.
- 9 file in `tests/`:
  - `tests/twin_core_mechanics.rs:9`, `tests/twin_core_integration.rs:13,256`, `tests/holy_support_affordance.rs:3,12`, `tests/holy_support_mechanics.rs:4,18`, `tests/holy_support_resolution.rs:5`, `tests/patamon_blueprint_seam.rs:4`, `tests/dorumon_predator_runtime.rs:9`, `tests/predator_loop_kernel.rs:9`.

### Build Order

1. **Verifica esatta dei simboli da re-esportare**: `rg "^pub (struct|enum|fn|const) " src/combat/blueprints/{agumon,patamon,dorumon}/identity.rs` per costruire la lista canonica. Punto di rischio principale (e unico) della slice: dimenticare un simbolo causa errori di compilazione a catena ma facilmente fixabili.
2. **Modifica i tre `blueprints/<name>/mod.rs`** aggiungendo i `pub use identity::*` mirati. A questo punto i path canonici esistono e i path-shim coesistono — la build resta verde.
3. **Rimuovi i tre `pub use` shim** da `combat/mod.rs`. La build ora rompe esattamente sugli 11 call-site noti.
4. **Riallinea i call-site** uno-a-uno (compiler-driven). Niente surprise: i target sono già conosciuti dal `rg` iniziale.
5. **Verifica finale**: `cargo check`, `cargo check --features windowed`, `cargo test`, `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` (atteso: zero match, modulo eventuali comment dentro `identity.rs` che riferiscono il vecchio nome — innocui, ma verificare).

### Verification Approach

- `cargo check` (headless): zero errori, zero warning nuovi.
- `cargo check --features windowed`: zero errori, zero warning nuovi.
- `cargo test`: ≥673 test verdi (baseline post-S01).
- `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests`: exit 1 / zero match. Acceptance criterion esplicito da M020-ROADMAP.
- Smoke optional: `cargo run --features windowed` per verificare che il binario non rompa al boot (improbabile dato che il refactor è puramente lessicale).

## Constraints

- **Headless first** (CLAUDE.md): nessuna nuova dipendenza fuori dal feature-gate `windowed`. Il refactor non aggiunge import esterni.
- **Path canonico fissato in M020-CONTEXT** (Architectural Decision #2): `combat::blueprints::<name>::<Type>`. NON usare `combat::blueprints::<name>::identity::<Type>` (espone struttura interna senza beneficio).
- **No `#[serde(alias)]` o legacy compat** (Architectural Decision #3 in S01, principio applicabile qui): clean break, nessun keep-alias shim.
- **MEM001**: `follow_up.rs` mantiene una sua `ResolveActorsQuery` locale. Questa slice non tocca componenti né query — vincolo informativo, non operativo.

## Common Pitfalls

- **Simboli mancanti nel re-export**: se `identity.rs` espone una `fn pub` o `const pub` che un call-site usa e dimentichiamo nel `pub use identity::{...}`, l'errore di compilazione è chiaro ma rallenta. Mitigazione: lo step 1 della build order (rg sui simboli `pub`).
- **Comment block orfani**: i tre `pub use` shim in `combat/mod.rs` sono preceduti da doc-comment multilinea che spiegano lo shim. Rimuoverli insieme al `pub use`; lasciarli come dangling commenta su `pub mod battery_loop;` sarebbe confondente.
- **Differenza `crate::` vs `bevyrogue::`**: i call-site in `src/` usano `crate::combat::...`; i test in `tests/` usano `bevyrogue::combat::...` (extern crate name). Non confondere.
- **Riferimenti testuali in commenti `identity.rs`**: dopo la rimozione, vecchi commenti tipo "vedi twin_core::..." potrebbero diventare stale ma non bloccano la build. Verificare con `rg` finale; opzionale lasciarli per non gonfiare la slice.

## Open Risks

Nessuno significativo. Refactor lessicale puro, zero cambi di semantica runtime, baseline post-S01 stabile (673 test verdi).

## Skills Discovered

Nessuno skill aggiuntivo necessario — refactor Rust di routine, tutti gli skill rilevanti (`bevy-ecs-expert`, `verify-before-complete`, `lint`) sono già disponibili a livello user.
