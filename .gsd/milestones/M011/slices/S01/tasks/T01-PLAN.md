---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add 4 lifecycle CombatEventKind variants and update exhaustive matchers

Estendere `CombatEventKind` (src/combat/events.rs) con 4 nuove varianti dedicate al lifecycle dell'azione: `OnActionDeclared { intent_kind: ActionIntentKind }`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`. Definire un nuovo enum `ActionIntentKind { Basic, Skill, Ultimate }` (serializable, `Debug + Clone + PartialEq + Eq + serde::Serialize`) co-locato in `events.rs` (NON riusare `ActionIntent` che porta payload completi). Aggiornare il matcher esaustivo `assert!(kinds.iter().all(|kind| matches!(kind, ...)))` in `tests/event_stream.rs:200-210` per includere le 4 nuove varianti, in modo che il test continui a compilare e passare quando T02 inizia a emettere gli eventi (anche se `event_stream.rs` non li asserisce esplicitamente come 'presenti', solo come 'consentiti'). Aggiornare anche il commento DOC su D026 nel campo `follow_up_depth` di `CombatEvent` se serve chiarezza, ma NON rimuovere il guard (rimozione in S08 con D046). Verificare che `jsonl_logger.rs` e `log.rs` non abbiano `match` esaustivi sull'enum (oggi usano `_ => {}` o pattern parziali); se ne hanno, aggiungere un arm `_ => {}` di default per le nuove varianti. Skill suggerito per l'executor: `bevy-ecs-expert` (installato globalmente).

## Inputs

- ``src/combat/events.rs` — enum esistente da estendere`
- ``tests/event_stream.rs` — matcher esaustivo allowed-set da espandere`
- ``src/combat/jsonl_logger.rs` — verificare se ha match esaustivo`
- ``src/combat/log.rs` — verificare se ha match esaustivo`

## Expected Output

- ``src/combat/events.rs` — 4 varianti lifecycle aggiunte + enum `ActionIntentKind``
- ``tests/event_stream.rs` — allowed-set match esteso con le 4 varianti`

## Verification

CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | tee /tmp/t01-check.log && grep -q 'OnActionDeclared' src/combat/events.rs && grep -q 'OnActionResolved' tests/event_stream.rs
