# Tentomon — Basic: `petit_thunder`

> **Goal**: baseline electric + **+2 SP gen** (vs +1 standard). Eccezione battery-role nel `SpEarned` payload.
>
> **Gap §2.2b condivisi:** params G1, ordering G4. Qui nuovi.

## §1 — Intent

- **Cost:** 0 SP — **Gen:** **+2 SP** (battery role, vs +1 default), +25 Ult.
- **Effect:** Damage Electric `≈5` (basso). Identità: SP-gen >> damage.
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Charge → Zap → Recovery`.

```
commit → Charge(2f) → Zap(4f) → Recovery(3f) → exit
                       │
                       │ on_enter:
                       │   EmitDamage { hits:1, mul_param:"basic_mul" }
                       │   SpawnParticle("petit_arc","antennae")
                       │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Charge` | 2 | 0–1 | `SpawnParticle("static_buildup","antennae")` |
| `Zap` | 4 | 2–5 | damage + particle + shake |
| `Recovery` | 3 | 6–8 | — |

Frame budget: 9 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt(target, ≈5, Electric)`
2. `SpEarned(Tentomon, 2)`   ← **+2 anziché +1** (override battery)
3. `UltimateCharged(Tentomon, 25)`

## §5 — Open questions (nuovi)

1. **B1 — Override `+2 SP` viene da blueprint o da `units.ron`?**
   - **A.** `units.ron.sp_gen_per_basic: 2` (data-side).
   - **B.** Blueprint Tentomon emette `EmitSpGrant { amount:1, target:Self }` in più sull'on_enter (oltre al default +1 del kernel).
   - **Decisione consigliata:** A. Coerente con data/logic separation. Senza override, default +1.
2. **B2 — Tag Electric esiste nel sistema?** Verificare `src/combat/types.rs::DamageTag`. Se solo Fire/Ice/Holy/Dark, **action item:** aggiungere Electric. Necessario per Tentomon weakness/resist consistency.
3. **B3 — Speed=85 implica Tentomon spesso basa.** Battery sustain regge solo se basic frequente. Game-feel da playtest.

## §6 — Verdetto

Basic ordinario tranne `+2 SP`. **Gap unico:** dove vive l'override (B1, decisione data-side). Nessuna nuova command richiesta.
