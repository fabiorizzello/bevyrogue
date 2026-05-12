# §5 — Slice candidate (allineato §8 roster minimal)

> Decomposizione M017 in slice indipendenti, ordinate per rischio. **Allineato a §8 minimal scope**: 6 Rookie con kit uniforme (Basic/Skill/Ult/Passive), 5 modifier-firma chiusi, niente skill-tree, niente status set extension, niente QTE. Rispetto alla versione precedente, le slice di porting skill massivo e migrazione full hook-taxonomy (kind A-F) sono **deferred fuori M017**.

| Slice | Title | Risk | Depends |
|---|---|---|---|
| S01 | `KernelEffect` bus generico + ponte `From<*Transition>` legacy (coesistenza) + cascade drain scaffold | **high** | — |
| S02 | Migrazione 5 blueprint subsystem a `Bevy Plugin` (kernel.rs ripulito da `*Transition`) + cleanup legacy | **high** | S01 |
| S03 | `unit_stats.ron` + `signal_bindings.ron` (estrarre numeri da `units.ron` e `skills.ron`, hot-reload check) | medium | S02 |
| S03b | `SkillBehavior` trait + registry + `SkillExecCtx` (§2.7) — preview / legality / execute; cascade drain riscritto + `IdempotencyScope` + `CauseChain` propagation; **modifier-firma vocabulary v0 (5 voci §8.1)** come enum chiuso. Niente hook taxonomy A-F estesa, niente gate F complesso, niente phase-state. | **high** | S02 |
| S03c | **Skill RON v2 + porting 18 skill minimal (§8.3)** — `version: 2` su skills.ron; le 18 skill come `SkillBehavior` (la maggior parte tramite helper `DeclarativeSkill` §2.7.F; 1-2 custom Rust per Predator state check e Twin Core hook). Verifica ogni modifier-firma almeno una volta su un seed fisso. | low | S03b, S03f, S03g |
| S03d | **Kernel suspend/resume minimal** — fase `AwaitingSkillYield`, `SkillCursor`, `YieldReason::CascadeComplete` + `BlueprintHook`. QTE/PlayerChoice/AnimationGate **stub** (resolve immediato). Bastano i due reason per i modifier-firma reattivi. | medium | S03b |
| S03f | **AnimGraph FSM parser + interprete + validator (§2.2b)** — schema RON dell'`AnimGraph` (nodi/edge/Commands/Predicate), parser, validator contract test (entry, reachability, frame in-bounds, param refs esistono), `tick_fsm` puro su `(rt, kernel_events, user_inputs)`, golden test deterministico `(graph, events) → commands_sequence`. Vocabolario Commands v0: **`EmitDamage`, `EmitStatus`** (gli altri 4 verbi §2.2b deferred). Headless-only. | medium | S03b |
| S03g | **AnimGraph integration con SkillBehavior (§2.2b §H)** — `SkillExecCtx::kernel_events_since_resume()`, blueprint executor di `EmitDamage`/`EmitStatus`, `translate_command → KernelEffect`. Una skill reference (Agumon `baby_burner`) migrata come reference per il pattern Reactive node. Le altre 17 skill usano AnimGraph 3-nodi (Windup→Strike→Recovery), `baby_burner` + 4 skill con modifier-firma usano 4-nodi. | medium | S03c, S03d, S03f |
| S04 | Animation 2-asset .ron schema (`clip.ron` + `animation_fsm.ron`) + 6 Digimon clip.ron derivati lossless da `_atlas.json` + hot-reload. **Nota:** S04 cura asset pipeline (load/parse/hot-reload) e i 18 FSM degenerate/reactive. Il parser/runtime FSM è in S03f. | medium | — (parallelo a S01-S03) |
| S05 | `status_effects.ron` minimo — solo i 5 status v0 in §8.3 (`Heated`, `Chilled`, `Slowed`, `Paralyzed`, `Blessed`), validazione stack rules. `Confused` rimosso (round-3 revisione 2026-05-12, da Renamon dropped). Niente status set extension oltre questi. | low | S03 |
| S06 | `encounter_balance.ron` + `WildPack` enemy variant bootstrap | low | S03, S05 |
| S07 | `run_config.ron` + run-loop state machine (multi-encounter, party persist, KO revive) | medium | S06 |
| S08 | CLI playable run-loop UX | low | S07 |
| S09 | Balance pass su 6 Rookie + 3 enemy hand-crafted + wild curve (solo tuning RON, niente Rust) | low | S08 |

**Slice budget: 13.** Decremento di 2 rispetto alla versione precedente — `S03e` (FollowUpIntent subsume) era pensata per migrare M015 cleanup nel nuovo modello, ma con kit minimal non sono richiesti follow-up reactive nuovi (Twin Core e Predator Loop riusano i blueprint esistenti).

## Razionali architetturali

**S01 + S02 = rischio strutturale principale.** Refactor: ~60 file di test, path di import, tipi. Vanno per primi perché ogni slice successivo eredita il churn altrimenti.

**S01 separato da S02:**
- S01: `KernelEffect` come *alternativa* alle 5 `*Transition` esistenti (coesistono via `From<*Transition>`). Tutti i test passano senza modifiche.
- S02: flip definitivo (kernel emette solo `KernelEffect`, blueprint = `Plugin`, `*Transition` spariscono).

**S03-S06 = blocco "dati in RON".** Ogni slice estrae un dominio di dati dal Rust hardcoded al RON corrispondente. Test di parità: pre/post slice il combat headless deve produrre gli stessi `CombatEvent` su seed fisso. Hot-reload deve funzionare per `animation_fsm.ron` (S04) e `encounter_balance.ron` (S06) — gate esplicito.

**S03b vs S03d/f/g — scope split:**
- S03b introduce trait + registry + cascade + cause chain + modifier-firma enum. Cascade è generica fin da subito (nessun pattern hardcoded). Yield reasons sono stub finché non arriva S03d.
- S03d aggiunge la fase `AwaitingSkillYield` con 2 yield reason minimi (`CascadeComplete`, `BlueprintHook`). QTE/PlayerChoice/AnimationGate restano stub (resolve immediato). Sufficienti per i 5 modifier-firma di §8.1.
- S03c migra le 18 skill **dopo** S03d, S03f, S03g (le skill reattive richiedono entrambi cascade e AnimGraph runtime).
- S03f puro headless (parser + interprete + validator). Niente touch a kernel o blueprint. Può girare in parallelo a S03c. Converge in S03g.
- S03g integra (`SkillBehavior::execute` chiama `tick_fsm`). Va **dopo** S03c (servono behavior per migrare), S03f (serve runtime FSM), S03d (serve suspend/resume per yield reactive).

**S04 in parallelo a S01-S03:** non tocca kernel né blueprint Rust, lavora su asset. Può iniziare appena §2.2 schema è approvato.

**S05 minimo:** solo 5 status v0. Niente Stealth/Frostbite/Burn/Bleed/Shock multipli. Status set extension = fuori M017 (vedi §8.7).

**S06-S09:** demoabili individualmente, S09 dimostra il valore della RON-ificazione (intero balance pass = solo edit RON).

## Accorpamenti possibili al planning

- S03 ↔ S05 (RON puro, dipendenze deboli).
- S03b ↔ S03d (entrambi toccano kernel, ma rischio combinato alto → meglio separati).
- S03f ↔ S03g (parser+interprete con integration; rischio: golden test si stabilizza dopo l'integration).
- S06 ↔ S07.

Decisione finale al planning di milestone.

## Ordering vincoli (riassunto)

- S01 → S02 → S03 → (S03b, S03f in parallelo) → S03d → S03g → S03c → S04/S05 → S06 → S07 → S08 → S09.
- S04 fuori dal main path (asset pipeline, da iniziare appena §2.2 è approvato).

## Cosa è uscito da M017 rispetto alla versione precedente

- **`S03e` (FollowUpIntent subsume)** — non più richiesto: Twin Core e Predator Loop riusano blueprint esistenti; il nuovo modello cascade è sufficiente.
- **Porting massivo di ~30 skill esistenti** — ridotto a 18 skill minimal (3 per Digimon) tutti scritti from-scratch nel nuovo schema.
- **Hook taxonomy A-F** — fuori scope. Solo 5 modifier-firma in vocabolario chiuso (§8.1).
- **Gate F complesso (Grace, PreyLock gate, Phase F⇄D)** — fuori scope. Le legality v0 sono semplici (SP, KO, ultimate charged).
- **`DamageMod` pipeline con `ignore_def`** — fuori scope. v0 usa il damage pipeline esistente.
- **3 spender nuovi (`meltdown_strike`, `frost_lance`, `prey_strike`)** — sostituiti dai 3 skill minimal di §8.3 (`baby_flame`, `gabumon_shot`, `dash_metal`).
- **QTE infrastructure** — fuori scope. AnimGraph in v0 non emette `StartQTE`.
