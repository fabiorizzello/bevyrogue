# M020: Reactive bus uniforme + shim removal

**Gathered:** 2026-05-14
**Status:** Ready for planning

## Project Description

Milestone di consolidamento del kernel combat in vista di M021 (`trait Skill` + `SkillCtx`). Due interventi indipendenti ma complementari:

1. Completare il bus reactive (`CombatEventKind`) con i variant ancora mancanti — `UltimateUsed` e `UnitDied` con payload — così che ogni listener downstream (postmortem, revenge, analytics, follow-up) abbia un segnale uniforme e informativo per le due transizioni più importanti del turno.
2. Rimuovere i tre `pub use` shim Digimon-specific (`twin_core`, `holy_support`, `predator_loop`) da `src/combat/mod.rs`, ribilanciando i call-site sui path canonici dei blueprint.

## Why This Milestone

Il bus eventi è già la single source of truth per UI/log/follow-up (P001 in KNOWLEDGE), ma due transizioni cruciali sono ancora opache: l'uso di una ultimate non emette evento dedicato, e `OnKO` è un marker payload-less che costringe i listener a re-interrogare lo stato. Allo stesso tempo, `combat/mod.rs` contiene tre `pub use` shim aggiunti durante M017–M019 per non rompere i call-site mentre i blueprint Digimon migravano nei sottomoduli; questi alias sono debito puro.

M021 introdurrà `trait Skill` + `SkillCtx`: in quel contesto i listener dei blueprint avranno bisogno di un bus uniforme (per reagire a death/ult senza rileggere componenti) e di path canonici stabili (perché `SkillCtx` esporrà query tipate sui blueprint, e gli alias legacy sono confusione gratis). Farlo ora costa poco (refactor lineare, basso rischio) e libera M021 dal dover fare housekeeping mentre introduce un'astrazione nuova.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Eseguire `cargo run --features windowed` e osservare nei log JSONL un evento `{"kind":"UltimateUsed","unit_id":...}` ogni volta che un'unità lancia la ultimate
- Eseguire `cargo run --features windowed` e osservare nei log JSONL un evento `{"kind":"UnitDied","status_remaining":[...],"heated_remaining":N}` al KO di ogni unità
- Eseguire `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` e ottenere zero match fuori dai blueprint stessi

### Entry point / environment

- Entry point: `cargo test` (verifica primaria), `cargo run` headless e `cargo run --features windowed` (smoke)
- Environment: local dev, headless first
- Live dependencies involved: nessuna

## Completion Class

- **Contract complete means:** test integration verdi (≥74 con i due nuovi), `cargo check` headless e `--features windowed` senza warning nuovi, `rg` mostra zero occorrenze degli alias shim fuori dai blueprint
- **Integration complete means:** non applicabile (refactor interno, nessuna integrazione cross-subsystem nuova)
- **Operational complete means:** non applicabile (nessun lifecycle runtime)

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Una run combat reale (`cargo run`) emette `UltimateUsed` esattamente una volta per cast e `UnitDied` con payload coerente per ogni KO osservato
- L'intera suite `cargo test` (72+ esistenti + 2 nuovi) passa deterministicamente
- Nessun file fuori da `src/combat/blueprints/<name>/` importa via `combat::twin_core` / `combat::holy_support` / `combat::predator_loop`

## Architectural Decisions

### UnitDied.status_remaining: snapshot completo, insertion order

**Decision:** `status_remaining` snapshotta **tutti i kind attivi** nel `StatusBag` al momento del KO (buff + debuff), mantenendo l'**insertion order** del bag.

**Rationale:** Listener generici post-mortem (death-gives-buff, revenge-on-debuff, postmortem analytics) traggono valore dal payload completo senza dover pre-filtrare. Insertion order è già deterministico nel `Vec<StatusInstance>` backing del bag — non serve sort canonico.

**Alternatives Considered:**
- Solo debuff — chiude porte gratis a listener su buff residui (Blessed, Atk-up)
- Sort per enum discriminant — over-engineering, l'insertion order è già stabile

### Path canonico post-shim: re-export al livello blueprint module

**Decision:** Dopo la rimozione del `pub use` da `combat/mod.rs`, ogni blueprint mod (`agumon/mod.rs`, `patamon/mod.rs`, `dorumon/mod.rs`) aggiunge `pub use identity::TwinCoreState` (e analoghi). I call-site importano come `combat::blueprints::agumon::TwinCoreState`.

**Rationale:** `identity` è dettaglio interno del blueprint, non parte dell'API che gli altri moduli devono conoscere. Path corto, zero alias top-level, nessuna ambiguità su dove vive il tipo.

**Alternatives Considered:**
- Path verbose `combat::blueprints::agumon::identity::TwinCoreState` — espone struttura interna senza beneficio
- Tenere lo shim a livello `combat::` — sconfigge lo scopo del milestone

### JSONL wire-format: clean break

**Decision:** Il discriminator serde cambia da `{"kind":"OnKO"}` a `{"kind":"UnitDied","status_remaining":[...],"heated_remaining":N}` senza `#[serde(alias = "OnKO")]`, senza migration note.

**Rationale:** Nessun consumer esterno esiste (replay tool, analytics, dashboard); il progetto è in prototipazione. Solo i test interni e `jsonl_logger.rs` consumano il wire-format. Aggiungere alias retro-compat per consumer inesistenti è cruft.

**Alternatives Considered:**
- `#[serde(alias = "OnKO")]` — speculative compat, debito immediato

---

> See `.gsd/DECISIONS.md` for the full append-only register of all project decisions.

## Error Handling Strategy

Refactor puramente interno: nessuna nuova superficie di errore runtime. Gli unici punti di attenzione sono:

- `turn_system/mod.rs:488` emette `OnKO` da un sito dove il `StatusBag` del defender potrebbe non essere in scope. Policy: se non recuperabile via la query già in scope, emettere `UnitDied { status_remaining: vec![], heated_remaining: 0 }` con commento di una riga che spiega il vincolo. Non panic, non query allargate solo per questo caso.
- `apply_damage_only` riceve già `defender_status: Option<&StatusBag>`: se `None`, payload con vettore vuoto e `heated_remaining: 0` (nessun bag = nessuno status, by construction).

## Risks and Unknowns

- **Quarto hoist block (hop loop ~line 1856 in pipeline.rs)** — emette `UltGain` ma il gate su `UltEffect::Reset` per `UltimateUsed` va riverificato in esecuzione. Una ultimate `PerHop` è insolita ma legale.
- **`turn_system/mod.rs:488` payload fidelity** — il sito potrebbe essere un death-on-poison o break-stun path; payload vuoto è il fallback accettabile, ma vale leggere il contesto prima di decidere.
- **Test JSON snapshot** — sei file di test contengono snapshot del wire-format `OnKO`; tutti devono essere aggiornati alla nuova forma. Cattura comune: dimenticare i campi default per i test senza setup status.

## Existing Codebase / Prior Art

- `src/combat/events.rs` — definizione `CombatEventKind`, sito unico per i nuovi variant
- `src/combat/resolution.rs:559-561, ~780` — emit sites di `OnKO` in `apply_damage_only`
- `src/combat/turn_system/pipeline.rs` — quattro hoist block (single-target, Blast/AllEnemies, AllAllies, hop loop) per emit `UltimateUsed` + sei match arm da rinominare per `UnitDied`
- `src/combat/turn_system/mod.rs:488` — emit OnKO da sito non-pipeline (status tick / break path)
- `src/combat/jsonl_logger.rs` — consumer del wire-format serde
- `src/combat/mod.rs` — sede dei tre `pub use` shim da rimuovere
- `src/combat/observability.rs`, `src/combat/blueprints/gabumon.rs`, ~9 file in `tests/` — call-site degli shim
- `tests/follow_up_triggers.rs`, `tests/event_stream.rs`, `tests/combat_coherence.rs`, `tests/pipeline_dispatch.rs`, `tests/toughness_enemy_only.rs` — sei snapshot JSON da aggiornare

## Relevant Requirements

- **R-COMBAT-EVENTS** (implicito, vedi P001 in KNOWLEDGE) — `CombatEvent` è il bus single source of truth per UI/log; uniformare i variant è prerequisito per i listener M021+
- **R-KERNEL-GENERICITY** (P001) — il kernel non deve esporre alias franchise-specific; M020 chiude il debito accumulato in M017–M019

## Scope

### In Scope

- `CombatEventKind::UltimateUsed { unit_id }` — definizione + emit nei 4 hoist block
- `CombatEventKind::OnKO` → `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` — rinomina + payload fill in `apply_damage_only` + rinomina match arm in pipeline e `turn_system/mod.rs`
- Due nuovi test integration: `tests/ultimate_event.rs`, `tests/unit_died_payload.rs`
- Rimozione dei tre `pub use` shim da `src/combat/mod.rs`
- Aggiunta dei `pub use identity::*` necessari nei `blueprints/<name>/mod.rs`
- Aggiornamento di tutti i call-site (observability, gabumon, test) ai path canonici
- Aggiornamento dei sei snapshot JSON nei test esistenti

### Out of Scope / Non-Goals

- `trait Skill` + `SkillCtx` (M021)
- Nuovi listener che consumano `UnitDied`/`UltimateUsed` (verranno con i blueprint M021+)
- Migrazione del wire-format JSONL con alias retro-compat
- Refactor di `apply_damage_only` oltre la modifica del payload all'emit site
- Estrazione di un helper `emit_ultimate_used_if_reset` (opzionale, decidere in T01 se i quattro siti diventano duplicati fastidiosi)

## Technical Constraints

- **Headless first** (CLAUDE.md): nessun import winit/wgpu/egui in path combat
- **Determinismo test**: no wall-clock, no RNG senza seed; usare lo stile `apply_effects` direct-call (vedi `tests/dr_pipeline.rs`) o Bevy-world quando serve driving completo (vedi `tests/ultimate_meter.rs`)
- **Single source of truth**: il pattern mutate-in-match-arm + emit-via-event-writer va preservato; payload assembly avviene a monte (in `apply_damage_only`), non duplicato nei consumer
- **MEM001**: `follow_up.rs` mantiene una sua `ResolveActorsQuery` locale; M020 non aggiunge componenti alla query principale, ma se T01/T02 richiedessero nuovi accessi, l'aggiornamento in lockstep è obbligatorio

## Integration Points

- `src/combat/jsonl_logger.rs` — consumatore del wire-format; verifica che la serializzazione del nuovo payload non rompa il logger (no custom `Serialize`, derive serde di default → safe)
- `src/ui/combat_panel.rs` (feature `windowed`) — verifica che non matchi esplicitamente `OnKO` (se sì, rinomina; presunto: legge solo via display generico)

## Testing Requirements

- Due nuovi test integration:
  - `tests/ultimate_event.rs`: drive `ActionIntent::Ultimate` con `attacker.ult.current == max`, asserisce **una sola** emissione `UltimateUsed { unit_id }` con id corretto; asserisce assenza di emissione su Basic/Skill non-Reset
  - `tests/unit_died_payload.rs`: defender con `Heated(2)` + `Slowed(1)` nel `StatusBag`, danno fatale, asserisce `UnitDied` con `status_remaining` contenente entrambi i kind in insertion order e `heated_remaining == 2`
- Aggiornamento dei sei snapshot JSON esistenti alla nuova forma `{"kind":"UnitDied","status_remaining":[],"heated_remaining":0}` (per i test senza setup status)
- `cargo test` deve passare con ≥ 74 test verdi
- `cargo check --features windowed` deve produrre zero warning nuovi

## Acceptance Criteria

**S01 — UltimateUsed + UnitDied payload:**
- `CombatEventKind::UltimateUsed { unit_id }` definito ed emesso una sola volta per cast nei quattro hoist block
- `CombatEventKind::UnitDied { status_remaining, heated_remaining }` definito; `apply_damage_only` riempie il payload dal `defender_status` in scope
- Tutti i match arm su `OnKO` (sei siti pipeline + uno in `turn_system/mod.rs`) rinominati a `UnitDied { .. }`
- `tests/ultimate_event.rs` e `tests/unit_died_payload.rs` aggiunti e verdi
- I sei snapshot JSON dei test esistenti aggiornati
- `cargo test` verde, `cargo check` headless e `--features windowed` senza warning nuovi

**S02 — Shim removal:**
- I tre `pub use` (twin_core, holy_support, predator_loop) rimossi da `src/combat/mod.rs`
- `pub use identity::*` aggiunti nei `blueprints/{agumon,patamon,dorumon}/mod.rs` per i tipi previamente esposti
- Tutti i call-site (observability.rs, gabumon.rs, ~9 test) aggiornati ai path canonici `combat::blueprints::<name>::<Type>`
- `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` restituisce zero match fuori dai blueprint stessi
- `cargo test` verde, `cargo check` headless e `--features windowed` senza warning nuovi

## Open Questions

Nessuna aperta — le tre grey-area (status_remaining shape, path canonico, wire-format break) sono state risolte in fase di intervista (vedi Architectural Decisions).
