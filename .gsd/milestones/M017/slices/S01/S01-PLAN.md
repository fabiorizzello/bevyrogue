# S01: Enum rewrite + RON migration + tests cascade

**Goal:** Atomic rewrite della status taxonomy: rimpiazzare Burn/Freeze/Shock/DeepFreeze con i 5 status canon (Heated/Chilled/Paralyzed/Slowed/Blessed) + reserved Burn/Shock gas-era no-op, migrare RON e tutti i call site (src + tests) in modo che `cargo check` e `cargo test` (full headless suite) restino verdi a fine slice. Niente nuove semantiche per-status (quelle entrano in S03-S05) — solo wiring vocabolario + struct shape + apply/refresh/tick scheletro compatibile con i test esistenti aggiornati ai nuovi nomi.
**Demo:** `cargo check` + `cargo test` full suite verdi senza referenze alla vecchia tassonomia. `grep -r 'Burn\|Freeze\|Shock\|DeepFreeze' src/ tests/` non trova match (eccetto Burn/Shock reserved).

## Must-Haves

- cargo check (default) e cargo check --features windowed verdi
- cargo test (full integration suite) verde
- grep -rE '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/ assets/ trova match SOLO per Burn/Shock variant reserved §H.1 documentate
- Enum StatusKind contiene Heated, Chilled, Paralyzed, Slowed, Blessed + Burn (reserved), Shock (reserved)
- Effect::ApplyStatus accetta i 5 nuovi id RON; loader rifiuta id legacy con errore chiaro
- Nessuna regressione di comportamento osservabile (test suite non-status passa identica)

## Proof Level

- This slice proves: cargo check + cargo test full integration suite + grep guard sul vocabolario legacy

## Integration Closure

Tutti i moduli che referenziavano la vecchia tassonomia (speed, battery_loop, rng, observability, kernel, turn_system, status_effect, skills_ron) usano i nuovi varianti. Tutti i 7 test file migrati. RON migrato. Niente shim né alias legacy in produzione (zero debt).

## Verification

- JSONL log e ValidationSnapshot continuano a emettere status names — i nomi cambiano da Burn/Freeze/Shock/DeepFreeze a Heated/Chilled/Paralyzed/Slowed/Blessed. La normalizzazione completa del vocabolario log e snapshot diff è scope di S06; in S01 basta che i nuovi nomi compaiano senza vecchie referenze.

## Tasks

- [x] **T01: Enum rewrite + apply/refresh/tick skeleton** `est:2-3h`
  Riscrivere StatusKind in src/combat/status_effect.rs sostituendo Burn/Freeze/Shock/DeepFreeze con Heated, Chilled, Paralyzed, Slowed, Blessed. Aggiungere Burn e Shock come reserved §H.1 (varianti dichiarate ma senza effetto attivo, documentate inline). Mantenere apply/refresh/tick come scheletro coerente: apply inserisce (target,kind) single-instance, re-apply applica refresh_max_dur (max(old.dur, new.dur)), tick decrementa e drop a 0. Nessuna semantica per-status (amp%, skip, delay, +Ult) — quelle in S03-S05. Aggiornare BuffKind se serve. Eventuali helper di pattern match aggiornati. Niente shim legacy.
  - Files: `src/combat/status_effect.rs`
  - Verify: cargo check (default + windowed) compila. Lo step T05 garantirà cargo test verde.

- [x] **T02: Effect::ApplyStatus RON schema + validator** `est:1h`
  Aggiornare src/data/skills_ron.rs: Effect::ApplyStatus accetta i 5 id canon ('heated', 'chilled', 'paralyzed', 'slowed', 'blessed'). Validator a load-time rigetta id legacy ('burn_v0', 'freeze_v0', etc.) con messaggio chiaro che indica i 5 id validi. Eventuali costanti id collegate riscritte.
  - Files: `src/data/skills_ron.rs`
  - Verify: cargo check verde. Test di parsing RON esistenti continuano (saranno aggiornati ai nuovi id in T03/T05).

- [x] **T03: RON migration (assets/data/skills.ron + units.ron)** `est:1h`
  Sostituire tutti i status id legacy in assets/data/skills.ron (11 occorrenze) e assets/data/units.ron (3 occorrenze) con i 5 id canon. Mappa di traduzione di lavoro: Burn->Heated, Freeze->Chilled, Shock->Paralyzed, DeepFreeze->Slowed. Blessed entra solo dove un buff offensivo è già modellato (probabile zero match attuali). Verifica che ogni occorrenza sia status id (non skill name come 'baby_flame'). Niente cambio di durate/numeri.
  - Files: `assets/data/skills.ron`, `assets/data/units.ron`
  - Verify: cargo run --bin combat_cli (smoke) carica i RON senza loader error. Test di parsing RON in T05 confermano.

- [ ] **T04: Cascade rename src/combat/* (8 file)** `est:1-2h`
  Sostituire occorrenze legacy in: src/combat/speed.rs (1), battery_loop.rs (1), rng.rs (1), observability.rs (1), kernel.rs (1), turn_system/mod.rs (7), turn_system/tests.rs (11). Mappare ogni token a Heated/Chilled/Paralyzed/Slowed coerentemente con la mappa di T03. Lasciare commenti '// canon §H.1' su siti non triviali (es. switch su StatusKind). Niente cambio logica.
  - Files: `src/combat/speed.rs`, `src/combat/battery_loop.rs`, `src/combat/rng.rs`, `src/combat/observability.rs`, `src/combat/kernel.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/tests.rs`
  - Verify: cargo check (default + windowed) verde dopo questo task.

- [ ] **T05: Cascade rename tests/* (7 file)** `est:2-3h`
  Aggiornare le referenze nei file: tests/status_effect_apply.rs (6), status_effect_integration.rs (11), status_effect_turn_tick.rs (11), combat_coherence.rs (8), status_accuracy.rs (6), follow_up_chains.rs (1), form_identity.rs (2). Stessa mappa T03/T04. Per i test 'status_effect_*' che assertano semantica per-status (DoT, skip turn, ecc): aggiornare i nomi, lasciare la semantica TODO con #[ignore] se i nuovi varianti non hanno ancora behavior (entra in S03-S05). Documentare ogni #[ignore] con commento '// S03 — Heated DoT amp%' style.
  - Files: `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/combat_coherence.rs`, `tests/status_accuracy.rs`, `tests/follow_up_chains.rs`, `tests/form_identity.rs`
  - Verify: cargo test --no-fail-fast: tutti i test non-ignored verdi, count ignored ≤ N documentato nel summary slice.

- [ ] **T06: Grep guard + smoke run + summary** `est:30min`
  Eseguire grep -rE '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/ assets/ e verificare che gli unici match residui siano (a) la variant reserved Burn/Shock in status_effect.rs con commento '// reserved §H.1' (b) eventuali docs/ esclusi. cargo run --bin combat_cli smoke headless. Documentare nel SUMMARY.md la lista ignored test con la slice S0N target.
  - Files: `.gsd/milestones/M017/slices/S01/S01-SUMMARY.md`
  - Verify: grep guard ok, cargo check + cargo test full passa, smoke CLI runs.

## Files Likely Touched

- src/combat/status_effect.rs
- src/data/skills_ron.rs
- assets/data/skills.ron
- assets/data/units.ron
- src/combat/speed.rs
- src/combat/battery_loop.rs
- src/combat/rng.rs
- src/combat/observability.rs
- src/combat/kernel.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/tests.rs
- tests/status_effect_apply.rs
- tests/status_effect_integration.rs
- tests/status_effect_turn_tick.rs
- tests/combat_coherence.rs
- tests/status_accuracy.rs
- tests/follow_up_chains.rs
- tests/form_identity.rs
- .gsd/milestones/M017/slices/S01/S01-SUMMARY.md
