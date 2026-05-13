# M018: M018: Time-manipulation split + TargetShape resolver expansion

**Vision:** Abilitare le primitive che alimentano gran parte delle skill identity del roster: split semantico advance/delay sul turn pipeline (con cap e clamp esplicitati nel codice, non solo nel design), e resolver `TargetShape` esteso oltre `Single` con tie-break deterministico. Insieme, queste due primitive sbloccano Renamon Tōhakken, Kitsune Grace, Patamon AoE, Tentomon Bounce, e in generale ogni skill identity che chiede targeting non-`Single` o manipolazione del turn order. Foundation già presente in M017 (Slowed → TurnAdvance) viene refactored qui per non avere accumulator AV pre-cap.

## Success Criteria

- AdvanceTurn(pct) e DelayTurn(pct) sono enum variants distinti; nessun caller residuo usa il vecchio TurnAdvance signed
- Cap ±50% per chiamata e clamp [0,200] dopo somma sono enforced in codice (non solo in design doc), con test deterministici di boundary
- TargetShape resolver supporta Single + Blast + AoE(All) + Bounce(N), tie-break su slot_index ascendente, deterministico in tutti i test
- Selectors AdjLowest, LowestHpPctAlive, RandomEnemyAlive{seed}, SingleAlly disponibili nel resolver e usabili da skills.ron
- Skill di esempio (advance/delay + Bounce chain) scriptable da CLI scenario, output JSONL leggibile, ogni step verificato in headless
- cargo check + cargo test verdi, zero regressioni sui 40 binari di test esistenti, status taxonomy M017 invariata

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: Test headless deterministici per cap/clamp boundary + regressione Slowed (M017) che continua a funzionare con la nuova primitive. CLI scenario advance/delay print AV gauge step-by-step.

- [x] **S02: S02** `risk:med` `depends:[]`
  > After this: CLI scenario scripted con Blast (target primario + spillover adiacenti slot_index ±1) e AoE(All), ordine di applicazione damage stabile su 10 run. JSONL log mostra target list per ogni hit.

- [ ] **S03: TargetShape: Bounce(N) path-dependent chain con tie-break** `risk:high` `depends:[S02]`
  > After this: CLI scenario con N=3 hops, enemy che muore al hop 2: chain ricalcola hop 3 sui survivors mantenendo tie-break slot_index asc. JSONL log mostra sequenza hop completa con stato vivo/morto a ogni step.

- [ ] **S04: Selectors estesi: AdjLowest, LowestHpPctAlive, RandomEnemyAlive{seed}, SingleAlly** `risk:med` `depends:[S02]`
  > After this: CLI scenario con i 4 selector usati in skill di esempio (RON): output target stabile e seed-deterministico per Random. JSONL log mostra selector usato e target risolto per ogni skill.

## Boundary Map

## Boundary Map

Not provided.
