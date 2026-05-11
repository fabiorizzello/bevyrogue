---
estimated_steps: 6
estimated_files: 41
skills_used: []
---

# T01: Atomic rename Element→DamageTag across types, fields, RON, and full call/test sweep

Esegue il rename atomico richiesto da D044 senza alias e senza fallback. Cambia il tipo `enum Element { Fire, Water, Plant, Electro, Light, Dark }` in `enum DamageTag { Physical, Fire, Ice, Electric, Light, Dark }` (nuovo ordine variants → nuovo index ordinality). Rinomina i field: `UnitDef.basic_element`→`basic_damage_tag`, `SkillDef.element`→`damage_tag`, `ResolvedAction.element`→`damage_tag`, `AttackContext.element`→`damage_tag`, `CombatEventKind::OnBreak { element }`→`{ damage_tag }`, `LogEntry::Break.element`→`damage_tag`, `ValidationLogEntry::Break.element`→`damage_tag`, `Toughness.weaknesses: Vec<Element>`→`Vec<DamageTag>`. Aggiorna `Toughness::apply_hit` e `classify` signature.

**Variant remap:** `Water→Ice`, `Electro→Electric`, `Plant→Physical`. **Index remap nei test che accedono `resistances.0[N]` per ordinale:** old Fire=0 diventa Fire=1; old Dark=5 resta Dark=5 (perché ora c'è Physical=0). Aggiorna ogni `resistances.0[OLD_IDX]=N` ai nuovi indici secondo nuovo discriminant order.

**Scope intentionally large** (~30 file): è un rename meccanico testo-su-testo, non logica. Preserva temporaneamente `Resistances([i8;6])` struct e la formula additiva esistente in `calculate_damage` — sarà T02 a sostituirle. Dopo T01 la suite deve essere ancora verde.

File RON da aggiornare: `assets/data/units.ron` (key `basic_element:`→`basic_damage_tag:`, valori `Water→Ice` `Electro→Electric`, `weaknesses:` valori rimappati), `assets/data/skills.ron` (key `element:`→`damage_tag:`, valori rimappati). Test `tests/bootstrap_spawn_composition.rs:107` Plant→Physical (Hackmon è placeholder fixture).

**Update exhaustive matchers:** `tests/event_stream.rs` strict matcher e i tre test-local matchers (`follow_up_reentrancy.rs`, `follow_up_triggers.rs`, `combat_coherence.rs`). `OnBreak` ha solo cambio nome field — tutti i pattern `OnBreak { element }` diventano `OnBreak { damage_tag }`.

**Snapshot fixture:** `tests/validation_snapshot.rs:124` ha stringa hardcoded `weaknesses=[Water]` — aggiorna a `weaknesses=[Ice]` (o equivalente per il fixture). Allinea `format_weaknesses` in `observability.rs` se il formato impone case/variant.

## Inputs

- ``src/combat/types.rs` — enum to rename + variant set change`
- ``src/combat/toughness.rs` — Toughness.weaknesses Vec<Element> field`
- ``src/combat/events.rs` — CombatEventKind::OnBreak { element } variant`
- ``src/combat/damage_tests.rs` — 24-cell matrix using Element variants and resistances index`
- ``src/data/units_ron.rs` — UnitDef.basic_element field + tests fixtures`
- ``src/data/skills_ron.rs` — SkillDef.element field + tests fixtures`
- ``assets/data/units.ron` — basic_element/weaknesses/resistances per unit`
- ``assets/data/skills.ron` — element key per skill`

## Expected Output

- ``src/combat/types.rs` — `enum DamageTag { Physical, Fire, Ice, Electric, Light, Dark }`; `Resistances([i8;6])` ancora presente (T02 lo rimuove)`
- ``src/combat/toughness.rs` — `weaknesses: Vec<DamageTag>`, `apply_hit` e `classify` con signature aggiornata`
- ``src/combat/events.rs` — `CombatEventKind::OnBreak { damage_tag: DamageTag }``
- ``src/combat/damage.rs` — `AttackContext.damage_tag`; formula additiva ancora intatta`
- ``src/combat/damage_tests.rs` — tutti i test rinominati e con indici aggiornati al nuovo discriminant order; suite verde`
- ``src/data/units_ron.rs` — `basic_damage_tag: DamageTag` + tests fixture aggiornati`
- ``src/data/skills_ron.rs` — `damage_tag: DamageTag` + tests fixture aggiornati`
- ``assets/data/units.ron` — chiavi `basic_damage_tag:` e valori rimappati Water→Ice/Electro→Electric`
- ``assets/data/skills.ron` — chiavi `damage_tag:` e valori rimappati`
- ``tests/event_stream.rs` — exhaustive matcher con `OnBreak { damage_tag }` esplicito`
- ``tests/validation_snapshot.rs` — fixture `weaknesses=[Ice]` aggiornata`
- ``tests/bootstrap_spawn_composition.rs` — Hackmon basic_damage_tag: Physical (Plant rimosso)`

## Verification

cargo test --no-fail-fast 2>&1 | tee /tmp/s02-t01.log | grep -qE 'test result: ok\..*0 failed' && ! grep -rn 'Element::\|: Element\|basic_element' src/ tests/ assets/data/

## Observability Impact

Nessun cambio runtime — solo rename meccanico. Il `OnBreak` event continua a essere emesso con stesso payload semantico, solo nome del field cambia.
