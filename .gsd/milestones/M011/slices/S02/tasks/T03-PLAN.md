---
estimated_steps: 10
estimated_files: 10
skills_used: []
---

# T03: Add CombatRng resource + OnStatusResisted event + status_accuracy roll + retrofit Shock RNG

Introduce determinismo (R019) sull'asse RNG. Crea risorsa `CombatRng(SmallRng)` in `src/combat/rng.rs` (nuovo modulo, registrato in `mod.rs`) seedata dal bootstrap. Espone helper `roll_pct(threshold: i32) -> bool` che ritorna true se il roll passa.

**Estendi `CombatEventKind` con `OnStatusResisted { kind: StatusEffectKind }`** in `events.rs`. Aggiorna l'exhaustive matcher in `tests/event_stream.rs` con un arm esplicito. I tre test-local matcher (`follow_up_reentrancy`, `follow_up_triggers`, `combat_coherence`) hanno wildcard `_ => "Other"` e non richiedono modifica.

**Cabla l'accuracy roll** in `src/combat/turn_system/pipeline.rs:192` (sezione status_to_apply). Calcola `status_acc = triangle_modifiers(attacker.attribute, defender.attribute).status_acc_modifier` (default 1.0; 0.90 se attacker perde il triangle). Roll: `if rng.roll_pct((status_acc * 100.0) as i32) { apply status; emit OnStatusApplied }` else `{ emit OnStatusResisted; do not insert StatusEffect }`. Posiziona l'evento tra `OnActionPreApp` e `OnActionApplied` per preservare il lifecycle contract S01.

**Retrofit `src/combat/turn_system/mod.rs:267`** — sostituisce `rand::thread_rng().gen_range(0..100)` con `combat_rng.roll_pct(*cancel_chance_pct as i32)` per chiudere la pre-existing tech debt R019.

**Bootstrap seeding**: aggiunge `CombatRng::from_seed(seed)` in `bootstrap.rs` con seed di default `[42u8; 32]` per i test e configurabile via `BootstrapConfig` (se esiste già un seed pattern, riusalo; altrimenti aggiungi `bootstrap_rng_seed: u64`).

**Aggiungi `tests/status_accuracy.rs`** con almeno 3 scenari:
1. Vaccine attacker → Data defender (attacker perde, status_acc=0.90): seed scelto in modo che il roll vada `>=90` → `OnStatusResisted` emesso, no `OnStatusApplied`, `StatusEffect` component non inserito.
2. Stesso matchup, seed scelto in modo che roll vada `<90` → `OnStatusApplied` emesso, status applicato.
3. Vaccine attacker → Vaccine defender (neutrale, status_acc=1.0): qualunque seed → status applicato (R076: solo l'attaccante perdente subisce penalità).

Verifica che `tests/pipeline_dispatch.rs` continui a passare — i test esistenti che applicano status devono usare matchup neutro o essere aggiornati per registrare `CombatRng` con seed che fa passare il roll.

## Inputs

- ``src/combat/events.rs` — CombatEventKind enum da estendere con OnStatusResisted`
- ``src/combat/turn_system/pipeline.rs` — sezione status_to_apply (line ~192)`
- ``src/combat/turn_system/mod.rs` — Shock cancel-roll a line 267 (R019 violation)`
- ``src/combat/bootstrap.rs` — punto di seed per CombatRng`
- ``src/combat/damage.rs` — triangle_modifiers helper (da T02) per leggere status_acc_modifier`
- ``tests/event_stream.rs` — exhaustive matcher`
- ``tests/pipeline_dispatch.rs` — lifecycle test, deve continuare a passare`

## Expected Output

- ``src/combat/rng.rs` — nuovo modulo `CombatRng(SmallRng)` con `from_seed(u64)` e `roll_pct(threshold: i32) -> bool``
- ``src/combat/events.rs` — variante `OnStatusResisted { kind: StatusEffectKind }``
- ``src/combat/turn_system/pipeline.rs` — accuracy roll cablato; emette OnStatusResisted o OnStatusApplied condizionalmente, lifecycle contract preservato`
- ``src/combat/turn_system/mod.rs` — Shock roll usa CombatRng, thread_rng rimosso`
- ``src/combat/bootstrap.rs` — CombatRng inizializzata con seed deterministico`
- ``src/combat/mod.rs` — `pub mod rng` esportato`
- ``src/headless.rs` — CombatRng resource registrata nel plugin`
- ``tests/event_stream.rs` — match arm `OnStatusResisted { .. } => "OnStatusResisted"``
- ``tests/status_accuracy.rs` — 3+ test deterministici (miss, hit, neutrale)`

## Verification

cargo test --test status_accuracy --no-fail-fast && cargo test --test pipeline_dispatch --no-fail-fast && cargo test --no-fail-fast 2>&1 | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'thread_rng' src/combat/

## Observability Impact

OnStatusResisted rende osservabile il caso 'status non applicato per accuracy miss', che prima sarebbe invisibile (status semplicemente non presente). CombatRng centralizza tutto l'accesso randomico in una resource ispezionabile/seedabile — sblocca debug riproducibile e tests deterministici (R019). Lifecycle contract S01: `OnActionPreApp` → core events (OnDamageDealt, OnBreak, OnStatusApplied|OnStatusResisted) → `OnActionApplied` → `OnActionResolved`.
