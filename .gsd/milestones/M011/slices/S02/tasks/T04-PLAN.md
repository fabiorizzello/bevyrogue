---
estimated_steps: 11
estimated_files: 11
skills_used: []
---

# T04: Expose tag_mod_pct + triangle_mod_pct on OnDamageDealt and add Greymon-vs-Devimon JSONL breakdown scenario test

Estende `CombatEventKind::OnDamageDealt` da `{ amount: i32, kind: DamageKind }` a `{ amount: i32, kind: DamageKind, tag_mod_pct: i32, triangle_mod_pct: i32 }`. I valori sono percentuali intere (es. `tag_mod_pct: 125` per weak, `75` per resist, `100` per neutrale; `triangle_mod_pct: 87`/`100`/`111` per win/neutral/lose).

**Refactor `calculate_damage`** in `src/combat/damage.rs` per ritornare una struct `DamageBreakdown { final_damage: i32, tag_mod_pct: i32, triangle_mod_pct: i32 }` invece del solo `i32`. Il call site in `src/combat/resolution.rs` (apply_effects, ~line 217) passa i due percentuali quando costruisce l'evento `OnDamageDealt`.

**Aggiorna tutti gli exhaustive matchers e i test che asseriscono `OnDamageDealt`** (i pattern `OnDamageDealt { amount, kind: dkind }` devono diventare `OnDamageDealt { amount, kind: dkind, .. }` dove i moduli non si interessano dei due nuovi campi). File coinvolti: `tests/event_stream.rs`, `tests/encounter_e2e.rs`, `tests/follow_up_triggers.rs`, `tests/follow_up_reentrancy.rs`, `tests/combat_coherence.rs`, `tests/pipeline_dispatch.rs`, `src/combat/turn_system/pipeline.rs:150`, `src/combat/resolution.rs`.

**Crea `tests/damage_breakdown_log.rs`**: scenario integrato Bevy headless con due unit:
- Attacker: Vaccine attribute, `damage_tag: DamageTag::Fire`, base_damage 100.
- Defender: Virus attribute (Vaccine vince → `triangle_mod_pct=87` per dmg_modifier su difensore vincente — wait, Vaccine vs Virus: Vaccine vince. Quindi defender perde, attacker vince → `triangle_mod_pct=100` per attacker che vince. Verifica MEM022). Per ottenere `triangle_mod_pct=111` serve attacker che PERDE (es. Virus attacker vs Vaccine defender). Devimon è Virus, Greymon è Vaccine — quindi scenario corretto: **Devimon (Virus, attacker) vs Greymon (Vaccine, defender)** ottiene attacker losing → `triangle_mod_pct=111`.
  - Roadmap dice 'Greymon vs Devimon' — interpretiamo come 'lo scenario coinvolge Greymon e Devimon'; l'attacker che produce 111 è Virus. Setup: Devimon attacca Greymon con basic Fire (basic_damage_tag=Fire). Greymon resist Fire? Decidiamo: per il test fixture, Greymon ha `resists: vec![DamageTag::Fire]` → `tag_mod_pct=75`.
  - Atteso evento `OnDamageDealt { amount: round(100×0.75×1.11) = 83, tag_mod_pct: 75, triangle_mod_pct: 111 }`.
- Test asserisce sia `tag_mod_pct=75` sia `triangle_mod_pct=111` sia `amount=83` letti dal `CombatEvent` bus dopo un singolo `app.update()`.
- Optional: aggiunge un secondo scenario simmetrico con un weak match per coprire `tag_mod_pct=125`.

Questo chiude il requisito esplicito del roadmap S02: «log JSONL mostra tag_mod, triangle_mod e final_dmg coerenti».

## Inputs

- ``src/combat/events.rs` — variante OnDamageDealt da estendere`
- ``src/combat/damage.rs` — calculate_damage da T02 (ritorno i32 da rifattorizzare a DamageBreakdown)`
- ``src/combat/resolution.rs` — apply_effects che costruisce OnDamageDealt`
- ``src/combat/turn_system/pipeline.rs` — line 150 destructure di OnDamageDealt`

## Expected Output

- ``src/combat/events.rs` — `OnDamageDealt { amount, kind, tag_mod_pct: i32, triangle_mod_pct: i32 }``
- ``src/combat/damage.rs` — `calculate_damage` ritorna `DamageBreakdown { final_damage, tag_mod_pct, triangle_mod_pct }``
- ``src/combat/resolution.rs` — apply_effects usa il breakdown per costruire l'evento`
- ``src/combat/turn_system/pipeline.rs` — destructure aggiornato (amount, kind, ..)`
- ``tests/damage_breakdown_log.rs` — scenario Devimon(Virus)→Greymon(Vaccine,resistFire) asserting amount=83, tag_mod_pct=75, triangle_mod_pct=111`
- `Tutti gli exhaustive matchers nei test esistenti aggiornati con `..` per ignorare i nuovi field`

## Verification

cargo test --test damage_breakdown_log --no-fail-fast && cargo test --no-fail-fast 2>&1 | tee /tmp/s02-t04.log | grep -qE 'test result: ok\..*0 failed'

## Observability Impact

tag_mod_pct e triangle_mod_pct sono i due campi richiesti esplicitamente dal roadmap S02 per esposizione JSONL: 'log JSONL mostra tag_mod, triangle_mod e final_dmg coerenti'. Con questa estensione `BEVYROGUE_JSONL=1` produrrà per ogni colpo una entry strutturata che permette al rebalance owner (S09) di leggere il breakdown senza re-runnare i calcoli a mano. Auto-derive Serialize su CombatEventKind copre i nuovi campi senza lavoro extra.
