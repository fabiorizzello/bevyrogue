# §3 — Run-loop CLI (gameplay scope)

## 3.1 Stato attuale

`src/bin/combat_cli.rs` (1021 LoC): seleziona 4 alleati da roster, sceglie 1 preset encounter, esegue **un singolo encounter** fino a victory/defeat, esce.

## 3.2 Target M017

Run-loop semplificato (no meta-loop, no permadeath cross-run):

```
[Run start]
  → seleziona 4 Rookie da 6
  → encounter 1 (minion wave)
  → [post-combat] HP roll-over, SP reset, ult charge persiste?
  → encounter 2 (minion wave o random)
  → encounter 3 (mini-boss)
  → encounter 4 (minion)
  → encounter 5 (boss finale)
  → [run complete] o [run failed se party KO]
```

**Decisioni da prendere:**

| Item | Opzioni | Default proposto |
|---|---|---|
| Numero encounter per run | 3 / 5 / 7 | **5** (10-15 min di gioco) ✅ |
| HP carryover tra encounter | full heal / no heal / parziale | **parziale (50% missing HP curato)** ✅ |
| SP carryover | reset / persiste | **reset a 3** ✅ |
| Ult charge carryover | reset / persiste | **persiste (premia chi non l'ha sparato)** ✅ |
| KO permanenti? | sì / revivable | **revivable solo via skill in-combat** (no auto-revive post-combat) ✅ |
| Scelta encounter | lineare / branching (StS-like map) | **lineare M017**, branching dopo ✅ |
| Skill upgrade tra encounter | sì / no | **no** — out of scope M017 ✅ |
| Reward post-encounter | sì / no | **no** — out of scope M017 ✅ |

**KO revivable — design implications:**
- **Revive è skill-driven, non automatico.** Già implementato: `patamon_revive` "Holy Revive" in `assets/data/skills.ron:315` con `Effect::Revive(25)` (25% HP). In M017 va validato il mapping `Revive(pct)` → `KernelEffect::Revive { target, hp_pct }` come parte del refactor §2.3.
- **Niente auto-revive post-combat.** Ogni revive deve essere il risultato dell'uso di una skill in combat (oggi Patamon, in futuro altre Digimon possono averla con %  diverse). Coerente con la regola "ogni skill è diversa" — il revive è una scelta del kit, non una meccanica gratuita.
- **HP carryover (§3.2 tabella) si applica solo a Digimon non-KO.** Un Digimon entrato in KO durante l'encounter resta KO al prossimo encounter salvo che venga revivato in-combat *prima* del termine.
- **Run failed condition:** tutti e 4 KO **simultaneamente** in un encounter → run fail. Se almeno 1 sopravvive ma 3 sono ancora KO al prossimo encounter, la run continua a 4 vs N con 3 alleati incoscienti — il giocatore deve farli rientrare con Holy Revive (o accettare di giocare a -3 fino al boss).

## 3.3 Loop seam nella codebase

Attualmente `CombatState` ha `CombatPhase` ma il binding "encounter complete → next encounter" non esiste. Serve uno strato sopra: `RunState { current_encounter: u8, party: Vec<UnitId>, ko_set: HashSet<UnitId> }`.

Resource Bevy a livello App, sopra al combat plugin. Quando `check_victory_system` osserva victory → emette `RunEvent::EncounterCleared` → un sistema `advance_run` resetta `CombatState` + bootstrap del prossimo encounter.

## 3.4 Verifica

Headless: integration test `tests/run_loop.rs` che esegue una run completa con AI scriptata (target dummy) e asserta:
- 5 encounter completati in sequenza
- Party stato consistente tra encounter (HP carryover, ult charge carryover)
- Run termina con `RunComplete` o `RunFailed`

Interactive CLI: lanciabile da terminale, una run dura ~10 min.
