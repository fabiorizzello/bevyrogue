---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T02: Collapse resolve_action_system and resolve_follow_up_action_system to single-system flow with lifecycle events

Eliminare il punto di rottura della pipeline: collassare `step_declaration` → `step_app` in un unico tick di sistema, ed emettere i 4 lifecycle event attorno alle chiamate. CONCRETO: (1) Refactor `resolve_action_system` (src/combat/turn_system/mod.rs:134-162): rimuovere il parametro `inflight: Option<Res<InFlightAction>>` e il guard `inflight.is_some()`; chiamare `step_declaration` ottenendo un `Option<InFlightAction>` locale (NON `commands.insert_resource`); se `Some(inflight)`, emettere `OnActionDeclared { intent_kind }` + `OnActionPreApp` con `source/target` dell'attaccante e `follow_up_depth = 0`, chiamare `step_app(&mut commands, &inflight, ...)` direttamente, poi emettere `OnActionApplied` + `OnActionResolved`. (2) Mirror identico in `resolve_follow_up_action_system` (src/combat/follow_up.rs:283-327), ma con `follow_up_depth = 1`. (3) Rimuovere `action_pipeline_system` dallo schedule: `src/headless.rs:128` (e import a riga 23) + `src/windowed.rs:108` (e import a riga 13). (4) In `src/combat/turn_system/pipeline.rs` cancellare la funzione `action_pipeline_system` (righe 224-261) e rimuoverne l'export `pub use` da `src/combat/turn_system/mod.rs:401`. (5) `step_declaration` deve restare puro/non-emetter (NON aggiungere `event_writer.write` lì dentro — l'emissione vive nel caller per centralizzare l'ordering). (6) Rimuovere le righe `state.action_stage = ActionStage::PreApp` (pipeline.rs:55) e tutte le `state.action_stage = ActionStage::None` da `step_app` (pipeline.rs:74,106,146,220) — non più rilevanti. NON toccare `InFlightAction` come tipo (resta struct, anche se non più Resource — la definizione viene aggiornata in T03). NON toccare il guard `event.follow_up_depth >= 1` di `follow_up_listener_system` (rimozione in S08). Verificare che `step_app` mantenga la sua firma corrente (è chiamata anche da `resolve_follow_up_action_system`). Quality-gate: dopo T02 i 13 binari falliti devono diventare verdi (validazione parziale OK, T04 fa la verifica completa). Skill: `bevy-ecs-expert`.

## Inputs

- ``src/combat/turn_system/mod.rs` — `resolve_action_system` da refactorare`
- ``src/combat/turn_system/pipeline.rs` — `step_declaration`/`step_app` da preservare, `action_pipeline_system` da eliminare`
- ``src/combat/follow_up.rs` — `resolve_follow_up_action_system` da refactorare`
- ``src/headless.rs` — schedule registration da pulire`
- ``src/windowed.rs` — schedule registration da pulire`
- ``src/combat/events.rs` — 4 varianti lifecycle (output di T01)`

## Expected Output

- ``src/combat/turn_system/mod.rs` — `resolve_action_system` chiama `step_app` inline ed emette 4 lifecycle event`
- ``src/combat/turn_system/pipeline.rs` — `action_pipeline_system` rimosso; `step_declaration`/`step_app` preservati`
- ``src/combat/follow_up.rs` — `resolve_follow_up_action_system` chiama `step_app` inline con depth=1 ed emette 4 lifecycle event`
- ``src/headless.rs` — `action_pipeline_system` rimosso da Update chain e import`
- ``src/windowed.rs` — `action_pipeline_system` rimosso da Update chain e import`

## Verification

CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e --test follow_up_triggers --test sp_economy --test status_effect_apply --test ultimate_meter 2>&1 | grep -E '^test result' | grep -v 'FAILED' | wc -l | xargs -I {} test {} -ge 5 && ! grep -q 'pub fn action_pipeline_system' src/combat/turn_system/pipeline.rs

## Observability Impact

I 4 lifecycle event vivono nel bus `CombatEvent` con `follow_up_depth` corretto (0 root, 1 follow-up); `jsonl_logger_system` li scrive nel log JSONL automaticamente. Failure path: se `step_app` aborta per stun/SP shortfall, `OnActionFailed` resta emesso da `step_app` e `OnActionResolved` viene emesso comunque dal caller — diagnosi completa via lettura sequenziale eventi.
