# Continue — Digimon Skill Briefs Round

**Stato:** 6 Digimon × ~5 file (identity + 4 skill briefs) completi sotto `docs/future_design_draft/digimon/`. Tutti i brief non-agumon usano taglio **B** (~50-90 righe, identity-aligned, redirect ai gap §2.2b condivisi negli agumon files). Agumon (1-4) rimane il riferimento full-depth con stress-test FSM.

## Where it stands

- `digimon/agumon/` — full-depth (~90-140 righe/file, gap §2.2b originali catturati qui)
- `digimon/{dorumon,gabumon,patamon,renamon,tentomon}/` — taglio B, redirect a `agumon/01-04` per gap §2.2b condivisi
- `docs/future_design_draft/08_roster_minimal.md` — modificato (verificare coerenza con nuovi brief)
- `docs/combat_current.md` — **NON toccato** per scelta utente

## Gap nuovi cross-roster emersi (round-2 backlog per §2.2b)

**Selectors da formalizzare in `TargetShape`:**
- `AdjLowestX` (es. lowest-HP adjacente)
- `LowestHpPctAlive`
- `Bounce { hits, selector }` (chain attacks)
- `RandomEnemyAlive { seed }` (deterministico)
- `AoE { side, exclude_dead }`
- `SingleAlly`

**Verbi nuovi da aggiungere a `Effect`:**
- `EmitHeal`
- `EmitCleanse`
- `EmitSpGrant`
- `ApplyBuff` unificato (self/ally — oggi è frammentato)
- `AdvanceTurn` / `DelayTurn`
- `BlockReaction`
- `SetBlueprintState`

**Predicate nuove:**
- `BlueprintState { state_key, expected }` — per condizionare effetti su FSM custom (Twin Core, Predator Loop, Battery Loop, etc.)

**Eventi mancanti (event bus):**
- `CombatStarted`
- `UltimateUsed`
- `IncomingDamage` (pre-damage hook per shield/DR/reaction)
- `TurnEnded`
- `PredatorLoopResolved` — già esiste, citato per completezza

**Durata buff:**
- Variante `Permanent` (oggi solo `Turns(n)`)

**Regole DR stacking:**
- Cap 50% totale, additivo — formalizzare in resolution/damage.rs

**Tag/Status nuovi:**
- Tag `Electric` (Tentomon kit)
- Status `Paralyzed` (Tentomon Super Shocker / Battery Loop discharge) — verificare se già esiste in `status_effect.rs`

## Decisioni utente in sospeso

Alla fine dell'ultimo turno l'utente non ha scelto tra:
1. File aggregato `_findings_round2.md` (sintesi gap cross-Digimon → backlog §2.2b round-2)
2. Tornare su §2.2b per chiudere G1/G5/G9 (top-3 di agumon) + nuovi gap
3. Altro

→ **Riprendere chiedendo quale dei tre.**

## Conventions usate nei brief (taglio B)

```
# <Nome Skill>
> Gap §2.2b shared: vedi agumon/01-04. Qui solo gap nuovi specifici al kit.

## Intent
## FSM topology (nodes/edges)
## Nodes table (state → guard → emit → next)
## Kernel events expected
## Open questions
```

## Don't

- Non ri-stressare gap §2.2b già catturati negli agumon files (param plumbing, headless drop, ult charge timing, shake ms, OnKill/OnBreak edges, vocabolario Commands).
- Non toccare `docs/combat_current.md` (decisione utente: "lasciamo combatcurrent").
- Non espandere i brief B a full-depth senza richiesta esplicita.

## Mode

Caveman mode attivo (K002). Risposte terse, technical-substance exact.
