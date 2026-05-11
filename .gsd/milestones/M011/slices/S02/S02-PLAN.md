# S02: Damage Tag rename + matchup ±25% + Attribute Triangle v5.3 in-line

**Goal:** Allineare il damage system a v5.3: rinominare atomicamente Element→DamageTag (variant set Physical/Fire/Ice/Electric/Light/Dark), sostituire `Resistances([i8;6])` con `resists: Vec<DamageTag>` per Unit/UnitDef, riscrivere `calculate_damage` con il modello moltiplicativo (tag_mod ×1.25/×0.75/×1.0 e triangle_mods ×0.87/×1.11/×0.90), introdurre `CombatRng` deterministico con accuracy roll che emette `OnStatusResisted`, ed esporre `tag_mod_pct`/`triangle_mod_pct` su `OnDamageDealt` per ispezione JSONL.
**Demo:** scenario test Greymon vs Devimon: log JSONL mostra tag_mod, triangle_mod e final_dmg coerenti; cargo run via combat_cli (post-S04) replica i numeri attesi

## Must-Haves

- Suite intera (`cargo test --no-fail-fast`) verde (≥27 binari, S01 baseline 24 + 3 nuovi).
- `damage_tests.rs` riscritto sul nuovo modello multiplicativo, valori coerenti con MEM022.
- `tests/triangle_matchup.rs` valida tutte le 16 coppie (attacker, defender) attribute.
- `tests/status_accuracy.rs` dimostra seed→miss deterministico con `OnStatusResisted` emesso e `OnStatusApplied` non emesso.
- `tests/damage_breakdown_log.rs` esercita scenario Greymon (Vaccine, Fire) vs Devimon-like (Virus, weak Light, resist Fire) e asserisce `tag_mod_pct=75` (resist) e `triangle_mod_pct=111` su `OnDamageDealt`.
- `tests/pipeline_dispatch.rs` continua a passare; `OnStatusResisted` posizionato tra `OnActionPreApp` e `OnActionApplied` (lifecycle contract S01 preservato).
- `! grep -rn "Element::\|basic_element\|Resistances\|: Element" src/ tests/ assets/data/` ritorna zero match (rename atomico completo).
- `rand::thread_rng()` rimosso da `src/combat/turn_system/mod.rs:267` (Shock retrofit su `CombatRng`, R019 honored).
- R075 e R076 advanced ad active→validated.

## Proof Level

- This slice proves: - This slice proves: integration (formula + lifecycle event + JSONL exposure)
- Real runtime required: yes (Bevy app integration tests)
- Human/UAT required: no (UAT scenarios scheduled in S09)

## Integration Closure

- Upstream surfaces consumed: action pipeline lifecycle (S01) — `OnActionDeclared/PreApp/Applied/Resolved` brackets restano intatti; `OnStatusResisted` si inserisce come "core event" tra `PreApp` e `Applied` come gli altri (`OnDamageDealt`, `OnBreak`).
- New wiring introduced: `CombatRng` Resource registrato in `headless.rs` (e via `windowed.rs` per parità) seedato dal bootstrap; status-accuracy roll cablato in `pipeline.rs:192`.
- What remains before milestone is usable end-to-end: S04 combat_cli per playtest manuale, S05-S08 per le altre primitive di v5.3 (caps, Tempo Resistance, Toughness 3 categorie, Form Identity), S09 per il numerical rebalance finale.

## Verification

- Runtime signals: nuovo `CombatEventKind::OnStatusResisted { kind }` emesso quando l'accuracy roll fallisce; campi `tag_mod_pct: i32` e `triangle_mod_pct: i32` aggiunti a `OnDamageDealt` per esposizione JSONL.
- Inspection surfaces: `BEVYROGUE_JSONL=1` log entry con breakdown completo per ogni colpo (visibile via `tests/damage_breakdown_log.rs` e via S04 combat_cli quando atterra).
- Failure visibility: `OnStatusResisted` rende osservabile il caso "status missato per triangle accuracy" che altrimenti sarebbe silenzioso (status semplicemente non applicato).
- Redaction constraints: nessun PII/secret toccato.

## Tasks

- [x] **T01: Atomic rename Element→DamageTag across types, fields, RON, and full call/test sweep** `est:2h`
  Esegue il rename atomico richiesto da D044 senza alias e senza fallback. Cambia il tipo `enum Element { Fire, Water, Plant, Electro, Light, Dark }` in `enum DamageTag { Physical, Fire, Ice, Electric, Light, Dark }` (nuovo ordine variants → nuovo index ordinality). Rinomina i field: `UnitDef.basic_element`→`basic_damage_tag`, `SkillDef.element`→`damage_tag`, `ResolvedAction.element`→`damage_tag`, `AttackContext.element`→`damage_tag`, `CombatEventKind::OnBreak { element }`→`{ damage_tag }`, `LogEntry::Break.element`→`damage_tag`, `ValidationLogEntry::Break.element`→`damage_tag`, `Toughness.weaknesses: Vec<Element>`→`Vec<DamageTag>`. Aggiorna `Toughness::apply_hit` e `classify` signature.

**Variant remap:** `Water→Ice`, `Electro→Electric`, `Plant→Physical`. **Index remap nei test che accedono `resistances.0[N]` per ordinale:** old Fire=0 diventa Fire=1; old Dark=5 resta Dark=5 (perché ora c'è Physical=0). Aggiorna ogni `resistances.0[OLD_IDX]=N` ai nuovi indici secondo nuovo discriminant order.

**Scope intentionally large** (~30 file): è un rename meccanico testo-su-testo, non logica. Preserva temporaneamente `Resistances([i8;6])` struct e la formula additiva esistente in `calculate_damage` — sarà T02 a sostituirle. Dopo T01 la suite deve essere ancora verde.

File RON da aggiornare: `assets/data/units.ron` (key `basic_element:`→`basic_damage_tag:`, valori `Water→Ice` `Electro→Electric`, `weaknesses:` valori rimappati), `assets/data/skills.ron` (key `element:`→`damage_tag:`, valori rimappati). Test `tests/bootstrap_spawn_composition.rs:107` Plant→Physical (Hackmon è placeholder fixture).

**Update exhaustive matchers:** `tests/event_stream.rs` strict matcher e i tre test-local matchers (`follow_up_reentrancy.rs`, `follow_up_triggers.rs`, `combat_coherence.rs`). `OnBreak` ha solo cambio nome field — tutti i pattern `OnBreak { element }` diventano `OnBreak { damage_tag }`.

**Snapshot fixture:** `tests/validation_snapshot.rs:124` ha stringa hardcoded `weaknesses=[Water]` — aggiorna a `weaknesses=[Ice]` (o equivalente per il fixture). Allinea `format_weaknesses` in `observability.rs` se il formato impone case/variant.
  - Files: `src/combat/types.rs`, `src/combat/unit.rs`, `src/combat/toughness.rs`, `src/combat/state.rs`, `src/combat/events.rs`, `src/combat/log.rs`, `src/combat/observability.rs`, `src/combat/resolution.rs`, `src/combat/damage.rs`, `src/combat/damage_tests.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/tests.rs`, `src/combat/resolution_tests.rs`, `src/combat/follow_up_tests.rs`, `src/combat/bootstrap.rs`, `src/combat/mod.rs`, `src/data/units_ron.rs`, `src/data/skills_ron.rs`, `src/headless.rs`, `src/ui/combat_panel.rs`, `assets/data/units.ron`, `assets/data/skills.ron`, `tests/bootstrap_spawn_composition.rs`, `tests/boundary_contract.rs`, `tests/combat_coherence.rs`, `tests/encounter_e2e.rs`, `tests/event_stream.rs`, `tests/follow_up_reentrancy.rs`, `tests/follow_up_triggers.rs`, `tests/patamon_revive.rs`, `tests/pipeline_dispatch.rs`, `tests/revive_semantics.rs`, `tests/sp_economy.rs`, `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/ultimate_meter.rs`, `tests/validation_snapshot.rs`, `tests/commander_flow.rs`, `tests/enemy_ai.rs`, `tests/roster_smoke.rs`
  - Verify: cargo test --no-fail-fast 2>&1 | tee /tmp/s02-t01.log | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'Element::\|: Element\|basic_element' src/ tests/ assets/data/

- [x] **T02: Replace Resistances([i8;6]) with Vec<DamageTag> resists + rewrite calculate_damage on v5.3 multiplicative model** `est:1h30m`
  Rimuove `Resistances([i8;6])` struct dal modulo `types.rs`. Aggiunge `resists: Vec<DamageTag>` come field su `Unit` (componente Bevy) e su `UnitDef` (RON schema). Aggiorna fixture `assets/data/units.ron` (rimuove riga `resistances: Resistances((0,...,0))` da ogni unit, aggiunge `resists: []` — il roster MVP corrente ha tutte resistenze a zero).

**Riscrivi `calculate_damage`** secondo MEM022 / D043:
- `tag_mod = 1.25` se `defender.toughness.weaknesses.contains(&attack.damage_tag)` else `0.75` se `defender.unit.resists.contains(&attack.damage_tag)` else `1.0`.
- `triangle_modifiers(att_attr, def_attr) → TriangleMods { dmg_modifier, tough_modifier, status_acc_modifier }`. Convenzione documentata in commento sopra `triangle_modifiers`: il valore è applicato al danno OUTGOING (single number per attacco). Defender vince → `dmg_modifier = 0.87`. Attacker perde → `dmg_modifier = 1.11` (asimmetria voluta, vedi MEM022). Stesso schema vince/perde → `1.0`.
- Ciclo Vaccine > Virus > Data > Vaccine; Free neutrale a tutti (`1.0`).
- Final: `damage = round(base × tag_mod × triangle_mod × (2.0 if is_break else 1.0))`.
- **Drop** `clamp(0.25, 2.5)` — il modello moltiplicativo è naturalmente bounded.

**Re-write `damage_tests.rs`** (riscrittura completa): nuova matrice 3 tag-bucket × 3 triangle-bucket × 2 break = 18 test, più 4 edge case (Free neutralità sym, Physical neutralità tag, base=0, tag=resist+triangle=lose). Calcoli attesi documentati nei commenti (es. `tag=weak,triangle=win,no_break: 100×1.25×0.87 = 108.75 → round=109`).

**Aggiungi `tests/triangle_matchup.rs`** — test parametrico che enumera tutte le 16 (attacker_attr, defender_attr) e asserisce il triple `(dmg_modifier, tough_modifier, status_acc_modifier)` atteso da `triangle_modifiers`. Usa `Attribute::*` const list.

**Aggiorna `Toughness::apply_hit` e `classify`**: la signature `target_resists: &Resistances` viene rimossa. `classify` ora prende `weaknesses: &[DamageTag]` e `resists: &[DamageTag]`. Aggiorna call site in `resolution.rs:217-240` e `pipeline.rs:154` per passare `defender.unit.resists.as_slice()`.
  - Files: `src/combat/types.rs`, `src/combat/unit.rs`, `src/combat/toughness.rs`, `src/combat/damage.rs`, `src/combat/damage_tests.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/bootstrap.rs`, `src/data/units_ron.rs`, `assets/data/units.ron`, `tests/triangle_matchup.rs`
  - Verify: cargo test --test damage_tests --no-fail-fast && cargo test --test triangle_matchup --no-fail-fast && cargo test --no-fail-fast 2>&1 | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'Resistances\|resistances' src/ tests/ assets/data/

- [x] **T03: Add CombatRng resource + OnStatusResisted event + status_accuracy roll + retrofit Shock RNG** `est:1h30m`
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
  - Files: `src/combat/events.rs`, `src/combat/rng.rs`, `src/combat/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/bootstrap.rs`, `src/headless.rs`, `tests/event_stream.rs`, `tests/pipeline_dispatch.rs`, `tests/status_accuracy.rs`
  - Verify: cargo test --test status_accuracy --no-fail-fast && cargo test --test pipeline_dispatch --no-fail-fast && cargo test --no-fail-fast 2>&1 | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'thread_rng' src/combat/

- [x] **T04: Expose tag_mod_pct + triangle_mod_pct on OnDamageDealt and add Greymon-vs-Devimon JSONL breakdown scenario test** `est:1h`
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
  - Files: `src/combat/events.rs`, `src/combat/damage.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `tests/event_stream.rs`, `tests/encounter_e2e.rs`, `tests/combat_coherence.rs`, `tests/follow_up_triggers.rs`, `tests/follow_up_reentrancy.rs`, `tests/pipeline_dispatch.rs`, `tests/damage_breakdown_log.rs`
  - Verify: cargo test --test damage_breakdown_log --no-fail-fast && cargo test --no-fail-fast 2>&1 | tee /tmp/s02-t04.log | grep -qE 'test result: ok\..*0 failed'

## Files Likely Touched

- src/combat/types.rs
- src/combat/unit.rs
- src/combat/toughness.rs
- src/combat/state.rs
- src/combat/events.rs
- src/combat/log.rs
- src/combat/observability.rs
- src/combat/resolution.rs
- src/combat/damage.rs
- src/combat/damage_tests.rs
- src/combat/turn_system/pipeline.rs
- src/combat/turn_system/tests.rs
- src/combat/resolution_tests.rs
- src/combat/follow_up_tests.rs
- src/combat/bootstrap.rs
- src/combat/mod.rs
- src/data/units_ron.rs
- src/data/skills_ron.rs
- src/headless.rs
- src/ui/combat_panel.rs
- assets/data/units.ron
- assets/data/skills.ron
- tests/bootstrap_spawn_composition.rs
- tests/boundary_contract.rs
- tests/combat_coherence.rs
- tests/encounter_e2e.rs
- tests/event_stream.rs
- tests/follow_up_reentrancy.rs
- tests/follow_up_triggers.rs
- tests/patamon_revive.rs
- tests/pipeline_dispatch.rs
- tests/revive_semantics.rs
- tests/sp_economy.rs
- tests/status_effect_apply.rs
- tests/status_effect_integration.rs
- tests/status_effect_turn_tick.rs
- tests/ultimate_meter.rs
- tests/validation_snapshot.rs
- tests/commander_flow.rs
- tests/enemy_ai.rs
- tests/roster_smoke.rs
- tests/triangle_matchup.rs
- src/combat/rng.rs
- src/combat/turn_system/mod.rs
- tests/status_accuracy.rs
- tests/damage_breakdown_log.rs
