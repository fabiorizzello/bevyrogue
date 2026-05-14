# M019: M019: DR pipeline + Heal/Cleanse primitives + PerHop guard

**Vision:** Estendere il kernel combat con tre primitive generiche minimali — damage reduction (BuffKind::DR), heal (% maxHP), cleanse (rimozione N debuff) — senza introdurre regole skill-specifiche nel kernel (P001). Chiudere il debt M018 sul `DamageCurve::PerHop` runtime length guard. Tutte le specificità per-skill (scaling ATK, cleanse selettivo per kind, immunità custom) restano fuori dal kernel e sono deferite a M021 (`trait Skill` + `SkillCtx`).

## Success Criteria

- BuffKind::DR integrato in calculate_damage come step moltiplicativo con ARM/Break, senza cap (somma libera, può portare damage a 0)
- Effect::Heal { amount_pct_max_hp } applicato in resolution.rs, cap a maxHP, CombatEvent::Healed emesso, skip su unità KO
- Effect::Cleanse { count: Option<u8> } rimuove N debuff dalla StatusBag rispettando il flag immune già presente sulla StatusEntry; nessuna lista hardcoded di status immuni nel kernel
- DamageCurve::PerHop guard runtime: se coeffs.len() < hops_planned, fail-fast con event diagnostico o clamp (decisione di slice)
- Kernel resta franchise-agnostic: nessun nome Digimon, nessun if su skill_id, nessuna regola skill-specifica introdotta in src/combat/

## Slices

- [x] **S01: S01** `risk:medium` `depends:[]`
  > After this: Test integration tests/dr_pipeline.rs dimostra DR singolo, DR×N sommato, DR+ARM combinato, DR durante Break — damage clampato a 0 senza panic, CombatEvent::Damage emesso con amount=0 dove applicabile.

- [x] **S02: S02** `risk:low` `depends:[]`
  > After this: Test integration tests/heal_effect.rs: skill RON con Effect::Heal applicata su Single e AllAllies, cap a maxHP, no-op su KO, CombatEvent::Healed nel JSONL stream.

- [x] **S03: S03** `risk:low` `depends:[]`
  > After this: Test integration tests/cleanse_effect.rs: cleanse count=2 rimuove 2 debuff non-immuni; Blessed (immune) non rimosso; count=None svuota tutti i debuff non-immuni; CombatEvent::Cleansed nel JSONL.

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: Test tests/perhop_guard.rs: skill con DamageCurve::PerHop di lunghezza < hops_planned produce evento diagnostico (fail-fast o clamp — decisione registrata in DECISIONS.md) senza panic.

## Boundary Map

## Boundary Map

Not provided.
