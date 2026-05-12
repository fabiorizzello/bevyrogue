# Tentomon — Skill: `petit_thunder` (Bounce(3) + OnHit3 Paralyzed + self DR)

> **Goal**: primo skill **Bounce** del roster. Stress test target shape "chain jump" + edge condizionato a hit count. Più self-buff DR concorrente con identità tank-lite.
>
> **Canon ref:** Tentomon "Petit Thunder" (id 372) — "fires static electricity amplified by its wings". Mapping Bounce(3) = chain lightning wing-amplified arc.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6 (3 emit per chain), ApplyBuff (renamon/03 R3), ordering G4. Qui solo nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult
- **Effect:** Damage Electric medio `≈12` su 3 target consecutivi (Bounce). **`OnHitN(3)→Apply(Paralyzed)`** sul 3° target. **Self side-effect:** DR 25% self 1 turno.
- **Atlas clip:** `heavy_attack` (frames 22–33, count 12)

## §2 — FSM topology

5-nodo: `Wind → Hop1 → Hop2 → Hop3(+paralyzed) → Recovery`. Ogni hop emette damage al target risolto + bolt `Travel` dal precedente al successivo (chain lightning).

```
commit → Wind(2f) → Hop1(2f) → Hop2(2f) → Hop3(3f) → Recovery(3f) → exit
                       │           │           │
                       │ on_enter: │ on_enter: │ on_enter:
                       │  EmitDmg  │  EmitDmg  │  EmitDmg(hop3_target)
                       │  (hop1_t) │  (hop2_t) │  EmitStatus(Paralyzed, Single(hop3_target))   ← OnHitN(3)
                       │  bolt:    │  bolt:    │  bolt:
                       │   SP("lightning_bolt_first",
                       │     origin: SelfCenter,
                       │     motion: Travel { to: EntityCenter(FromParamSnapshot("hop1_target")),
                       │                       ease: EaseOut, ms: 90 })
                       │   SP("lightning_impact",
                       │     origin: EntityCenter(FromParamSnapshot("hop1_target")),
                       │     motion: Static)                                                    ← NEW impact on hop1
                       │           │  SP("lightning_bolt_chain",
                       │           │    origin: EntityCenter(FromParamSnapshot("hop1_target")),
                       │           │    motion: Travel { to: EntityCenter(FromParamSnapshot("hop2_target")),
                       │           │                     ease: EaseOut, ms: 90 })
                       │           │  SP("lightning_impact",
                       │           │    origin: EntityCenter(FromParamSnapshot("hop2_target")),
                       │           │    motion: Static)                                         ← NEW impact on hop2
                       │           │           │  SP("lightning_bolt_chain",
                       │           │           │    origin: EntityCenter(FromParamSnapshot("hop2_target")),
                       │           │           │    motion: Travel { to: EntityCenter(FromParamSnapshot("hop3_target")),
                       │           │           │                     ease: EaseOut, ms: 90 })
                       │           │           │  SP("lightning_impact",
                       │           │           │    origin: EntityCenter(FromParamSnapshot("hop3_target")),
                       │           │           │    motion: Static)                             ← NEW impact on hop3
                       │           │           │  SP("paralysis_lock",
                       │           │           │    origin: EntityCenter(FromParamSnapshot("hop3_target")),
                       │           │           │    motion: Static)
                       │           │           │  ApplyBuff(self, dr_self, dur:1)              ← tank hook
                       │           │           │  Shake { intensity:2, duration_ms:100 }
```

Bounce target resolution: blueprint risolve al commit la chain (`hop1_target` = primary, `hop2_target` = next-alive adj, `hop3_target` = next). Modalità:
- **A.** Risolti tutti al commit (snapshot). Se `hop2_target` muore prima di `Hop2`, hop salta o si re-targeta?
- **B.** Risolti just-in-time a ogni `on_enter` del hop. Più dinamico, ma rompe snapshot-once (G5).

**Decisione consigliata:** A (snapshot) + skip-on-dead (se target morto al hop, hop diventa no-op visivo, niente re-target). Coerente con G5 snapshot model. Snapshot keys `hop1_target / hop2_target / hop3_target` scritte nel param snapshot dal commit-time resolver e lette da `EntityCenter(FromParamSnapshot(...))` per i bolt (§2.2d §B + §H.4 snapshot rule).

**Travel-on-death policy:** se l'entità referenziata dal `Travel.to` muore tra `on_enter` del bolt e il termine dei 90ms, il VFX continua verso la posizione snapshot al momento dello spawn (§2.2d §H.4). Niente re-target visivo.

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Wind` | 2 | 22–23 | `SpawnParticle("static_charge", origin: SelfCenter, motion: Static)` |
| `Hop1` | 2 | 24–25 | `EmitDamage(hop1_target)` + `SpawnParticle("lightning_bolt_first", origin: SelfCenter, motion: Travel{to: EntityCenter(FromParamSnapshot("hop1_target")), ease: EaseOut, ms: 90})` + `SpawnParticle("lightning_impact", origin: EntityCenter(FromParamSnapshot("hop1_target")), motion: Static)` |
| `Hop2` | 2 | 26–27 | `EmitDamage(hop2_target)` + `SpawnParticle("lightning_bolt_chain", origin: EntityCenter(FromParamSnapshot("hop1_target")), motion: Travel{to: EntityCenter(FromParamSnapshot("hop2_target")), ease: EaseOut, ms: 90})` + `SpawnParticle("lightning_impact", origin: EntityCenter(FromParamSnapshot("hop2_target")), motion: Static)` |
| `Hop3` | 3 | 28–30 | `EmitDamage(hop3_target)` + `EmitStatus(Paralyzed, hop3_target)` + `ApplyBuff(self, dr)` + `SpawnParticle("lightning_bolt_chain", origin: EntityCenter(FromParamSnapshot("hop2_target")), motion: Travel{to: EntityCenter(FromParamSnapshot("hop3_target")), ease: EaseOut, ms: 90})` + `SpawnParticle("lightning_impact", origin: EntityCenter(FromParamSnapshot("hop3_target")), motion: Static)` + `SpawnParticle("paralysis_lock", origin: EntityCenter(FromParamSnapshot("hop3_target")), motion: Static)` + shake |
| `Recovery` | 3 | 31–33 | — |

Frame budget: 12 = atlas. ✅

## §4 — Kernel events expected

```
Hop1.on_enter → DamageDealt(hop1_target, ≈12, Electric)
Hop2.on_enter → DamageDealt(hop2_target, ≈12, Electric)
Hop3.on_enter →
  ├─ DamageDealt(hop3_target, ≈12, Electric)
  ├─ StatusApplied(hop3_target, Paralyzed, dur)
  └─ BuffApplied(Tentomon, dr_self, value:0.25, dur:1)
```

## §5 — Open questions (nuovi)

1. **C1 — `TargetShape::Bounce { hits: u8, selector: NextAliveAdj | Random }`.** ✅ **Chiuso (round-3, 2026-05-12, X17): formalizzato in `02-02b §C3`** come `Bounce { hits: u8, selector: Box<TargetShape> }` (selector è un altro `TargetShape`, supporta `NextAliveAdj { side, scan: ClockWise | CounterClockWise }` o `RandomEnemyAlive { seed }`). **Canon Tentomon §1:** `selector: NextAliveAdj { side: EnemyTeam, scan: ClockWise }`, skip morti durante la chain (re-resolve ogni hop, vedi `02-02b §C3` regola 4).
2. **C2 — Bounce chain con <3 nemici vivi.** Se 2 nemici vivi:
   - **A.** Hop3 ripete su tgt_2 (re-bounce).
   - **B.** Hop3 no-op (skip damage e skip Paralyzed).
   - **C.** Hop3 ripete su tgt_1 (full chain artificiosa).
   - **Decisione consigliata:** A (re-bounce sull'ultimo vivo). `OnHitN(3)→Paralyzed` ancora valido (3° hit landed, anche se stesso target). Bilanciamento accettabile.
3. **C3 — DR self 25% concurrent con `fur_cloak` mock?** Tentomon non ha `fur_cloak`. Self-DR è isolato. Stacking solo con `holy_aegis` (10% Patamon, additivo). Cap 50%.
4. **C4 — `Paralyzed` status definito?** Verificare `status_effect.rs`. Se non esiste, **action item:** aggiungere `Paralyzed { skip_next_turn: true | turn_gauge_freeze: 1 turn }`. Identity §4 non dettaglia il meccanismo. **Proposta:** Paralyzed = skip next turn (semplice, leggibile).
5. **C5 — Param snapshot keys `hopN_target`.** Le chiavi `hop1_target / hop2_target / hop3_target` devono essere scritte dal commit-time resolver del blueprint nel param snapshot (`02-02b §F` snapshot-once + `02-02b §S-Param ParamRef::Snapshot`) per essere lette da `EntityCenter(FromParamSnapshot(...))` nei bolt `Travel`. Convenzione naming non ancora formalizzata altrove nel roster: alternative `bounce_target_1..n` o `chain_hop_N`. **Decisione consigliata:** `hopN_target` (compatto, esplicito sulla semantica "bounce hop"). Verificare conflitto con eventuali altre skill chain (nessuna nel roster minimal). Da promuovere a `02-02b §S-Param` come pattern documentato quando arriva un secondo bounce skill.

## §6 — Verdetto

Bounce introduce **target shape nuovo** (`Bounce { hits, selector }`). ✅ **Chiuso (X17): canonizzato in `02-02b §C3`** insieme a `AdjLowest` di Gabumon e `RandomEnemyAlive` di Tentomon ult — vocabolario `TargetShape` unificato in 11 varianti single/multi-target con resolver blueprint-side.

`OnHitN(3)→Apply(Paralyzed)` è un **edge condizionato a hit count**, ma in pratica risolto come "Hop3 on_enter applica Paralyzed" — niente edge runtime, è dichiarativo nel nodo finale. Niente nuovo verbo predicate.

`Paralyzed` da formalizzare nel status set.
