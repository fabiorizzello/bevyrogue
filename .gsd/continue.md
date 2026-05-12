# Continue — round-3 audit follow-up

**Branch:** `master` (3 commit ahead di `origin/master`: `6c35c9f`, `26fb43a`, `a4fea2b`)
**Data snapshot:** 2026-05-12
**Working dir:** `/home/fabio/dev/bevyrogue`

## Done in questa sessione

- F-A restructure (commit `65b8d39`) — 6 passive doc riallineati al Full FSM mandate (`02-02e §A.0`).
- Round-3 audit closure low-effort (commit `6c35c9f`):
  - X1 (F-A regression cleanup), X2 (Twin Core ×1.15 canon), X3 (Confused removal v0 status set = 5),
    X4 (FoxDrive ref removal), X6 (Dematerialize DoD), X13 (Dorumon D3 propagation),
    X14 (Tentomon dual-path label), X18 (Chilled cap=6).
- Rename Agumon skill canon in RON+tests (commit `26fb43a`).
- Sprite pipeline config refresh (commit `a4fea2b`).

## Audit items NON chiusi (richiedono decisione o effort medio)

Riferimento snapshot: prior session "audit consolidato doc-only — 18 item".

| ID | Item | Tipo |
|---|---|---|
| **X5** | `08 §8.1` modifier-firma → `02-02b §C` Command mapping table | Structural (nuova tabella) |
| **X7** | Patamon role drift: `08 §8.5` tank-lite vs `patamon/00 §1` healer-support — scegli lane | Decisione |
| **X8** | Status formalization cross-ref `02-02b §C` ↔ `02-08 §H` (Paralyzed/Slowed/Blessed/Heated/Chilled) | Multi-doc cross-ref |
| **X9** | "tank-lite" vocab canonization in `02-02e` / `02-08` (oggi solo in `tentomon/00`) | Scope |
| **X10** | Block reaction doc fragmentation: `tentomon/00`, `02-02b`, `02-02e`, `02-08` non coordinati | Consolidation |
| **X11** | AdvanceTurn/DelayTurn speed-stat constraint (% gauge no flat) in `02-02b` | Scope |
| **X12** | Renamon K1 self-Ult gate: `renamon/04 §5` propone no-self, `renamon/00 §1` neutrale | Decisione (proposta già canon) |
| **X16** | Terminology "modifier" overloaded (playhead `02-02b:27` / firma `08:29` / pattern `02-02c:1`) | Naming decision |
| **X17** | TargetShape enum consolidato in `02-02b §C` (AdjLowest, LowestHpPctAlive, NextAliveAdj, AoE(All), Bounce, RandomEnemyAlive) | Structural |

## Decisioni canonizzate (carry-over)

- **Twin Core damage modifier = ×1.15 moltiplicativo** (HSR-style buff stacking).
- **Renamon time-manip metric = % gauge** (no flat speed-stat).
- **Blessed × Twin Core = no interaction** (Twin Core legge solo Agumon↔Gabumon).
- **FoxDrive `OnBreak→Detonate` rimosso** dal kit Renamon.
- **Status v0 set = 5**: Heated, Chilled, Slowed, Paralyzed, Blessed (Confused droppato).
- **Chilled cap = 6** (allineato a Heated, refresh durata).

## Open threads cross-doc (carry-over multi-sessione)

- K6 travel direction semantics (02-02b/02-02e).
- B4/B5/B8/B9 — Tentomon kernel surface: BlockReactionTriggered event, BuffComponent_* convention. **Naming collision**: §G in `02-02b` = "Headless determinism"; i patch programmati `§G-Events` (B9) e `§G-Param` (N5/X-ref) confliggono → rinominare a `§R-Events` / `§S-Param` o sotto-§C2/§G.
- N5/N5.5/N6 — Forma C convention in `02-02e`.
- C5-extended snapshot scope (`02-02b`).
- G1/G11 — Tentomon SP override + Ult charge `OnAnyAttack` semantics.
- Broken refs noti residui:
  - `02-02b §2.9 (worked example)` citata in 02-02b:453, 02-02b:568, 02-05:18, 02-06:88 ma §2.9 inesistente.
  - `02-02b §G-Param` citata in 02-02d:68 + 02-02d:279 ma §G attuale = Headless determinism.

## Stat baseline drift identity ↔ `units.ron` (RON-side, fuori scope doc audit)

Deferrato per esplicita istruzione utente ("non è importante quello che c'è nei file ron"). Per allinearlo in futuro:
- N2 Renamon ult_trigger 120 vs RON 100.
- N3 baby_flame.sp_cost = 4 vs identity 1.
- N7 Tentomon hp_max ~120 vs RON 92.
- N8 Gabumon hp_max=110 toughness=60 vs RON 95/48.
- N9 Patamon hp_max=95 ult_trigger=100 vs RON 88/80.

## Possibili prossimi passi

1. **X12 fast close** — Renamon K1 self-Ult `no-self` gate (decision-pending ma proposta già canon in `04 §5`).
2. **X5 structural** — Mapping table 4 modifier-firma → FSM edge + Command in `02-02b §C`.
3. **X17 structural** — Consolidare TargetShape enum in `02-02b §C` (impatta i 6 skill doc Digimon).
4. **Naming collision §G** — Risolvere prima di applicare B8/B9 patch (rinominare a §R-Events/§S-Param).
5. **Stat baseline RON sync** — Quando il design è frozen, riallineare `units.ron` ai valori identity §5.
