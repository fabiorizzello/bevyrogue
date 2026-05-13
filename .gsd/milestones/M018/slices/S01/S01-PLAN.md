# S01: Time-manipulation split: AdvanceTurn / DelayTurn con cap ±50% e clamp [0,200]

**Goal:** Sostituire la primitiva `Effect::TurnAdvance(i32)` con un split semantico `AdvanceTurn(u32)` / `DelayTurn(u32)`, enforced in codice con per-call cap ±50% e AV clamp finale `[0, 2*MAX_AV]`. Tutti i caller (resolver, pipeline Slowed M017, skills.ron, log/observability, JSONL, UI windowed) migrano alle nuove variant; nessun caller residuo usa il signed `TurnAdvance` a fine slice. `TempoResistance` continua ad applicarsi solo sui `DelayTurn`. Demo via CLI scenario `--scenario advance-delay-cap` che stampa AV gauge step-by-step + JSONL one-entry-per-applicazione.
**Demo:** Test headless deterministici per cap/clamp boundary + regressione Slowed (M017) che continua a funzionare con la nuova primitive. CLI scenario advance/delay print AV gauge step-by-step.

## Must-Haves

- `Effect::AdvanceTurn(u32)` e `Effect::DelayTurn(u32)` sono le sole primitive di time-manipulation nel DSL; `Effect::TurnAdvance(i32)` rimosso, zero occorrenze in `src/` e `assets/`.
- `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` e `DelayTurn { target, amount_pct: u32 }` sostituiscono `TurnAdvance`. `amount_pct` è sempre ≤ 50 al sito di emissione (cap enforced nel resolver, no accumulator AV pre-cap).
- Applicatore AV ha floor `0` e ceiling `2 * MAX_AV = 20_000`. `MIN_ACTION_THRESHOLD_AV` rimosso o deprecato. `TempoResistance` curva continua ad applicarsi solo al Delay path.
- M017 Slowed branch emette `DelayTurn{30}`; AV outcome identico (5000→2000) — `tests/status_slowed_delay.rs` e `tests/tempo_resistance.rs` verdi dopo aggiornamento ai nuovi event match.
- Nuovo `tests/turn_advance_split.rs` con boundary cases: cap 50/−50, doppio advance 50+50 → ceiling 20_000, delay > AV → floor 0, terzo advance no-op oltre ceiling.
- `cargo check` + `cargo check --features windowed` + `cargo test` full suite verdi. Zero regressioni sui 40 binari esistenti.
- CLI: `cargo run --bin combat_cli -- --scenario advance-delay-cap` produce JSONL leggibile con AV gauge pre/post per ogni applicazione + cap rilevato.

## Proof Level

- This slice proves: Boundary test file dedicato (`turn_advance_split.rs`) + regressione M017 verde + CLI scenario eseguibile con JSONL output.

## Integration Closure

M017 status Slowed pipeline continua a funzionare con la nuova primitive (event variant rename, AV outcome invariato). Status taxonomy invariata. ResolvedAction shape change propagata a tutti i test che la istanziano.

## Verification

- Log variants `LogEntry::TurnAdvance` e `ValidationLogEntry::TurnAdvance` splittate in `AdvanceTurn` / `DelayTurn`. JSONL schema keys aggiornati specularmente — qualsiasi tool downstream che fa match su `"TurnAdvance"` va aggiornato (rg-sweep incluso). UI `combat_panel.rs` (windowed) rendering aggiornato.

## Tasks

- [x] **T01: AV applicatore: cap ±50% + clamp [0, 2*MAX_AV], split Advance/Delay pure-logic** `est:M`
  Riscrivere il livello di AV math come funzioni pure (no Bevy, no event bus): split `apply_advance(av, pct)` / `apply_delay(av, pct, resistance)` con cap interno ≤50 (defensive) e clamp finale [0, 20_000]. Floor 0 rimpiazza `MIN_ACTION_THRESHOLD_AV`. `TempoResistance` curva resta delay-only. `ActionValue::advance` ceiling sale da MAX_AV a 2*MAX_AV; `is_ready()` invariato (>= MAX_AV). Inline `#[cfg(test)] mod tests` con casi boundary basici.
  - Files: `src/combat/resistance.rs`, `src/combat/av.rs`
  - Verify: cargo check && cargo test --lib resistance && cargo test --test tempo_resistance (after T03 update); pure-logic boundaries verified via inline #[cfg(test)] tests in resistance.rs.

- [ ] **T02: DSL + event bus + resolver: split AdvanceTurn/DelayTurn(u32), rimuovi TurnAdvance(i32)** `est:L`
  Aggiungere `Effect::AdvanceTurn(u32)` e `Effect::DelayTurn(u32)` in `src/data/skills_ron.rs`. Aggiungere `CombatEventKind::AdvanceTurn { target, amount_pct: u32 }` e `DelayTurn { target, amount_pct: u32 }` in `src/combat/events.rs`. Riscrivere `apply_turn_advance_system` (in `src/combat/turn_system/mod.rs`) come `apply_av_ops_system` con match sui due nuovi event kinds — usa le funzioni di T01. Nel resolver (`src/combat/resolution.rs`) sostituire `skill_turn_advance` extractor con due extractor (`skill_advance` / `skill_delay`) che applicano `pct.min(50)` PRIMA di costruire l'evento (cap al sito di emissione, no pre-cap accumulator). Aggiornare `ResolvedAction` in `src/combat/state.rs`: split `turn_advance_pct: i32` in `advance_pct: u32` + `delay_pct: u32`. **Rimuovere `Effect::TurnAdvance(i32)` e `CombatEventKind::TurnAdvance` completamente** (no shim residuo). Sweep meccanico dei test che istanziano `ResolvedAction` (~15 file in tests/) — defaultano 0/0 con i nuovi campi.
  - Files: `src/data/skills_ron.rs`, `src/combat/events.rs`, `src/combat/resolution.rs`, `src/combat/state.rs`, `src/combat/turn_system/mod.rs`, `src/combat/log.rs`, `src/combat/observability.rs`, `src/combat/jsonl_logger.rs`
  - Verify: cargo check && rg -n 'TurnAdvance' src/ assets/ tests/ deve mostrare zero occorrenze del vecchio signed `Effect::TurnAdvance` o `CombatEventKind::TurnAdvance`; solo `AdvanceTurn` / `DelayTurn`.

- [ ] **T03: Pipeline Slowed migration + skills.ron + UI windowed + M017 regression tests** `est:M`
  Pipeline branch Slowed in `src/combat/turn_system/pipeline.rs:758` emette ora `DelayTurn{amount_pct: 30}` invece del signed `TurnAdvance{-30}`. Stesso refactor per le copie del match arm a `pipeline.rs:359` e `:666` (log push). Migrare `assets/data/skills.ron`: ogni `TurnAdvance(N)` → `AdvanceTurn(N)`; ogni `TurnAdvance(-N)` → `DelayTurn(N)`. Aggiornare `src/ui/combat_panel.rs:649` (feature `windowed`) per match sui due nuovi `LogEntry::AdvanceTurn` / `DelayTurn`. Aggiornare `tests/status_slowed_delay.rs` e `tests/tempo_resistance.rs` ai nuovi event variant + invariant floor 0 (rimuovere/rinominare `apply_av_change_clamps_to_min_action_threshold` se necessario, mantenendo coverage equivalente).
  - Files: `src/combat/turn_system/pipeline.rs`, `assets/data/skills.ron`, `src/ui/combat_panel.rs`, `tests/status_slowed_delay.rs`, `tests/tempo_resistance.rs`
  - Verify: cargo check && cargo check --features windowed && cargo test --test status_slowed_delay && cargo test --test tempo_resistance — entrambi verdi con event match aggiornato ma AV outcome (5000→2000) invariato.

- [ ] **T04: Boundary test suite: tests/turn_advance_split.rs** `est:M`
  Nuovo file di integration test (naming funzionale, vedi CLAUDE.md). Casi deterministici headless: (a) `DelayTurn(80)` enforcement → cap a 50 → AV change `-5000` con AV iniziale `MAX_AV` → AV 5000; (b) `AdvanceTurn(80)` → cap 50 → AV change `+5000`; (c) doppio `AdvanceTurn(50)` su AV=10_000 → AV 20_000 (ceiling); (d) terzo `AdvanceTurn(50)` non muove AV oltre 20_000 (no overflow); (e) `DelayTurn(50)` su AV=2000 → AV 0 (no negative); (f) `DelayTurn(50)` su boss con `TempoResistance(0.25)` riduce raw delay del 75% (curva preservata); (g) cap NON applicato a livello evento — verificare che l'event emesso ha già `amount_pct ≤ 50`. Usare `Messages::get_cursor_current()` pattern come in `status_slowed_delay.rs:158`.
  - Files: `tests/turn_advance_split.rs`
  - Verify: cargo test --test turn_advance_split — tutti i casi verdi, deterministico su 3 run consecutivi.

- [ ] **T05: CLI scenario advance-delay-cap + full-suite regression gate** `est:M`
  Estendere `src/bin/combat_cli.rs` con scenario `--scenario advance-delay-cap`: spawna 2 unità, esegue una sequenza scriptata (e.g. AdvanceTurn(50), AdvanceTurn(50), DelayTurn(80), DelayTurn(50)) e stampa AV gauge step-by-step pre/post + JSONL one-entry-per-applicazione (event kind + amount_pct capped + AV pre/post). Eseguire `cargo test` full suite come gate finale: zero regressioni sui 40 binari esistenti, status taxonomy M017 invariata, TTK scenarios stabili.
  - Files: `src/bin/combat_cli.rs`
  - Verify: cargo run --bin combat_cli -- --scenario advance-delay-cap (esit 0 + JSONL leggibile su stdout con cap visibile); cargo test full passes; cargo check --features windowed passes.

## Files Likely Touched

- src/combat/resistance.rs
- src/combat/av.rs
- src/data/skills_ron.rs
- src/combat/events.rs
- src/combat/resolution.rs
- src/combat/state.rs
- src/combat/turn_system/mod.rs
- src/combat/log.rs
- src/combat/observability.rs
- src/combat/jsonl_logger.rs
- src/combat/turn_system/pipeline.rs
- assets/data/skills.ron
- src/ui/combat_panel.rs
- tests/status_slowed_delay.rs
- tests/tempo_resistance.rs
- tests/turn_advance_split.rs
- src/bin/combat_cli.rs
