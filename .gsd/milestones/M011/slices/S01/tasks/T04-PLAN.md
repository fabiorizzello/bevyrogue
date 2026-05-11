---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Add tests/pipeline_dispatch.rs lifecycle contract test and verify full integration suite green

Scrivere il contract test che dimostra R070 (4-fase lifecycle osservabile) e R071 (FIFO follow-up con depth=1), poi gate la slice sull'intera suite verde. CONCRETO: (1) Creare `tests/pipeline_dispatch.rs` modellato su `tests/encounter_e2e.rs` (stesso pattern minimal `App` headless, niente DefaultPlugins, registrazione manuale dei sistemi: `resolve_action_system`, `resolve_follow_up_action_system`, `follow_up_listener_system`, `advance_turn_system`, `check_victory_system`, `apply_status_effects_system` se serve — vedi headless.rs come reference). (2) Test 1 `lifecycle_root_action_emits_4_events_in_order`: costruire encounter Greymon vs Devimon, scrivere `ActionIntent::Basic`, chiamare `app.update()` 1-2 volte, leggere via `Messages::<CombatEvent>::get_cursor().read()`. Asserire che la sequenza degli `kind` contiene `OnActionDeclared { intent_kind: Basic }`, poi `OnActionPreApp`, poi (zero o più core event come `OnDamageDealt`, eventuale `OnBreak`/`OnKO`), poi `OnActionApplied`, poi `OnActionResolved`, in QUELL'ordine posizionale. Tutti con `follow_up_depth == 0`. (3) Test 2 `lifecycle_follow_up_action_emits_second_cycle_with_depth_1`: usare un trigger che esiste nel roster MVP (es. Patamon `OnAllyLowHp` se ancora attivo, altrimenti Tentomon o altro che ha `follow_up` configurato in skills.ron — verificare `assets/data/skills.ron` e `units.ron`). Tick l'app finché il follow-up scatta, asserire che dopo il primo ciclo declared→resolved (depth=0) appare un secondo ciclo declared→resolved (depth=1) sullo stesso bus. (4) Test 3 `lifecycle_emitted_even_when_action_fails_for_sp_shortfall`: forzare un `ActionIntent::Skill` quando l'attaccante ha SP insufficienti; asserire la sequenza `OnActionDeclared` → `OnActionPreApp` → `OnActionFailed { reason }` → `OnActionApplied` → `OnActionResolved` (l'apply path emette comunque Applied/Resolved, l'unica differenza è che core events sono assenti e `OnActionFailed` appare in mezzo). NOTA pre-existing pitfall: `headless_smoke_tick` rotto per `UnitId(5)` mancante (vedi research §Common Pitfalls) — fuori scope, non touchare `headless.rs`. (5) Eseguire `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | tee /tmp/s01-final.log` e verificare 21+ binari verdi; salvare il log finale come evidenza. Skill: `bevy-ecs-expert`.

## Inputs

- ``tests/pipeline_dispatch.rs` — nuovo file da creare`
- ``tests/encounter_e2e.rs` — template per minimal Bevy app headless`
- ``src/combat/events.rs` — varianti lifecycle (output di T01)`
- ``src/combat/turn_system/mod.rs` — `resolve_action_system` aggiornato (output di T02)`
- ``src/combat/follow_up.rs` — `resolve_follow_up_action_system` aggiornato (output di T02)`
- ``src/headless.rs` — reference per system registration order`
- ``assets/data/skills.ron` — config follow-up per scegliere fixture`
- ``assets/data/units.ron` — roster fixture`

## Expected Output

- ``tests/pipeline_dispatch.rs` — 3 test (root, follow-up depth=1, sp-shortfall) verdi`
- ``/tmp/s01-final.log` — evidenza che `cargo test --tests --no-fail-fast` riporta 21+ ok / 0 failed`

## Verification

CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | tee /tmp/s01-final.log | grep -E '^test result' | grep -c 'ok\.' | xargs -I {} test {} -ge 21 && CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test pipeline_dispatch 2>&1 | grep -q 'test result: ok'

## Observability Impact

Il test stesso è la inspection surface canonica per il contratto lifecycle. Pattern di lettura tramite `Messages::<CombatEvent>::get_cursor().read()` documenta come un futuro debugger può estrarre la sequenza eventi dal bus. Caso negativo SP-shortfall pinned: `OnActionResolved` deve essere emesso anche dopo `OnActionFailed`, garantendo che ogni azione (riuscita o fallita) chiuda il ciclo lifecycle (R070 robustezza).
