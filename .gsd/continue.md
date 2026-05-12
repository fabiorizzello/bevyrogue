# Continue — round-3 audit follow-up

**Branch:** `master` (3 commit ahead di `origin/master`: `6c35c9f`, `26fb43a`, `a4fea2b` + 1 sessione uncommitted)
**Data snapshot:** 2026-05-12
**Working dir:** `/home/fabio/dev/bevyrogue`

## Done in questa sessione (2026-05-12)

Doc-only consolidation pass (7 item, nessuna decisione utente richiesta):

- **X12** — Renamon K1 `kitsune_grace` self-Ult gate `no-self` propagato da `renamon/04 §5` a `renamon/00 §1` (kit shape) + `§8 D6` (decision register). `renamon/04 §5 K1` marcato chiuso.
- **§G naming collision** — refs `§G-Events`/`§G-Param` rinominati a `§R-Events`/`§S-Param` per evitare collisione con `02-02b §G — Headless determinism`. Aggiunte anchor stub `02-02b §R` (kernel events catalog: `UltimateUsed`, `BlockReactionTriggered`, `DamageDealt`, `StatusApplied`, `Healed`, `SpGranted`, `IncomingDamage`) e `02-02b §S` (`ParamRef` resolution: `Static`/`Snapshot`/`BlueprintState`/`Literal`). Refs aggiornate in `renamon/04`, `tentomon/04`, `tentomon/02`, `02-02d`.
- **Broken refs `§2.9`** — 4 forward-refs al "worked example" inesistente sostituiti con marker "deferred post-M017" (`02-02b §M`, `02-02b §Riferimenti`, `02-05`, `02-06`). Anche `tentomon/02 C5` (ref errata a `§G5+§G6 di §02-02b`) ripuntata a `02-02b §F` snapshot-once + `§S-Param`.
- **X17** — `TargetShape` enum consolidato in `02-02b §C3` (11 varianti single/multi-target: Primary/Self/AdjLeft/AdjRight/SingleAlly/AdjLowest/LowestHpPctAlive/NextAliveAdj/RandomEnemyAlive/Blast/AoE/Bounce). Frammenti in `agumon/04 §G-Sel`, `gabumon/02 §5`, `tentomon/02 §5 C1`, `tentomon/03 §5 D1` marcati chiusi e ripuntati a §C3.
- **X5** — Modifier-firma → FSM mapping table aggiunta in `02-02b §C4` (4 modifier `OnKill→Detonate`, `OnStatusApplied→Echo`, `OnKill→Chain`, `OnHitN→Apply` → edge predicate + Command `on_enter` su nodo Reactive). Cross-ref aggiunto in `08 §8.1` (colonna "FSM mapping").
- **X8** — Status cross-ref `02-02b §C/§C2` ↔ `02-08 §H` formalizzato (status registry `StatusKind` + `BuffKind` canon in `02-08 §H.1/§H.2`, Commands `EmitStatus`/`ApplyBuff` in `02-02b §C/§C2`, validator wiring).
- **X10** — Block reaction consolidation: `tentomon/04` designato fonte canonica unica (FSM topology, damage pipeline ordering, stack rules). Riga `BlockReaction` Command aggiunta in `02-02b §C2`. Pre-step `IncomingDamage` documentato in `02-08 §A`. Canon-source map block aggiunto in `tentomon/04 §1`.

## Done sessioni precedenti (carry-over)

- F-A restructure (commit `65b8d39`) — 6 passive doc riallineati al Full FSM mandate (`02-02e §A.0`).
- Round-3 audit closure low-effort (commit `6c35c9f`):
  - X1 (F-A regression cleanup), X2 (Twin Core ×1.15 canon), X3 (Confused removal v0 status set = 5),
    X4 (FoxDrive ref removal), X6 (Dematerialize DoD), X13 (Dorumon D3 propagation),
    X14 (Tentomon dual-path label), X18 (Chilled cap=6).
- Rename Agumon skill canon in RON+tests (commit `26fb43a`).
- Sprite pipeline config refresh (commit `a4fea2b`).

## Audit items ancora aperti (richiedono decisione utente)

| ID | Item | Tipo |
|---|---|---|
| **X7** | Patamon role drift: `08 §8.5` tank-lite vs `patamon/00 §1` healer-support — scegli lane | Decisione |
| **X9** | "tank-lite" vocab canonization in `02-02e` / `02-08` (oggi solo in `tentomon/00`) | Scope |
| **X11** | AdvanceTurn/DelayTurn speed-stat constraint (% gauge no flat) in `02-02b` | Scope |
| **X16** | Terminology "modifier" overloaded (playhead `02-02b:27` / firma `08:29` / pattern `02-02c:1`) | Naming decision |

## Decisioni canonizzate (carry-over)

- **Twin Core damage modifier = ×1.15 moltiplicativo** (HSR-style buff stacking).
- **Renamon time-manip metric = % gauge** (no flat speed-stat).
- **Blessed × Twin Core = no interaction** (Twin Core legge solo Agumon↔Gabumon).
- **FoxDrive `OnBreak→Detonate` rimosso** dal kit Renamon.
- **Status v0 set = 5**: Heated, Chilled, Slowed, Paralyzed, Blessed (Confused droppato).
- **Chilled cap = 6** (allineato a Heated, refresh durata).
- **Renamon `kitsune_grace` self-Ult gate = no-self** (X12, 2026-05-12).
- **`TargetShape` enum chiuso 11 varianti, blueprint-side resolver** (X17, 2026-05-12).
- **Block Reaction canon source = `tentomon/04`** (X10, 2026-05-12).

## Open threads cross-doc (carry-over multi-sessione)

- K6 travel direction semantics (02-02b/02-02e).
- B4/B5/B8/B9 — Tentomon kernel surface: BlockReactionTriggered event ✅ formalizzato (`02-02b §R-Events`), BuffComponent_* convention chiuso (`02-02e §E`). **Naming collision §G ✅ risolto X10/§G rename.**
- N5/N5.5/N6 — Forma C convention in `02-02e` (Forma C framing abbandonata, sub-variant A/B/C in `02-02e §A.1`).
- C5-extended snapshot scope (`02-02b`) — `gabumon/02 §5.C5` distinzione skill-commit vs edge-commit ancora open.
- G1/G11 — Tentomon SP override + Ult charge `OnAnyAttack` semantics.
- Broken refs noti residui: nessuno (tutte fixate questa sessione).

## Stat baseline drift identity ↔ `units.ron` (RON-side, fuori scope doc audit)

Deferrato per esplicita istruzione utente ("non è importante quello che c'è nei file ron"). Per allinearlo in futuro:
- N2 Renamon ult_trigger 120 vs RON 100.
- N3 baby_flame.sp_cost = 4 vs identity 1.
- N7 Tentomon hp_max ~120 vs RON 92.
- N8 Gabumon hp_max=110 toughness=60 vs RON 95/48.
- N9 Patamon hp_max=95 ult_trigger=100 vs RON 88/80.

## Possibili prossimi passi

1. **Commit della sessione** — doc-only changes spread su 9 file (`02-02b`, `02-02d`, `02-05`, `02-06`, `02-08`, `08`, `renamon/00`, `renamon/04`, `gabumon/02`, `tentomon/02`, `tentomon/03`, `tentomon/04`, `agumon/04`).
2. **X16 fast close** — naming decision "modifier" overload (decisione semantica, ma può essere rapida se l'utente sceglie convenzione).
3. **X7/X9 Patamon lane** — decisione healer vs tank-lite-with-heal (impatta `patamon/00`, `08 §8.5`, `02-02e`).
4. **X11 turn-manip metric vocab** — formalizzare in `02-02b §C2` che AdvanceTurn/DelayTurn pct è del gauge (no speed flat).
5. **Stat baseline RON sync** — Quando il design è frozen, riallineare `units.ron` ai valori identity §5.
