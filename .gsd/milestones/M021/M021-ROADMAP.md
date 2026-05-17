# M021: M021: Kernel framework + Timeline FSM + Registry<E>

**Vision:** Un kernel combat che espone solo primitive generiche (`Intent` come unica mutazione, `CompiledTimeline` come unica forma di skill, `Registry<E: ExtPoint>` come unico asse di estensione, `SkillCtx` come unico contesto, `SignalBus` come bus reattivo, `Clock` two-mode). Niente trait per skill, niente enum effect, niente trait per blueprint. Ogni Digimon = un solo modulo + un solo `register(reg: &mut ExtRegistries)`. Lo skilltree è context immutabile per il run, letto via predicate fn-by-id; abilita/disabilita branch nella FSM senza patch compile-time.

## Success Criteria

- `rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` → 0 righe
- `rg "enum Effect" src/data/skills_ron.rs` → 0 righe
- `rg "use bevy" src/combat/blueprints/` → 0 righe
- Tutte le 18 active canon via CompiledTimeline+ExtRegistries; 6 passive via PassiveRunner
- Aggiungere Digimon scriptato tocca solo blueprints/<new>/ + data/digimon/<new>/
- cargo test verde end-to-end
- cargo check headless + windowed senza warning nuovi
- JSONL contiene CombatKernelTransition::Blueprint round-trip
- DryRun ≡ Execute (I2) verde
- HeadlessAuto Intent stream ≡ Windowed (I3 / D026)
- Determinismo (I1) verde su path RNG-gated

## Slices

- [x] **S01: S01** `risk:medium` `depends:[]`
  > After this: cargo check headless + windowed puliti; CombatPlugin in main.rs; src/combat/api/ con i 7 file primitive; cast_id su CombatEvent; canary Intent::DealDamage end-to-end.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: Fixture OnTurnStart kills target verde; validate_timeline_refs scopre typo; LoopFrame single-level su chain_bolt port.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: DryRun≡Execute≡Preview verde; two-clock verde; circuit breaker @256.

- [x] **S04: S04** `risk:high` `depends:[]`
  > After this: Renamon kitsune_grace verde; JSONL Blueprint round-trip; debug_assert mismatch.

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: Tohakken + Petit Thunder via CompiledTimeline; typo→errore boot.

- [x] **S06: S06** `risk:high` `depends:[]`
  > After this: 18 active via CompiledTimeline; suite verde + Loop tier-N.

- [x] **S07: S07** `risk:high` `depends:[]`
  > After this: 6 passive via PassiveRunner; Block Reaction verde deterministico.

- [x] **S08: S08** `risk:medium` `depends:[]`
  > After this: Twin Core end-to-end; Bouncing Fire OFF≡baseline; no coupling.

- [x] **S09: S09** `risk:medium` `depends:[]`
  > After this: Predator Loop write in JSONL; Battery Loop deterministico.

- [ ] **S10: S10** `risk:medium` `depends:[]`
  > After this: Kernel digimon-free verificato grep; smoke UI 2 encounter.

- [ ] **S11: UI and AI consumers via SkillCtx Preview** `risk:low` `depends:[S06,S07]`
  > After this: UI preview damage via stream; AI score ottimale via stream.

- [ ] **S12: RosterEntry blueprint-keyed + ValidationSnapshot from registry** `risk:low` `depends:[S10]`
  > After this: Test 'add new digimon' modifica solo le 2 dir; suite verde.

## Boundary Map

## Boundary Map

Not provided.
