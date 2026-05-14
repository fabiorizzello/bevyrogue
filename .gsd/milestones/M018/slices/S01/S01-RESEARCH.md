# M018 / S01 — Research

**Date:** 2026-05-13
**Slice:** Time-manipulation split: `AdvanceTurn` / `DelayTurn` con cap ±50% per chiamata e clamp `[0, 200]` dopo somma.

## Summary

Oggi il progetto modella la manipolazione del turn order con un **unico effetto signed**: `Effect::TurnAdvance(i32)` nel DSL skills (`src/data/skills_ron.rs:174`), con convenzione `+ = advance, − = delay`. Lo stesso convoglia in un unico evento bus `CombatEventKind::TurnAdvance { target, amount_pct }` (`src/combat/events.rs:94`). Il sistema applicatore `apply_turn_advance_system` (`src/combat/turn_system/mod.rs:676`) chiama `resistance::apply_av_change` (`src/combat/resistance.rs:56`) che:
- converte `pct → raw AV` come `pct * MAX_AV / 100` (no per-call cap);
- applica `TempoResistance` (curva 100/50/25%) **solo ai delay**;
- clampa AV in basso a `−MIN_ACTION_THRESHOLD_AV = −15_000` e in alto a `MAX_AV = 10_000` (`src/combat/av.rs:6`).

S01 deve **split semantico**: due variants enum distinte (`AdvanceTurn(u32)` / `DelayTurn(u32)` con magnitudo non-negativa), **per-call cap ±50%** enforced in codice (non in design doc), e **clamp [0, 200]** sull'AV risultante (interpretazione: `[0, 2*MAX_AV] = [0, 20_000]`, allargando l'attuale ceiling MAX_AV e sostituendo la floor `−15_000`). Tutti i caller residui (`TurnAdvance` signed + path Slowed) devono migrare; nessun accumulator pre-cap deve sopravvivere.

L'effort è **chirurgico ma trasversale**: enum DSL + resolver + event bus + 3 punti di emit (resolution + pipeline Slowed branch + Effect dispatch) + 1 system applicatore + log/observability variants. Risk reale: rompere la regressione M017 (`tests/status_slowed_delay.rs`, `tests/tempo_resistance.rs`) se la conversione signed→split non preserva semantica delay (incluso `TempoResistance`-only-on-delay). Verifica boundary: cap 50, cap −50, somma 60+60 → clamp, AV preexistente alto + advance → ceiling 20_000.

## Recommendation

**Decomposizione consigliata (per il planner):**

1. **DSL split**: aggiungere `Effect::AdvanceTurn(u32)` e `Effect::DelayTurn(u32)` accanto a `TurnAdvance(i32)`. Mantenere `TurnAdvance` come **shim deprecato** *solo* fino al completamento della migrazione caller; rimuoverlo prima della chiusura della slice (success criterion: "nessun caller residuo usa il vecchio TurnAdvance signed"). Stesso pattern usato per blueprint signals in M017 (MEM001).
2. **Event bus split**: due varianti `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` e `DelayTurn { target, amount_pct: u32 }`. Niente signed. Cap ±50 enforced **al sito di emissione** (nel resolver) — clamp esplicito con `min(50)` prima di costruire l'evento, così la primitive bus è già normalizzata.
3. **AV applicatore unificato**: una funzione `apply_av_change(av, resistance, kind: AvOp)` dove `AvOp = Advance(u32) | Delay(u32)`. Conserva la curva `TempoResistance` (delay-only) e cambia i clamp a `[0, 2*MAX_AV]`. `MIN_ACTION_THRESHOLD_AV` va deprecato/rimosso (nuovo floor = 0). **Attenzione**: M017 si appoggia a floor negativo per impedire infinite-delay lock; con floor 0 il blocco antidelay-infinito è invece dato dalla curva `TempoResistance` (25% asintotico sui boss) + cap ±50% per chiamata. Vale la pena nota di rationale.
4. **Pipeline migration**: il branch Slowed in `src/combat/turn_system/pipeline.rs:758` emette oggi `TurnAdvance { amount_pct: -30 }` — sostituire con `DelayTurn { amount_pct: 30 }`. Identico per i due call sites in `src/combat/resolution.rs:364, 371` (defender advance + self advance). `Effect::SelfAdvance(i32)` (`src/data/skills_ron.rs:178`) può rimanere signed-positive (semantica già non-delay) ma andrebbe normalizzato in `u32` per coerenza — propongo di toccarlo in S01 solo se gratuito, altrimenti farlo in S04 (selectors estesi) per non gonfiare la slice.
5. **Log/observability**: `LogEntry::TurnAdvance` (`src/combat/log.rs:32`) e `ValidationLogEntry::TurnAdvance` (`src/combat/observability.rs:107, 221, 804`) vanno splittate in due variants speculari. Il JSONL logger deve riflettere il rename.
6. **Test boundary**: nuovo file `tests/turn_advance_split.rs` (naming funzionale, da CLAUDE.md) con casi: (a) `DelayTurn(80)` → enforced cap 50 → AV change `−50% * MAX_AV = −5000`; (b) `AdvanceTurn(80)` → enforced cap 50; (c) doppio `AdvanceTurn(50) + AdvanceTurn(50)` su AV=10_000 → AV 20_000 (ceiling clamp tiene); (d) terzo advance non muove AV oltre 20_000; (e) `DelayTurn(50)` su AV=2000 → AV 0 (no overflow negativo); (f) Slowed regression: prima applicazione ancora produce un `DelayTurn{30}` event con stesso AV outcome (5000 → 2000).
7. **CLI scenario**: estendere `src/bin/combat_cli.rs` (oggi consuma `TurnAdvanced`) con uno scenario script che stampa l'AV gauge step-by-step prima/dopo ogni advance/delay; JSONL una entry per applicazione.

## Implementation Landscape

### Key Files

- `src/data/skills_ron.rs:174-178` — `Effect::TurnAdvance(i32)` e `Effect::SelfAdvance(i32)`. Aggiungere `AdvanceTurn(u32)` / `DelayTurn(u32)`; deprecare `TurnAdvance` signed prima della fine slice.
- `src/combat/events.rs:92-97` — `CombatEventKind::TurnAdvance { target, amount_pct }`. Split in `AdvanceTurn` / `DelayTurn` con `amount_pct: u32`.
- `src/combat/resistance.rs:33-72` — `compute_av_change` / `apply_av_change`. Riscrivere su `AvOp` enum o due funzioni dedicate; **rimuovere uso di `MIN_ACTION_THRESHOLD_AV`** (floor 0); aggiungere ceiling a `2*MAX_AV`.
- `src/combat/av.rs:6-50` — `MAX_AV`, `ActionValue::advance/delay/self_advance`. Methods esistenti già fanno saturating; serve aggiornare `advance` per ceiling `2*MAX_AV` (oggi `MAX_AV`).
- `src/combat/resolution.rs:144-172, 363-375` — extractors `skill_turn_advance` / `skill_self_advance` + emit. Sostituire con due extractor per i nuovi variants e gating cap ±50 al sito di emissione.
- `src/combat/turn_system/pipeline.rs:758-762` — branch Slowed first-apply emette signed `TurnAdvance{-30}`. Sostituire con `DelayTurn{30}`. Anche le copie del match arm in `pipeline.rs:359-368, 666-674` (log push) vanno raddoppiate.
- `src/combat/turn_system/mod.rs:676-696` — `apply_turn_advance_system`. Diventa `apply_av_ops_system` con match su entrambi i kinds.
- `src/combat/log.rs:32` + `src/combat/observability.rs:107, 221, 804` + `src/combat/jsonl_logger.rs` — variants speculari e mapping JSONL.
- `src/ui/combat_panel.rs:649` (feature `windowed`) — rendering log entry; aggiornare entrambe le variants.
- `src/combat/state.rs:44-48` — `ResolvedAction.turn_advance_pct: i32`, `self_advance_pct: i32`. Split in due campi `u32` `advance_pct` / `delay_pct` *oppure* lasciare i campi attuali ma trattarli come `u32` post-cap (cleaner: split esplicito).
- `assets/data/skills.ron` — qualsiasi skill con `TurnAdvance(±N)` va migrata a `AdvanceTurn(N)` / `DelayTurn(N)`. Tutte le occorrenze a oggi sono in `skills.ron` (`rg "TurnAdvance" assets/`).
- `tests/status_slowed_delay.rs`, `tests/tempo_resistance.rs` — regressioni M017; aggiornare i match arm su event kind ma mantenere outcome numerico identico.
- `src/bin/combat_cli.rs:193, 958` — scenario CLI: aggiungere un sotto-comando o flag `--scenario advance-delay-cap` per il demo richiesto dalla DoD slice.

### Build Order

1. **Prima: cap ±50% pure-logic, no Bevy.** Definire la funzione `clamp_pct(pct) -> u32 = pct.min(50)` e i due `apply_advance(av, pct_capped)` / `apply_delay(av, pct_capped, resistance)`. Test pure-logic boundary (cap 50/−50, somma → ceiling 20_000, floor 0). Questo prova la matematica prima di toccare event bus.
2. **Dopo: enum split DSL + event** (compatta, mecccanica), guidata dal compilatore. Il pattern shim signed→split tiene M017 in vita finché tutti i caller non sono migrati.
3. **Terzo: pipeline Slowed migration**, gated dal test `status_slowed_delay.rs` aggiornato che vuole `DelayTurn{30}` come event ma AV `5000 → 2000` invariato.
4. **Ultimo: CLI scenario** — pura presentation, nessun gameplay nuovo.

### Verification Approach

- `cargo check` — guida la migrazione caller (compiler-driven).
- `cargo test --test turn_advance_split` — nuovi test boundary (cap, clamp, multi-call).
- `cargo test --test status_slowed_delay --test tempo_resistance` — regressione M017 verde.
- `cargo test` full — assicura nessuna regressione sui 40 binari (zero shift in TTK scenarios, status_amp, follow-up, etc.).
- `cargo run --bin combat_cli -- --scenario advance-delay-cap` — manual: il JSONL output deve mostrare AV gauge step-by-step e gli eventi `AdvanceTurn`/`DelayTurn` con cap applicato.

## Don't Hand-Roll

| Problema | Esistente | Perché usarlo |
|----------|-----------|----------------|
| Clamp AV su somma | `i32::clamp(0, 2*MAX_AV)` | stdlib; nessuna ragione per scrivere helper custom |
| Resistance curve | `TempoResistance::multiplier()` (`resistance.rs:17`) | già coperto da `tempo_resistance.rs`; va solo richiamato dal nuovo path delay-only |
| Cursor su event bus | `Messages::get_cursor_current()` (vedi `status_slowed_delay.rs:158`) | pattern già usato nei test; replicarlo per i nuovi boundary test |

## Constraints

- **Headless first** (CLAUDE.md): nessun hook su `windowed`. Il rename log entry deve essere gated `#[cfg(feature = "windowed")]` solo nel render UI, non nel data flow.
- **Determinismo** (CLAUDE.md): test senza wall-clock, senza RNG non-seeded. `apply_turn_advance_system` non usa RNG e va bene così.
- **`TempoResistance` semantica delay-only** (M017 baseline): la curva si applica **solo** sui `DelayTurn`, mai sugli `AdvanceTurn`. Test esistente `compute_av_change_advance_bypasses_resistance` lo verifica — preservare.
- **`MIN_ACTION_THRESHOLD_AV` rimozione**: oggi il floor `−15_000` evita infinite-delay lock. Con floor 0 il vincolo è strutturalmente diverso (un'unità a AV 0 può comunque accumulare via Speed). Verificare che `tempo_resistance.rs::apply_av_change_clamps_to_min_action_threshold` e `apply_av_change_clamps_without_resistance_too` siano aggiornati al nuovo invariant (floor 0). Documentare la rationale (cap ±50 + curva resistance + `Speed` accumulator = no lock).
- **Status taxonomy invariata** (success criterion M018): nessun cambio agli `StatusEffectKind` o alle loro tick semantics; solo l'output del Slowed branch cambia (event variant rename).

## Common Pitfalls

- **Doppio emit pre-cap**: tentazione di clampare in `apply_av_change` invece che al sito di emissione. Il design slice vuole "no accumulator AV pre-cap": cap deve essere **al boundary del DSL** (resolver) — l'event bus e il sistema applicatore vedono solo pct ≤ 50.
- **`SelfAdvance` lasciato signed**: `Effect::SelfAdvance(i32)` accetta valori negativi oggi (mai usato in assets). Confermare in S01 se va normalizzato `u32` o se è scope S04. Raccomandazione: lascia signed in S01, ma fail-fast in resolver se `< 0` con `OnActionFailed`.
- **`ResolvedAction` shape change** è breaking per i ~15 test che istanziano `ResolvedAction { turn_advance_pct: 0, self_advance_pct: 0, … }` (vedi tests/dorumon_blueprint.rs, presentation_metadata_boundary.rs, ecc.). Il planner deve prevedere un sweep meccanico — preferibile mantenere i nomi dei campi (zero-default ancora valido) o rinominare in blocco con un singolo edit.
- **Shim `TurnAdvance` signed dimenticato in `skills.ron`**: tutti i RON skill data vanno migrati; build green con shim shim non basta — success criterion vuole "zero caller residui".

## Open Risks

- **Ceiling change `MAX_AV → 2*MAX_AV`**: `ActionValue::is_ready()` controlla `>= MAX_AV` (`av.rs:43`). Con AV che può salire a 20_000, **un'unità doppio-advanced agisce subito al prossimo tick**, ma il reset post-turn la riporta a 0 — comportamento desiderato. Confermare però che `advance_turn_system` (`turn_system/mod.rs:654-668`) ordina i `units_ready` per AV desc e che AV `19_000 > 10_000` non rompa il tie-break (sort esistente è stable su AV desc).
- **JSONL schema break**: consumer downstream (test? CLI viewer?) che fanno match sulla stringa `TurnAdvance` rompono con `AdvanceTurn`/`DelayTurn`. Rg veloce: `rg "\"TurnAdvance\"" tests/ src/` per individuare hard-coded JSON keys.
- **UI feature `windowed`**: `combat_panel.rs:649` ha pattern match su `LogEntry::TurnAdvance`; aggiornarlo o `cargo check --features windowed` rompe.

## Skills Discovered

| Tecnologia | Skill | Status |
|------------|-------|--------|
| Bevy ECS | `bevy` (`/home/fabio/.agents/skills/bevy/SKILL.md`) | installed |
| Rust idiom | `rust-best-practices`, `rust-testing` | installed |

Nessuna skill esterna identificata come necessaria oltre a quelle già installate.

## Sources

- M017 foundation: `.gsd/milestones/M017/M017-ROADMAP.md`, `tests/status_slowed_delay.rs`, `tests/tempo_resistance.rs` — definiscono la baseline `Slowed → TurnAdvance(−30)` e la curva `TempoResistance` che S01 deve preservare.
- Decisione M011/S08 di tenere `SelfAdvance` separato da `TurnAdvance` (defender-targeted vs attacker-targeted): `.gsd/milestones/M011/slices/S08/tasks/T03-SUMMARY.md` — la stessa logica vale per il nuovo split, ma il discriminatore semantico ora è `Advance` vs `Delay` (segno), non `Self` vs `Defender` (target).
- Design intent: `.gsd/milestones/M018/M018-ROADMAP.md` (success criteria, slice vision).
