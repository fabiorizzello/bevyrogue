# M018: Time-manipulation split + TargetShape resolver expansion

**Gathered:** 2026-05-13
**Status:** Ready for planning (Q2 + Q3 carried forward as planning-phase decisions)

## Project Description

Roguelite RPG monster-taming turn-based (Rust + Bevy 0.18, headless-first). M018 ships due foundation primitives che sbloccano la maggior parte delle skill identity del roster in arrivo:

1. **Time-manipulation split** — `TurnAdvance` (signed) viene splittato in due variants enum distinte: `AdvanceTurn(pct)` e `DelayTurn(pct)`.
2. **TargetShape resolver expansion** — oltre `Single`, supporto a `Blast`, `AoE(All)`, `Bounce(N)`, con tie-break deterministico su `slot_index` asc e selectors estesi.

## Why This Milestone

Le skill identity di gran parte del roster (Renamon Tōhakken, Kitsune Grace, Patamon AoE, Tentomon Bounce…) richiedono targeting non-`Single` e manipolazione esplicita del turn order. La foundation `Slowed → TurnAdvance` introdotta in M017 va refactorata per evitare l'accumulator AV pre-cap e per esporre semantica advance/delay distinta a livello di DSL skill RON. Senza queste due primitive il roster post-M018 non è esprimibile in modo deterministico.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Eseguire da CLI scenario una skill che applica `AdvanceTurn(pct)` o `DelayTurn(pct)` e vedere il turn order ricalcolato passo-passo nel JSONL log.
- Eseguire da CLI scenario una skill `Blast` (primary + slot_index ±1), `AoE(All)`, o `Bounce(N)` e osservare target list + ordine di applicazione damage stabile su run multipli (seeded).

### Entry point / environment

- Entry point: CLI scenario binary (`combat_cli`) — `cargo run -- scenario <name>`
- Environment: headless (default feature set, no `windowed`)
- Live dependencies involved: none (RNG seedato per Random selector)

## Completion Class

- **Contract complete means:** integration test headless in `tests/` coprono advance/delay split, resolver shapes (Blast/AoE/Bounce), selectors estesi, con boundary deterministici e seed esplicito.
- **Integration complete means:** CLI scenario binary esegue skill di esempio per ciascuna primitive; JSONL log leggibile mostra target list, AV step-by-step, e turn order risultante.
- **Operational complete means:** N/A — M018 è pure-logic, nessun lifecycle ops.

## Final Integrated Acceptance

Per dichiarare M018 completo dobbiamo dimostrare:

- Scenario CLI: skill advance/delay applicata, turn order ricalcolato, JSONL log stampa AV gauge step-by-step e turn order risultante stabile su run ripetuti.
- Scenario CLI: skill `Bounce(N=3)` con un enemy che muore al hop 2 — la chain ricalcola hop 3 sui survivors mantenendo tie-break `slot_index` asc, JSONL log mostra sequenza hop completa con stato vivo/morto per hop.
- Scenario CLI: skill `Blast` con adiacente KO'd — comportamento definito (absorb vs skip, da decidere in S02) eseguito deterministicamente e documentato in fixture.
- Zero regressioni sui ~40 binari di test esistenti; status taxonomy M017 invariata.

## Architectural Decisions

### Time-manipulation split senza cap/clamp globale

**Decision:** `TurnAdvance` (signed) viene rimpiazzato da due variants distinte `AdvanceTurn(pct)` e `DelayTurn(pct)`. **Niente cap ±50% e niente clamp [0,200]** globali — l'AV resta nel range raw esistente (`MAX_AV` / `MIN_ACTION_THRESHOLD_AV` in `src/combat/resistance.rs` + `src/combat/turn_system/mod.rs:676`). Eventuali cap per-unit sono attribuiti a **passive boss-specifiche** (estensione di `TempoResistance`), non a una regola globale.

**Rationale:** L'utente ha esplicitamente respinto il cap globale ("non serve il cap all'advance o delay") e ha indicato che eventuali attenuazioni saranno **passive di singoli boss**. Il bilanciamento resta nei numeri delle skill RON, non in una saturation function nel kernel.

**Alternatives Considered:**
- Cap ±50% + clamp [0,200] globale (come scritto nel roadmap SC#2) — scartato dall'utente.
- Reinterpretare `[0,200]` come gauge unit nuovo che sostituisce `MAX_AV` — scartato (AV resta raw).

> Override esplicito del Success Criterion #2 del roadmap: il roadmap dice "cap ±50% e clamp [0,200] enforced in codice"; il design definitivo elimina questa regola. **Aggiornare il roadmap SC#2 in fase di planning.**

### Determinismo via `slot_index` come tie-break canonico

**Decision:** Tutte le forme `TargetShape` non-`Single` (Blast/AoE/Bounce) e tutti i selectors estesi usano `slot_index` ascendente come tie-break deterministico. La rappresentazione concreta di `slot_index` (nuovo `Component` persistente vs. alias dell'iteration order / `Unit.id`) è demandata a S02 in planning.

**Rationale:** Determinismo è constraint hard (CLAUDE.md: "tests devono essere deterministici, no wall-clock, no RNG senza seed"). Avere un tie-break canonico evita drift JSONL tra run.

**Alternatives Considered:**
- Tie-break su `Unit.id` raw — fragile se IDs cambiano con bootstrap.
- Tie-break random seedato — peggiora la leggibilità del log JSONL.

## Error Handling Strategy

Il resolver `TargetShape` deve essere **total** (mai panic): KO'd target spillover, target list vuota, slot_index out-of-range → comportamento deterministico definito (Q2 da risolvere in S02 planning). Selectors seeded → RNG ricavato da seed combat-state, mai wall-clock. Eventuali errori RON validation restano al loader (`src/data/skills_ron.rs`), non al resolver. CLI scenario binary fail-fast su skill malformate (loader error), non sul resolver.

## Risks and Unknowns

- **`slot_index` introduction (Q3 — da risolvere in S02):** `rg slot_index` returns zero hits in `src/` e `tests/`. Decidere se è nuovo `Component` (bootstrap wiring + durable identity) o alias dell'iteration order corrente / `Unit.id`. Impatta surface S02 e i fixture JSONL.
- **Blast/Bounce con KO'd unit (Q2 — da risolvere in S02):** absorb (spillover perso) vs skip al prossimo vivo? Bounce(N) selector per hop ≥2 (lowest-HP-pct / seeded random / next-slot)? Stesso bersaglio ripetibile in chain o no? Impatta testabilità deterministica e fixture JSONL di S02/S03.
- **`TempoResistance` boss-only extension:** estensione/refactor del component esistente per esprimere cap per-unit; rischio di drift rispetto al passive design futuro (post-M018 roster).
- **Roadmap drift SC#2:** il roadmap originale codifica cap ±50% / clamp [0,200] che il design definitivo elimina; planning deve aggiornare ROADMAP.md.

## Existing Codebase / Prior Art

- `src/combat/resistance.rs` — definisce `MAX_AV = 10_000`, `MIN_ACTION_THRESHOLD_AV = -15_000`, `TempoResistance` (1.0/0.5/0.25 boss attenuation). Sito dell'eventuale estensione per-unit cap.
- `src/combat/turn_system/mod.rs:676` — sito attuale della AV application; entry point per il refactor advance/delay split.
- `src/data/skills_ron.rs:9` — `TargetShape` enum corrente (solo `Single`); va esteso con `Blast` / `AoE(All)` / `Bounce(N)`.
- `src/combat/resolution.rs` — applicazione effetti skill; consumer del nuovo resolver.
- `src/combat/bootstrap.rs` — sito di assegnazione `slot_index` se diventa Component persistente.
- M017 SUMMARY — `Slowed → TurnAdvance` foundation che va refactorata qui senza accumulator pre-cap.
- `assets/data/skills.ron` — RON sorgente per le skill di esempio del CLI scenario.

## Relevant Requirements

- (da popolare in planning con cross-ref a `.gsd/REQUIREMENTS.md` per skill identity roster; il roster post-M018 dipende direttamente dalle primitive di M018.)

## Scope

### In Scope

- Split enum `AdvanceTurn(pct)` / `DelayTurn(pct)` con refactor dei caller residui di `TurnAdvance` signed.
- TargetShape resolver: `Single` + `Blast` + `AoE(All)` + `Bounce(N)`, tie-break `slot_index` asc.
- Selectors: `AdjLowest`, `LowestHpPctAlive`, `RandomEnemyAlive{seed}`, `SingleAlly`.
- CLI scenario + JSONL log step-by-step per advance/delay e Bounce chain.
- Estensione `TempoResistance` per cap **per-unit boss-tagged** (non globale), se necessaria al success criterion.
- Aggiornamento ROADMAP.md SC#2 (rimozione cap globale).

### Out of Scope / Non-Goals

- Cap ±50% / clamp [0,200] globale (rimosso esplicitamente).
- UI egui per visualizzare turn gauge (windowed feature gate, fuori scope M018).
- Skill identity finali del roster post-M018 (vengono in milestone successivi, sopra queste primitive).
- Nuovi status effects oltre quelli M017 (taxonomy invariata).

## Technical Constraints

- **Headless first:** ogni system deve girare senza feature `windowed`.
- **Determinismo:** no wall-clock, RNG sempre seedato esplicitamente.
- **Zero regressioni:** ~40 binari di test esistenti devono restare verdi; status taxonomy M017 invariata.
- **Skill DSL:** estensioni a `TargetShape` e selectors passano per `src/data/skills_ron.rs` (schema RON canonico).
- **Eventi:** `CombatEvent` resta single-source-of-truth; resolver scrive eventi, non muta stato direttamente.

## Integration Points

- `src/combat/turn_system/mod.rs` — sito refactor advance/delay split.
- `src/data/skills_ron.rs` + `src/combat/resolution.rs` — sito resolver TargetShape + selectors.
- `assets/data/skills.ron` — skill di esempio per CLI scenario (advance/delay, Blast, AoE, Bounce).
- `combat_cli` scenario runner — entry point user-visible per le primitive.
- `src/combat/jsonl_logger.rs` — log JSONL deve riportare target list, AV step, hop chain.

## Testing Requirements

- Integration test headless in `tests/` per:
  - advance/delay split (boundary AV alti/bassi, no cap globale verificato).
  - TargetShape resolver: Blast spillover, AoE iteration order, Bounce chain con KO mid-chain.
  - Selectors: LowestHpPct tie-break con `slot_index`, Random seedato (run ripetuto → stesso output).
- Regressione M017: `Slowed → TurnAdvance` continua a funzionare riconvertito su `AdvanceTurn` / `DelayTurn`.
- CLI smoke test: scenario scriptato per ciascuna primitive, JSONL log diff-stabile su run ripetuti.
- Naming funzionale (CLAUDE.md): `target_shape_blast_spillover.rs`, `bounce_chain_ko_midchain.rs`, ecc. — NO naming `sNN_*`.

## Acceptance Criteria

(da definire per-slice in fase di planning; gathering rimandato a S01–S04 plan. Ogni slice produrrà i propri criteri testabili, ancorati ai Final Integrated Acceptance qui sopra.)

## Open Questions

- **Q2 — Blast/Bounce con KO'd unit:** absorb (spillover perso) o skip al prossimo vivo? Bounce(N) selector hop ≥2 (lowest-HP-pct / seeded random / next-slot)? Stesso bersaglio ripetibile in chain o no? *Carry-forward to S02 planning — risolvere prima di scrivere fixture JSONL.*
- **Q3 — `slot_index` representation:** nuovo `Component` persistente al bootstrap (durable identity) o alias dell'iteration order corrente / `Unit.id`? *Carry-forward to S02 planning — decide impatta su bootstrap wiring e fixture.*
- **Roadmap SC#2 drift:** SC#2 codifica cap ±50% / clamp [0,200] che il design definitivo elimina. *Aggiornare ROADMAP.md in fase di planning slice 0.*
