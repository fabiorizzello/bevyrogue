# Tentomon — Basic: `hard_claw`

> **Goal**: baseline melee claw con **Electric tag via VFX** (claws + static) + **+2 SP gen** (vs +1 standard). Eccezione battery-role nel `SpEarned` payload.
>
> **Canon ref:** Tentomon "Katai Tsume / Hard Claw" (id 375). Tag Electric flavor-canon: Tentomon ha più mosse claws-electric (Shock Jaw 378 "spins covered in electricity", Rhino Spin 376 "charged with electricity").
>
> **Gap §2.2b condivisi:** params G1, ordering G4. Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** **+2 SP** (battery role, vs +1 default), +25 Ult.
- **Effect:** Damage Electric `≈5` (basso). Identità: SP-gen >> damage.
- **Tag:** Electric (claws + static VFX, no charge buildup — quick swipe).
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Wind → Strike → Recovery`. Niente charge: claw è quick/lightweight (battery basic deve essere fast-cycle).

```
commit → Wind(2f) → Strike(4f) → Recovery(3f) → exit
                       │
                       │ on_enter:
                       │   EmitDamage { hits:1, mul_param:"basic_mul" }
                       │   SpawnParticle("static_claw", origin: SelfCenter, motion: Static)
                       │   SpawnParticle("static_spark_impact", origin: EntityCenter(Primary), motion: Static)  // NEW — impact on target
                       │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Wind` | 2 | 0–1 | `SpawnParticle("claw_charge", origin: SelfCenter, motion: Static)` (subtle static) |
| `Strike` | 4 | 2–5 | `EmitDamage` + `SpawnParticle("static_claw", origin: SelfCenter, motion: Static)` (weapon-side flash) + `SpawnParticle("static_spark_impact", origin: EntityCenter(Primary), motion: Static)` (impact on target) + shake |
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
4. **B8 — Electric tag su melee-claw consistente?** Damage pipeline applica weakness/resist via tag, non via animation. Hard Claw con tag Electric → enemy weak-Electric prende crit-mul anche se attacco è claw. **Decisione consigliata:** OK. Tag = damage type, claws sono delivery vehicle. Allinea a Renamon basic se cross-element (verificare).

## §6 — Verdetto

Basic ordinario claw tranne `+2 SP` e tag Electric flavor-canon. **Gap unico architetturale:** dove vive l'override (B1, decisione data-side). Tag Electric su melee è consistency win (Tentomon = electric-themed unit). Nessuna nuova command richiesta.
