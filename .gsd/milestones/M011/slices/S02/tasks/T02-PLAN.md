---
estimated_steps: 10
estimated_files: 11
skills_used: []
---

# T02: Replace Resistances([i8;6]) with Vec<DamageTag> resists + rewrite calculate_damage on v5.3 multiplicative model

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

## Inputs

- ``src/combat/types.rs` — Resistances struct da rimuovere`
- ``src/combat/unit.rs` — Unit.resistances field da sostituire`
- ``src/combat/damage.rs` — calculate_damage e tipo_table da riscrivere`
- ``src/combat/damage_tests.rs` — matrice 24-cell additiva da rifare in 18-cell moltiplicativa`
- ``src/combat/toughness.rs` — classify signature con target_resists: &Resistances`
- ``src/data/units_ron.rs` — UnitDef.resistances field`
- ``assets/data/units.ron` — `resistances: Resistances((0,...))` rows da rimuovere`

## Expected Output

- ``src/combat/types.rs` — Resistances struct rimosso; solo DamageTag + altri tipi`
- ``src/combat/unit.rs` — `pub resists: Vec<DamageTag>` al posto di resistances`
- ``src/combat/damage.rs` — `calculate_damage` riscritto multiplicativo + helper `triangle_modifiers(att,def) -> TriangleMods`; clamp rimosso`
- ``src/combat/damage_tests.rs` — nuova matrice 18 test + edge case con valori v5.3`
- ``src/combat/toughness.rs` — `classify(tag, weaknesses, resists, is_break)` signature pulita`
- ``src/data/units_ron.rs` — `resists: Vec<DamageTag>` su UnitDef`
- ``assets/data/units.ron` — ogni unit ha `resists: []` (MVP roster a zero)`
- ``tests/triangle_matchup.rs` — nuovo test che enumera 16 coppie e valida il triple di TriangleMods`

## Verification

cargo test --test damage_tests --no-fail-fast && cargo test --test triangle_matchup --no-fail-fast && cargo test --no-fail-fast 2>&1 | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'Resistances\|resistances' src/ tests/ assets/data/

## Observability Impact

La formula damage cambia output osservabile per ogni attacco. Test damage_breakdown_log (T04) verificherà i nuovi valori; combat_coherence/encounter_e2e potrebbero richiedere update di valori HP attesi nei loro scenari. Documenta la convenzione `dmg_modifier` (single number) in un commento sopra `triangle_modifiers` per evitare la confusione MEM022 framing dmg_in vs dmg_out.
