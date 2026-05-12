# Tentomon — Skill: `electro_shocker` (Bounce(3) + OnHit3 Paralyzed + self DR)

> **Goal**: primo skill **Bounce** del roster. Stress test target shape "chain jump" + edge condizionato a hit count. Più self-buff DR concorrente con identità tank-lite.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6 (3 emit per chain), ApplyBuff (renamon/03 R3), ordering G4. Qui solo nuovi.

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult
- **Effect:** Damage Electric medio `≈12` su 3 target consecutivi (Bounce). **`OnHitN(3)→Apply(Paralyzed)`** sul 3° target. **Self side-effect:** DR 25% self 1 turno.
- **Atlas clip:** `heavy_attack` (frames 22–33, count 12)

## §2 — FSM topology

5-nodo: `Wind → Hop1 → Hop2 → Hop3(+paralyzed) → Recovery`. Ogni hop emette damage al target risolto.

```
commit → Wind(2f) → Hop1(2f) → Hop2(2f) → Hop3(3f) → Recovery(3f) → exit
                       │           │           │
                       │ on_enter: │ on_enter: │ on_enter:
                       │  EmitDmg  │  EmitDmg  │  EmitDmg(target_3)
                       │  (tgt_1)  │  (tgt_2)  │  EmitStatus(Paralyzed, target:Single(tgt_3))   ← OnHitN(3)
                       │           │           │  ApplyBuff(self, dr_self, dur:1)              ← tank hook
                       │           │           │  SpawnParticle("paralysis_lock","tgt_3")
                       │           │           │  Shake { intensity:2, duration_ms:100 }
```

Bounce target resolution: blueprint risolve al commit la chain (tgt_1 = primary, tgt_2 = next-alive adj, tgt_3 = next). Modalità:
- **A.** Risolti tutti al commit (snapshot). Se tgt_2 muore prima di `Hop2`, hop salta o si re-targeta?
- **B.** Risolti just-in-time a ogni `on_enter` del hop. Più dinamico, ma rompe snapshot-once (G5).

**Decisione consigliata:** A (snapshot) + skip-on-dead (se target morto al hop, hop diventa no-op visivo, niente re-target). Coerente con G5 snapshot model.

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Wind` | 2 | 22–23 | `SpawnParticle("static_charge","antennae")` |
| `Hop1` | 2 | 24–25 | EmitDamage(tgt_1) + bounce arc particle |
| `Hop2` | 2 | 26–27 | EmitDamage(tgt_2) + bounce arc |
| `Hop3` | 3 | 28–30 | EmitDamage(tgt_3) + EmitStatus(Paralyzed, tgt_3) + ApplyBuff(self, dr) + shake |
| `Recovery` | 3 | 31–33 | — |

Frame budget: 12 = atlas. ✅

## §4 — Kernel events expected

```
Hop1.on_enter → DamageDealt(tgt_1, ≈12, Electric)
Hop2.on_enter → DamageDealt(tgt_2, ≈12, Electric)
Hop3.on_enter →
  ├─ DamageDealt(tgt_3, ≈12, Electric)
  ├─ StatusApplied(tgt_3, Paralyzed, dur)
  └─ BuffApplied(Tentomon, dr_self, value:0.25, dur:1)
```

## §5 — Open questions (nuovi)

1. **C1 — `TargetShape::Bounce { hits: u8, selector: NextAliveAdj | Random }`.** Estensione necessaria. Bounce semantica:
   - `NextAliveAdj`: hop al primo nemico vivo adiacente (clockwise dal primary). Skip morti durante la chain.
   - `Random`: pick random alive (deterministic-seed).
   - **Decisione canon Tentomon §1:** `NextAliveAdj`.
2. **C2 — Bounce chain con <3 nemici vivi.** Se 2 nemici vivi:
   - **A.** Hop3 ripete su tgt_2 (re-bounce).
   - **B.** Hop3 no-op (skip damage e skip Paralyzed).
   - **C.** Hop3 ripete su tgt_1 (full chain artificiosa).
   - **Decisione consigliata:** A (re-bounce sull'ultimo vivo). `OnHitN(3)→Paralyzed` ancora valido (3° hit landed, anche se stesso target). Bilanciamento accettabile.
3. **C3 — DR self 25% concurrent con `fur_cloak` mock?** Tentomon non ha `fur_cloak`. Self-DR è isolato. Stacking solo con `holy_aegis` (10% Patamon, additivo). Cap 50%.
4. **C4 — `Paralyzed` status definito?** Verificare `status_effect.rs`. Se non esiste, **action item:** aggiungere `Paralyzed { skip_next_turn: true | turn_gauge_freeze: 1 turn }`. Identity §4 non dettaglia il meccanismo. **Proposta:** Paralyzed = skip next turn (semplice, leggibile).

## §6 — Verdetto

Bounce introduce **target shape nuovo** (`Bounce { hits, selector }`). Concorre con `AdjLowest` di Gabumon: vocabolario `TargetShape` cresce di 2 varianti.

`OnHitN(3)→Apply(Paralyzed)` è un **edge condizionato a hit count**, ma in pratica risolto come "Hop3 on_enter applica Paralyzed" — niente edge runtime, è dichiarativo nel nodo finale. Niente nuovo verbo predicate.

`Paralyzed` da formalizzare nel status set.
