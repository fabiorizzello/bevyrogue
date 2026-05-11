# S01: Unblock action pipeline (ApplyDeferred chain)

**Goal:** Sblocca la suite di test (oggi 13 binari falliti per `step_app` mai eseguito) collassando l'action pipeline a un singolo sistema Bevy, e rendere le 4 fasi del lifecycle (Declaration → PreApp → App → Resolution) osservabili come varianti di `CombatEvent` sul bus, in modo che R070 (lifecycle multi-fase) e R071 (FIFO follow-up) siano provati end-to-end da un nuovo `tests/pipeline_dispatch.rs`.

**Demo:** `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast` riporta 21/21 binari verdi; `tests/pipeline_dispatch.rs` asserta che per un'azione root la sequenza emessa contiene `OnActionDeclared` → `OnActionPreApp` → (eventi core: `OnDamageDealt`, eventuale `OnBreak`/`OnKO`) → `OnActionApplied` → `OnActionResolved` in quell'ordine posizionale; per un trigger follow-up appare un secondo ciclo declared→resolved con `follow_up_depth = 1`.

**Must-Haves:**
- M001: tutti i 21+ binari di integration verdi (R070, R071 verificati end-to-end)
- M002: emesse 4 nuove varianti `CombatEventKind` (`OnActionDeclared { intent_kind }`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`) attorno a `step_app`, in ordine fisso, con `follow_up_depth` propagato
- M003: ordine FIFO follow-up preservato; il ciclo follow-up emette il proprio lifecycle con `follow_up_depth = 1`; `follow_up_listener_system` non scatta sui nuovi lifecycle event
- M004: `action_pipeline_system` rimosso dallo schedule (`headless.rs`, `windowed.rs`) e dagli export di modulo; `InFlightAction` non è più Bevy `Resource` (passato come valore locale tra `step_declaration` e `step_app`); `CombatState::action_stage` rimosso (o motivato se mantenuto)
- M005: `tests/pipeline_dispatch.rs` copre azione root, follow-up trigger, e caso azione fallita per SP shortfall (verifica che `OnActionResolved` sia emesso anche dopo `OnActionFailed`)
- M006: D026 cap (1-hop) lasciato in piedi in S01 (rimozione schedulata in S08 con D046)
**Demo:** cargo test passa al 100%; nuovo integration tests/pipeline_dispatch.rs esercita declared→pre→apply→resolve end-to-end via CombatEvent bus

## Must-Haves

- ## Verification
- `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | grep -E "^test result"` — ogni linea "ok"; totale ≈ 21 pass / 0 fail.
- `tests/pipeline_dispatch.rs` (nuovo): asserta sequenza lifecycle root + ciclo follow-up con depth=1 + ramo SP-shortfall (declared+failed+resolved emessi).
- `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests` clean.
- ## Requirement Impact
- Requirements touched: R070 (lifecycle 4-fasi osservabile), R071 (FIFO follow-up). Entrambi `active`, owned dalla slice.
- Re-verify: intera integration suite (21 binari) — il collapse cambia il punto di esecuzione di `step_app`, ogni asserzione su `BasicHit`/`Break`/`Ko`/`StatusApplied` deve continuare a passare.
- Decisions revisited: D026 cap follow-up resta in vigore in S01; D046 (rimozione cap) è schedulata in S08. D021 (pipeline imperativa, niente multi-stage Bevy) confermata e rinforzata da questo collapse.
- ## Proof Level
- This slice proves: integration (lifecycle events osservati end-to-end via bus su un'app Bevy reale, headless).
- Real runtime required: yes (Bevy `App::update()`).
- Human/UAT required: no — tutto gated da test automatici.
- ## Observability / Diagnostics
- Runtime signals: 4 nuovi `CombatEventKind` varianti scritti su bus + emessi via `debug!(target: "combat.events")` (già strutturato in `emit_combat_event`); `jsonl_logger_system` li scriverà automaticamente nel log JSONL (volume +4 entry per azione).
- Inspection surfaces: `tests/pipeline_dispatch.rs` come canonical contract test; `Messages<CombatEvent>::get_cursor().read()` per ispezione runtime; JSONL log per replay post-mortem.
- Failure visibility: `OnActionFailed` resta emesso dentro `step_app` per stun/SP shortfall; `OnActionResolved` emesso sempre dopo l'apply (anche su fallimento) — pattern garantito dal contratto del nuovo test.
- Redaction constraints: nessuno (no PII, single-player offline).
- ## Integration Closure
- Upstream surfaces consumed: `src/combat/turn_system/{mod.rs, pipeline.rs}`, `src/combat/follow_up.rs`, `src/combat/state.rs`, `src/combat/events.rs`, `src/headless.rs`, `src/windowed.rs`, `tests/event_stream.rs` (matcher esaustivo da estendere), `tests/validation_snapshot.rs` (snapshot di `CombatState`).
- New wiring introduced: chiamate inline `step_declaration → emit_lifecycle → step_app → emit_lifecycle` dentro `resolve_action_system` e `resolve_follow_up_action_system`; rimozione di `action_pipeline_system` dagli schedule headless/windowed.
- What remains before milestone usable: niente per S01 — sblocca tutte le slice successive (S02+) che dipendono da test verdi.

## Proof Level

- This slice proves: integration

## Integration Closure

Sblocca l'esecuzione completa della pipeline dichiarazione→apply nelle test app esistenti (oggi solo `resolve_action_system` è registrato, e dopo M010 quel sistema non chiama più `step_app`). Dopo S01 niente altro è richiesto per arrivare a milestone-usable rispetto al contratto della slice; le slice S02-S09 dipendono da questo unblock per girare i loro nuovi test.

## Verification

- 4 nuove varianti `CombatEventKind` (OnActionDeclared/OnActionPreApp/OnActionApplied/OnActionResolved) emesse attorno a `step_app` in `resolve_action_system` e `resolve_follow_up_action_system`. Visibili via `MessageReader<CombatEvent>` (test introspection), `debug!(target: "combat.events", ...)` (tracing strutturato già in `emit_combat_event`), e nel log JSONL prodotto da `jsonl_logger_system`. `follow_up_listener_system` continua a scattare solo sui core event (OnDamageDealt/OnKO/OnBreak/...) — i nuovi lifecycle event non sono pattern-matchati da `evaluate_follow_up`, quindi non causano falsi trigger.

## Tasks

- [x] **T01: Add 4 lifecycle CombatEventKind variants and update exhaustive matchers** `est:30m`
  Estendere `CombatEventKind` (src/combat/events.rs) con 4 nuove varianti dedicate al lifecycle dell'azione: `OnActionDeclared { intent_kind: ActionIntentKind }`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`. Definire un nuovo enum `ActionIntentKind { Basic, Skill, Ultimate }` (serializable, `Debug + Clone + PartialEq + Eq + serde::Serialize`) co-locato in `events.rs` (NON riusare `ActionIntent` che porta payload completi). Aggiornare il matcher esaustivo `assert!(kinds.iter().all(|kind| matches!(kind, ...)))` in `tests/event_stream.rs:200-210` per includere le 4 nuove varianti, in modo che il test continui a compilare e passare quando T02 inizia a emettere gli eventi (anche se `event_stream.rs` non li asserisce esplicitamente come 'presenti', solo come 'consentiti'). Aggiornare anche il commento DOC su D026 nel campo `follow_up_depth` di `CombatEvent` se serve chiarezza, ma NON rimuovere il guard (rimozione in S08 con D046). Verificare che `jsonl_logger.rs` e `log.rs` non abbiano `match` esaustivi sull'enum (oggi usano `_ => {}` o pattern parziali); se ne hanno, aggiungere un arm `_ => {}` di default per le nuove varianti. Skill suggerito per l'executor: `bevy-ecs-expert` (installato globalmente).
  - Files: `src/combat/events.rs`, `tests/event_stream.rs`
  - Verify: CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | tee /tmp/t01-check.log && grep -q 'OnActionDeclared' src/combat/events.rs && grep -q 'OnActionResolved' tests/event_stream.rs

- [x] **T02: Collapse resolve_action_system and resolve_follow_up_action_system to single-system flow with lifecycle events** `est:1h30m`
  Eliminare il punto di rottura della pipeline: collassare `step_declaration` → `step_app` in un unico tick di sistema, ed emettere i 4 lifecycle event attorno alle chiamate. CONCRETO: (1) Refactor `resolve_action_system` (src/combat/turn_system/mod.rs:134-162): rimuovere il parametro `inflight: Option<Res<InFlightAction>>` e il guard `inflight.is_some()`; chiamare `step_declaration` ottenendo un `Option<InFlightAction>` locale (NON `commands.insert_resource`); se `Some(inflight)`, emettere `OnActionDeclared { intent_kind }` + `OnActionPreApp` con `source/target` dell'attaccante e `follow_up_depth = 0`, chiamare `step_app(&mut commands, &inflight, ...)` direttamente, poi emettere `OnActionApplied` + `OnActionResolved`. (2) Mirror identico in `resolve_follow_up_action_system` (src/combat/follow_up.rs:283-327), ma con `follow_up_depth = 1`. (3) Rimuovere `action_pipeline_system` dallo schedule: `src/headless.rs:128` (e import a riga 23) + `src/windowed.rs:108` (e import a riga 13). (4) In `src/combat/turn_system/pipeline.rs` cancellare la funzione `action_pipeline_system` (righe 224-261) e rimuoverne l'export `pub use` da `src/combat/turn_system/mod.rs:401`. (5) `step_declaration` deve restare puro/non-emetter (NON aggiungere `event_writer.write` lì dentro — l'emissione vive nel caller per centralizzare l'ordering). (6) Rimuovere le righe `state.action_stage = ActionStage::PreApp` (pipeline.rs:55) e tutte le `state.action_stage = ActionStage::None` da `step_app` (pipeline.rs:74,106,146,220) — non più rilevanti. NON toccare `InFlightAction` come tipo (resta struct, anche se non più Resource — la definizione viene aggiornata in T03). NON toccare il guard `event.follow_up_depth >= 1` di `follow_up_listener_system` (rimozione in S08). Verificare che `step_app` mantenga la sua firma corrente (è chiamata anche da `resolve_follow_up_action_system`). Quality-gate: dopo T02 i 13 binari falliti devono diventare verdi (validazione parziale OK, T04 fa la verifica completa). Skill: `bevy-ecs-expert`.
  - Files: `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/follow_up.rs`, `src/headless.rs`, `src/windowed.rs`
  - Verify: CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e --test follow_up_triggers --test sp_economy --test status_effect_apply --test ultimate_meter 2>&1 | grep -E '^test result' | grep -v 'FAILED' | wc -l | xargs -I {} test {} -ge 5 && ! grep -q 'pub fn action_pipeline_system' src/combat/turn_system/pipeline.rs

- [x] **T03: Drop dead InFlightAction Resource, ActionStage variants, and CombatState::action_stage field** `est:45m`
  Pulire le primitive di stato che T02 ha reso obsolete. CONCRETO: (1) `src/combat/state.rs:53-58`: rimuovere `#[derive(Resource, ...)]` da `InFlightAction` (resta `#[derive(Debug, Clone, PartialEq, Eq)]` se serve, ma NON Resource). (2) `src/combat/state.rs:17-25`: ispezionare quali varianti di `ActionStage` sono ancora referenziate; dopo T02 nessuna lo è (solo `None` e `PreApp` erano vive, e T02 ha rimosso le scritture). Eliminare l'intero enum `ActionStage` se nessun consumatore resta. (3) `src/combat/state.rs:63-67,74,87,96,137`: rimuovere il campo `action_stage: ActionStage` da `CombatState` e tutti i suoi inizializzatori (Default, reset, test fixture). (4) `tests/validation_snapshot.rs:35`: aggiornare la fixture per rimuovere `action_stage` dal letterale di `CombatState`. (5) `src/combat/turn_system/mod.rs:144` aveva `state.action_stage != ActionStage::None` — già rimosso in T02 (parte del guard); verificare clean. `src/combat/follow_up.rs:293` stesso pattern — verificare clean. (6) `src/combat/mod.rs:32` aggiornare il commento doc del re-export se cita `ActionStage`. (7) Rimuovere import non più usati: `ActionStage` da `pipeline.rs:14`, `mod.rs:14`, `follow_up.rs:16` se non più referenziati. NON aggiungere alias o re-export di compatibilità — il rename è atomico (coerente con D044 atomic-rename pattern). Skill: `bevy-ecs-expert`.
  - Files: `src/combat/state.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/follow_up.rs`, `src/combat/mod.rs`, `tests/validation_snapshot.rs`
  - Verify: CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | grep -E 'warning: unused|error' | wc -l | xargs -I {} test {} -eq 0 && ! grep -rn 'ActionStage\|action_stage' src/combat/ tests/ && ! grep -q '#\[derive(.*Resource.*)\]' src/combat/state.rs

- [x] **T04: Add tests/pipeline_dispatch.rs lifecycle contract test and verify full integration suite green** `est:1h30m`
  Scrivere il contract test che dimostra R070 (4-fase lifecycle osservabile) e R071 (FIFO follow-up con depth=1), poi gate la slice sull'intera suite verde. CONCRETO: (1) Creare `tests/pipeline_dispatch.rs` modellato su `tests/encounter_e2e.rs` (stesso pattern minimal `App` headless, niente DefaultPlugins, registrazione manuale dei sistemi: `resolve_action_system`, `resolve_follow_up_action_system`, `follow_up_listener_system`, `advance_turn_system`, `check_victory_system`, `apply_status_effects_system` se serve — vedi headless.rs come reference). (2) Test 1 `lifecycle_root_action_emits_4_events_in_order`: costruire encounter Greymon vs Devimon, scrivere `ActionIntent::Basic`, chiamare `app.update()` 1-2 volte, leggere via `Messages::<CombatEvent>::get_cursor().read()`. Asserire che la sequenza degli `kind` contiene `OnActionDeclared { intent_kind: Basic }`, poi `OnActionPreApp`, poi (zero o più core event come `OnDamageDealt`, eventuale `OnBreak`/`OnKO`), poi `OnActionApplied`, poi `OnActionResolved`, in QUELL'ordine posizionale. Tutti con `follow_up_depth == 0`. (3) Test 2 `lifecycle_follow_up_action_emits_second_cycle_with_depth_1`: usare un trigger che esiste nel roster MVP (es. Patamon `OnAllyLowHp` se ancora attivo, altrimenti Tentomon o altro che ha `follow_up` configurato in skills.ron — verificare `assets/data/skills.ron` e `units.ron`). Tick l'app finché il follow-up scatta, asserire che dopo il primo ciclo declared→resolved (depth=0) appare un secondo ciclo declared→resolved (depth=1) sullo stesso bus. (4) Test 3 `lifecycle_emitted_even_when_action_fails_for_sp_shortfall`: forzare un `ActionIntent::Skill` quando l'attaccante ha SP insufficienti; asserire la sequenza `OnActionDeclared` → `OnActionPreApp` → `OnActionFailed { reason }` → `OnActionApplied` → `OnActionResolved` (l'apply path emette comunque Applied/Resolved, l'unica differenza è che core events sono assenti e `OnActionFailed` appare in mezzo). NOTA pre-existing pitfall: `headless_smoke_tick` rotto per `UnitId(5)` mancante (vedi research §Common Pitfalls) — fuori scope, non touchare `headless.rs`. (5) Eseguire `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | tee /tmp/s01-final.log` e verificare 21+ binari verdi; salvare il log finale come evidenza. Skill: `bevy-ecs-expert`.
  - Files: `tests/pipeline_dispatch.rs`
  - Verify: CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | tee /tmp/s01-final.log | grep -E '^test result' | grep -c 'ok\.' | xargs -I {} test {} -ge 21 && CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test pipeline_dispatch 2>&1 | grep -q 'test result: ok'

## Files Likely Touched

- src/combat/events.rs
- tests/event_stream.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- src/combat/follow_up.rs
- src/headless.rs
- src/windowed.rs
- src/combat/state.rs
- src/combat/mod.rs
- tests/validation_snapshot.rs
- tests/pipeline_dispatch.rs
