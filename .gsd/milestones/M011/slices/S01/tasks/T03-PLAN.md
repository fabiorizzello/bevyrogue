---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T03: Drop dead InFlightAction Resource, ActionStage variants, and CombatState::action_stage field

Pulire le primitive di stato che T02 ha reso obsolete. CONCRETO: (1) `src/combat/state.rs:53-58`: rimuovere `#[derive(Resource, ...)]` da `InFlightAction` (resta `#[derive(Debug, Clone, PartialEq, Eq)]` se serve, ma NON Resource). (2) `src/combat/state.rs:17-25`: ispezionare quali varianti di `ActionStage` sono ancora referenziate; dopo T02 nessuna lo è (solo `None` e `PreApp` erano vive, e T02 ha rimosso le scritture). Eliminare l'intero enum `ActionStage` se nessun consumatore resta. (3) `src/combat/state.rs:63-67,74,87,96,137`: rimuovere il campo `action_stage: ActionStage` da `CombatState` e tutti i suoi inizializzatori (Default, reset, test fixture). (4) `tests/validation_snapshot.rs:35`: aggiornare la fixture per rimuovere `action_stage` dal letterale di `CombatState`. (5) `src/combat/turn_system/mod.rs:144` aveva `state.action_stage != ActionStage::None` — già rimosso in T02 (parte del guard); verificare clean. `src/combat/follow_up.rs:293` stesso pattern — verificare clean. (6) `src/combat/mod.rs:32` aggiornare il commento doc del re-export se cita `ActionStage`. (7) Rimuovere import non più usati: `ActionStage` da `pipeline.rs:14`, `mod.rs:14`, `follow_up.rs:16` se non più referenziati. NON aggiungere alias o re-export di compatibilità — il rename è atomico (coerente con D044 atomic-rename pattern). Skill: `bevy-ecs-expert`.

## Inputs

- ``src/combat/state.rs` — `InFlightAction`, `ActionStage`, `CombatState::action_stage` da pulire`
- ``src/combat/turn_system/pipeline.rs` — riferimenti a `ActionStage` da rimuovere (output di T02)`
- ``src/combat/turn_system/mod.rs` — import e riferimenti `ActionStage` (output di T02)`
- ``src/combat/follow_up.rs` — import e riferimenti `ActionStage` (output di T02)`
- ``src/combat/mod.rs` — commento doc`
- ``tests/validation_snapshot.rs` — fixture letterale di `CombatState``

## Expected Output

- ``src/combat/state.rs` — `InFlightAction` non Resource; `ActionStage` rimosso; `CombatState` senza `action_stage``
- ``src/combat/turn_system/pipeline.rs` — zero riferimenti a `ActionStage``
- ``src/combat/turn_system/mod.rs` — zero riferimenti a `ActionStage``
- ``src/combat/follow_up.rs` — zero riferimenti a `ActionStage``
- ``src/combat/mod.rs` — doc comment aggiornato`
- ``tests/validation_snapshot.rs` — fixture senza campo `action_stage``

## Verification

CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | grep -E 'warning: unused|error' | wc -l | xargs -I {} test {} -eq 0 && ! grep -rn 'ActionStage\|action_stage' src/combat/ tests/ && ! grep -q '#\[derive(.*Resource.*)\]' src/combat/state.rs
