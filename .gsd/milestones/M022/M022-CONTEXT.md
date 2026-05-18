# M022: Asset pipeline — clip + anim_graph loader/validator/hot-reload (Agumon-only)

**Gathered:** 2026-05-18
**Status:** Ready for planning

## Project Description

Validare lo schema dei due asset di animazione del modello UE5 a 2 file (canon §2.2 / §2.2b) su **un solo Digimon (Agumon)**: `clip.ron` (geometria frame, lossless dal `_atlas.json`) e `anim_graph.ron` (grafo FSM orchestratore). Consegnare i `RonAssetPlugin`-based loader, il validator statico §L, e hot-reload funzionante. **Nessun runtime player** (`tick_fsm`, Command→KernelEffect): è M023.

## Why This Milestone

M023 ("prima volta che il combat gira sullo schermo") costruisce il render + AnimGraph runtime **sopra** questi asset. Se lo schema `clip.ron`/`anim_graph.ron` è sbagliato, va scoperto qui — su loader+validator headless-testabili su 1 Digimon — non a metà del milestone visual più rischioso del piano. M022 è prerequisito secco di M023 (DAG portfolio) ed è l'unico nodo sbloccato (M021 ✅).

## User-Visible Outcome

### When this milestone is complete, the user can:

- Eseguire `cargo test` e vedere i contract test del validator §L verdi: un `anim_graph.ron` plausibile-ma-rotto (entry mancante, edge dangling, frame fuori bounds, param-ref inesistente, StartQTE senza headless default) viene **rifiutato al boot**, non a runtime.
- Eseguire `cargo run --features windowed` con Agumon: `clip.ron` + `anim_graph.ron` caricati come asset tipizzati; modificare il file a runtime ricarica l'asset senza crash né world-state corrotto.
- Caricare l'Agumon `clip.ron` (generato lossless da `agumon_atlas.json`) e asserire headless che la geometria frame combacia esattamente col json sorgente.

### Entry point / environment

- Entry point: `cargo test` (loader + validator headless); `cargo run --features windowed` (hot-reload demo).
- Environment: local dev, headless-first; demo hot-reload richiede `windowed`.
- Live dependencies involved: Bevy `AssetServer` (file-watch hot-reload), `bevy_common_assets` RON loader. Nessun servizio esterno.

## Completion Class

- Contract complete means: loader produce `Clip`/`AnimGraph` tipizzati dai RON Agumon; tutti i check §L hanno un test che passa sul valido e fallisce-al-boot sul rotto; `clip.ron` Agumon è lossless vs `agumon_atlas.json`.
- Integration complete means: i due asset si combinano per nome-clip (`AnimGraph.clip` referenzia un range valido in `clip.ron`) e il validator cross-asset (`Frame range in-bounds`, `Command params reference exist`) gira su Agumon reale.
- Operational complete means: hot-reload Agumon — edit del file mentre l'app gira → asset re-emette LoadedWithDependencies, nessun panic, nessuno stato sporco.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Agumon `clip.ron` + `anim_graph.ron` (baby_flame, forma §M) caricano come asset tipizzati in un app headless e passano l'intero validator §L.
- Un fixture `anim_graph.ron` rotto per ognuno dei check §L fa fallire `App::finish()` (o il contract test equivalente) con errore tipizzato `DataError` che nomina il file e il check violato.
- Hot-reload live in `--features windowed`: modifica a caldo ricaricata senza crash (non simulabile headless — richiede il file-watcher reale).

## Architectural Decisions

### Asset naming canon: `clip.ron` + `anim_graph.ron`

**Decision:** I due file si chiamano `clip.ron` e `anim_graph.ron`, per-Digimon sotto `assets/digimon/<name>/`.

**Rationale:** Il nome `anim_graph.ron` rispecchia esattamente il tipo Rust `AnimGraph` (snake_case del tipo), rendendo la mappatura filename→tipo trasparente e coerente col pattern già usato (`clip.ron → Clip`). I nomi precedenti avevano problemi: `clipmontage.ron` era legato al modello flat list pre-FSM (§2.2 orig); `animation_fsm.ron` esponeva un dettaglio implementativo (FSM è il meccanismo, non il concetto).

**Alternatives Considered:**
- `clipmontage.ron` (canon §2.2b) — scartato: legato al framing "montage" del modello piatto precedente all'adozione del grafo.
- `animation_fsm.ron` (portfolio) — scartato: espone il meccanismo (FSM) anziché il concetto (grafo animazione); il nome non scala se l'interprete runtime cambia.

**Impatto:** I design doc in `docs/future_design_draft/` (§2.2, §2.2b) usano ancora `clipmontage.ron` — sono archivio di design, non source-of-truth operativa. L'unica source-of-truth per il naming è questo CONTEXT e il ROADMAP M022.

### Loader sul pattern `RonAssetPlugin` esistente

**Decision:** `Clip` e `AnimGraph` sono caricati via `bevy_common_assets::ron::RonAssetPlugin::<T>` + `AssetServer`, come già fanno `UnitRoster`/`SkillBook`/`PartyConfig` in `src/data/mod.rs`.

**Rationale:** Hot-reload è gratis dal file-watch dell'AssetServer (stesso meccanismo del data RON esistente). Zero loader custom da mantenere. Parse failure già tipizzata `DataError::Validation` (hardening refactor 2026-05-18).

**Alternatives Considered:**
- `AssetLoader` custom hand-rolled — scartato: duplica ciò che `RonAssetPlugin` già dà, niente vantaggio.
- `include_str!` compile-time (come lo skill-book) — scartato: no hot-reload, è esplicitamente richiesto.

### `clip.ron` generato lossless dal `_atlas.json`

**Decision:** L'Agumon `clip.ron` è derivato da `assets/digimon/agumon_atlas.json` preservando esattamente `meta` (frame_size/columns/rows/total_frames) e i range per-clip (start/end/count). Loader-side defaults: `fps`, `loop`, `texture_path`.

**Rationale:** §2.2 migrazione passo 1 ("lossless: stessa info"). Un test di equivalenza geometrica `clip.ron ≡ atlas.json` pinna la conversione e impedisce drift.

## Error Handling Strategy

Tutti i fallimenti di parse/validazione sono `DataError` tipizzati (register `src/data/error.rs`), propagati via `Result<(), BevyError>` — coerente con l'hardening 2026-05-18 (RON parse failure tipizzata, niente `panic!` su input dati). Il validator §L fallisce a `App::finish()` (boot) con messaggio che nomina file + check + nodo/edge offending. Cosmetic-only mancante non è errore. Reachability è warning, non error (dead branch utile). Hot-reload con file rotto: l'asset resta all'ultima versione valida, errore loggato, no crash.

## Risks and Unknowns

- **Forma RON dell'`AnimGraph` (nodi+edges+predicati chiusi §C/§C2/§D)** — è verbose e ricca; un parser sbagliato qui propaga in M023. Mitigato facendo S2 (parse tipizzato) presto su un caso reale (baby_flame §M), non un toy.
- **`anim_graph.ron` partecipa al gameplay (§2.2b §G)** — a differenza di §2.2 flat. M022 carica/valida solo; il rischio "FSM headless-deterministica" è retired in M023, ma lo schema deve già esporre i campi (`headless_default_param`, `modifier` in frame logici) — il validator §L li check-a.
- **Cross-asset validation** (`Command params reference exist` vs `skills.ron` Agumon) — accoppia il loader al data layer esistente; verificare che l'ordine di load non crei race (asset non ancora pronto).
- **Hot-reload non testabile headless** — richiede `--features windowed` + file-watch reale; UAT umano necessario per l'operational class.

## Existing Codebase / Prior Art

- `src/data/mod.rs` — `DataPlugin` con `RonAssetPlugin::<T>` + `load_data` + tracker LoadedWithDependencies: pattern esatto da riusare per `Clip`/`AnimGraph`.
- `src/data/error.rs` — `DataError::{Validation, TimelineCompile}`: il validator §L riusa questi varianti.
- `assets/digimon/agumon_atlas.json` — sorgente geometria per `clip.ron` Agumon (frame_size 557×561, 95 frame, 8 clip).
- `assets/data/digimon/agumon/skills.ron` — target del check cross-asset `Command params reference exist`.
- `docs/future_design_draft/02-02_animation_manifest.md` (§2.2) e `02-02b_animation_fsm.md` (§2.2b) — schema canonico, vocabolario Commands §C/§C2, predicati §D, validator §L, esempio §M. (Usano ancora il nome `clipmontage.ron` — archivio design; il nome operativo è `anim_graph.ron`.)
- `src/combat/runtime/` Timeline FSM (M021) — **non** è l'AnimGraph runtime; M022 non lo tocca. L'AnimGraph player è M023.

## Relevant Requirements

- (REQUIREMENTS.md vuoto) — M022 stabilisce il contratto asset-pipeline che le requirement visual M023+ assumeranno.

## Scope

### In Scope

- Schema tipizzato `Clip` + `RonAssetPlugin::<Clip>` loader, Agumon `clip.ron` generato lossless da `agumon_atlas.json`.
- Schema tipizzato `AnimGraph` (nodi, edges, `Predicate` §D chiuso, vocabolario `Command` §C+§C2 chiuso, `ParamRef` §S, `TargetShape` §C3) + `RonAssetPlugin::<AnimGraph>` loader; Agumon `anim_graph.ron` baby_flame (forma §M).
- Validator §L completo come contract test (`tests/anim_fsm_validation.rs`): entry exists, reachability (warn), exit reachable, dangling edges, priority unique, frame range in-bounds, command params reference exist (cross-asset vs skills.ron), StartQTE has headless_default, cancel coverage (warn).
- Hot-reload working per Agumon (demo `--features windowed`).
- I 5 Digimon non-Agumon: `anim_graph.ron` degenerate (1 nodo all-clip, §N) — stub, non popolato.

### Out of Scope / Non-Goals

- `tick_fsm` / `FsmRuntime` / AnimGraph runtime player → M023.
- `Command::translate_into_kernel_effect` (Command→KernelEffect/Notify) → M023.
- `skill_tree.ron` resolver, `effects.ron` cost/cooldown — schema riservato (§I/§J), non implementato.
- Migration script tooling (ex `tools/migrate_clipmontage_to_fsm.py`) — tooling opzionale, non gameplay slice.
- Sprite render, UI §9, phase strip — M023.
- Popolare `anim_graph.ron` reale per i 5 non-Agumon — milestone successive del roster.

## Technical Constraints

- **Headless first**: loader + validator girano in `cargo test` senza `windowed`. Solo la demo hot-reload usa `--features windowed`. Nessun import winit/wgpu/egui fuori dal feature gate.
- **Determinismo**: validator deterministico; nessun wall-clock, ordering stabile sugli edge (priority + dichiarazione).
- **Vocabolari chiusi**: `Command`, `Predicate`, `ParamRef`, `TargetShape`, `StatusKind`/`BuffKind` sono enum chiusi — nessuna `Custom(String)`. Il validator rigetta a load-time id fuori vocabolario.
- **Numeri via reference**: gli `on_enter` Command usano `ParamRef` (`Static`/`Snapshot`/`BlueprintState`/`Literal` solo soglie strutturali), mai literal di scaling — §C regola 2, validato da §L.
- Test integration in `tests/`, nomi funzionali, deterministici (KNOWLEDGE R003/R004).

## Integration Points

- Bevy `AssetServer` — file-watch hot-reload; stesso plugin/lifecycle di `src/data/mod.rs`.
- `src/data/error.rs` — il validator emette `DataError`.
- `assets/data/digimon/agumon/skills.ron` — cross-asset param-ref check.
- `clip.ron` ↔ `anim_graph.ron` — combinati per nome-clip; il validator verifica `AnimGraph.clip` + frame range contro `Clip`.

## Testing Requirements

- **Contract (headless, `tests/`)**: loader Agumon `clip.ron`/`anim_graph.ron` → asset tipizzato; equivalenza geometrica `clip.ron ≡ agumon_atlas.json`; un fixture rotto per ogni check §L che fallisce al boot con `DataError` nominante file+check.
- **Integration (headless)**: cross-asset validator (frame in-bounds, param-ref vs skills.ron) su Agumon reale; ordine di load asset deterministico.
- **Operational / UAT (windowed)**: hot-reload manuale — editare `clip.ron`/`anim_graph.ron` con app running, verificare reload senza crash/stato sporco. Non simulabile headless.

## Acceptance Criteria

Per-slice in `M022-ROADMAP.md`. Globale: loader tipizzati + validator §L completo verdi headless; hot-reload Agumon provato live; `clip.ron` Agumon lossless vs `agumon_atlas.json`.

## Open Questions

- `reverse: true` su Node (§Q.3) — speculativo nel canon. Decisione: lo schema lo accetta come campo opzionale (Agumon `recovery` lo usa nello sketch §M); semantica runtime rinviata a M023. Solo parse+validate in M022.
- Frame-range overlap tra nodi (§Q.2) — ammesso dal canon (caso override). Il validator §L `frame range in-bounds` lo permette; nessun check di non-overlap.
