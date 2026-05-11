# M011: Combat Architecture & Synergistic Roster (v5.3 alignment)

**Vision:** Allineare l'engine a combat_design.md v5.3, sbloccare il bug pipeline residuo da M010, riscrivere il roster MVP a 6 linee con archetipi distinti (Burn / Slow-sustain / Battery / Tempo / Break / Holy-sustain), produrre uno strumento di playtest CLI per validare il rebalance, e chiudere il milestone con un encounter end-to-end giocabile sotto TTK target verificabili.

## Success Criteria

- Tutti i 21+ binari di integration verdi a fine milestone
- R070, R071, R073, R075-R081 active → validated; R074 active (schema-only, validated in M012); R082, R083 validated
- combat_design.md sez. 1, 2, 5, 6, 9 allineate al codice
- 4 nuove decisioni in DECISIONS.md (D043, D044, D045, D046); 2 superseded rimosse (D016, D026)
- UAT manuale 30 minuti firmato via combat_cli a fine S09

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: cargo test passa al 100%; nuovo integration tests/pipeline_dispatch.rs esercita declared→pre→apply→resolve end-to-end via CombatEvent bus

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: scenario test Greymon vs Devimon: log JSONL mostra tag_mod, triangle_mod e final_dmg coerenti; cargo run via combat_cli (post-S04) replica i numeri attesi

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: cargo test verde dopo migrazione; tutti gli UnitDef hanno evo_stage esplicito; test dedicato verifica fail-loud su file legacy

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: cargo run --bin combat_cli permette di selezionare 4 alleati dal roster con inquire, di scegliere azioni interattivamente in console, di vedere lo stato post-azione e gli eventi del bus

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: scenario CLI: 3 turni di Basic con un Child mostrano discount al 3° skill; cap SP enforced via test che prova a sforare

- [x] **S06: S06** `risk:medium` `depends:[]`
  > After this: scenario CLI con boss: 3 hit consecutivi di Slow mostrano resistenza crescente; test parametrizzato verifica la curva

- [x] **S07: S07** `risk:medium` `depends:[]`
  > After this: scenario CLI: rompere un Armored mostra 2 colpi richiesti; Break Seal blocca successivi tentativi nel turno

- [x] **S08: S08** `risk:high` `depends:[]`
  > After this: scenario CLI Greymon: primo hit fire del round genera +5 Energy via Form Identity; scenario re-entrancy chain bounded da stack/cooldown nei dati, niente cap engine

- [x] **S09: S09** `risk:medium` `depends:[]`
  > After this: tests/scenarios/ con 3 fixture verdi per TTK target; UAT manuale 30 minuti firmato dal product owner via combat_cli; combat_design.md sez. 9 finale

## Boundary Map

**In scope:** action pipeline unblock, Damage Tag rename, Attribute Triangle v5.3 modifiers, EvoStage schema, Resource caps + Child mechanics, Tempo Resistance + Min Threshold, Toughness 3 categorie + Break Seal, Form Identity framework + 6 Adult MVP, Combat CLI playtest harness, Numerical rebalance.

**Out of scope (→ M012):** Tamer Gauge resource + 3 Commands (Data Scan / Emergency Guard / Retreat), Enemy Counterplay 4 trait (Type Trap / Reactive Armor / Break Seal nemico / Tempo Anchor), Charged Attacks telegrafati con Danger Window, DNA Chips schema RON.

**Out of scope (post-MVP):** istanze concrete DNA Chips (lista 6-8 chip), gameplay Perfect/Ultimate/SuperUltimate, linee Digimon tagliate (Guilmon, V-mon, Huckmon, Impmon, Plotmon, Terriermon).
