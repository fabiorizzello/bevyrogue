# Tentomon — Ult: `electrical_discharge` (AoE Electric + Paralyzed random + SP team grant)

> **Goal**: ult AoE con effetto **random target** (Paralyzed su 1 nemico random) + **SP grant team** (battery moment).
>
> **Canon ref:** Tentomon "Electrical Discharge" (id 381) — "discharges electricity from its whole body, shocking anyone near it". Mapping AoE(All) = full-body radial discharge.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, ordering G4. Verbi cross-roster **`ApplyBuff` + `EmitSpGrant`** ✅ chiusi round-3 in `02-02b §C2` (gabumon/03 S1-S2). Qui nuovi solo D1/D2/D4 (random target shape + grant cap policy + paralyzed dur).

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar. Anytime off-turn.
- **Effect:** Damage Electric medio `≈25` su tutti i nemici vivi; **`Paralyzed`** su 1 nemico random (seeded); **+1 SP team** via `EmitSpGrant` (cap-aware sul ricevente, fuori `RoundSpTracker` — vedi `02-02b §C2`).
- **Atlas clip:** `skill` (frames 49–64, count 16)

## §2 — FSM topology

3-nodo: `BuildUp → Discharge → Recovery`.

```
commit → BuildUp(5f) → Discharge(6f) → Recovery(5f) → exit
                          │
                          │ on_enter:
                          │   // damage cluster
                          │   for enemy in enemies_alive:
                          │     EmitDamage { hits:1, mul_param:"ult_mul", target:Single(enemy) }
                          │     SpawnParticle("electric_shock_impact",
                          │                   origin: EntityCenter(enemy),
                          │                   motion: Static)                              ← NEW per-target impact
                          │   // status cluster (random target)
                          │   EmitStatus { id:"paralyzed", dur_param:"para_dur",
                          │                target:RandomEnemyAlive(seed:"combat_rng") }
                          │   SpawnParticle("paralysis_lock",
                          │                 origin: EntityCenter(FromParamSnapshot("paralyzed_target")),
                          │                 motion: Static)                                ← NEW lock on random paralyzed
                          │   // SP grant cluster
                          │   EmitSpGrant { amount:1, target:Team }
                          │   SpawnParticle("lightning_storm", origin: SelfCenter, motion: Radial { range_tiles: 5.0, ms: 300 })
                          │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `BuildUp` | 5 | 49–53 | `SpawnParticle("massive_charge", origin: SelfCenter, motion: Static)` |
| `Discharge` | 6 | 54–59 | damage AoE + per-enemy `electric_shock_impact` (`EntityCenter(enemy_i)` Static) + paralyzed random + `paralysis_lock` (`EntityCenter(FromParamSnapshot("paralyzed_target"))` Static) + SP grant + `lightning_storm` Radial + shake |
| `Recovery` | 5 | 60–64 | — |

Frame budget: 16 = atlas. ✅

## §4 — Kernel events expected

```
Discharge.on_enter
  for each enemy_i:
    └─ DamageDealt(enemy_i, ≈25, Electric)
  └─ StatusApplied(random_enemy, Paralyzed, dur)
  └─ SpEarned(team_actor_k, 1) × N    (rispetta RoundSpTracker cap)
```

## §5 — Open questions (nuovi)

1. **D1 — `TargetShape::RandomEnemyAlive { seed: string }`.** Estensione vocabolario. Seed deve essere deterministico per test (default: `combat_rng` global seed). Headless: usa seed esplicito.
2. **D2 — SP grant team rompe `RoundSpTracker.max_non_basic_per_round`?** Identity §7 lo flagga.
   - **Decisione:** SP grant **non** passa dal contatore (è grant, non spese). Conta solo l'**uso** di SP per skill. Coerente.
   - Cap separato: `RoundSpTracker.max_grants_per_round` opzionale (es. 2) per evitare loop SP infinito tra Tentomon Ult + altri grant.
3. **D3 — Cleanse Patamon vs Paralyzed.** Cleanse rimuove Paralyzed (debuff filter, vedi patamon/02). Sinergia anti-Tentomon-allied. Game-design: Tentomon è in team con Patamon raramente useremo `electrical_discharge` sullo stesso nemico cleansable. Stop, è team enemy-side.
4. **D4 — Paralyzed dur su Ult.** Più lungo del skill (2 turni vs 1?). Game-design.
5. **D5 — SP grant `Team`** include Tentomon? **Decisione:** sì (lui è parte del team). Self-feed valido.

## §6 — Verdetto

Conferma bisogno di:
- **`EmitSpGrant`** verbo ✅ formalizzato in `02-02b §C2` (S1 chiuso round-3 2026-05-12).
- **`TargetShape::RandomEnemyAlive`** con seed deterministic (D1, ancora open — vocabolario `TargetShape` da estendere).
- **`RoundSpTracker.max_grants_per_round`** se decidiamo di cappare grant (D2, open — playtest M015+).

Nessun nuovo concetto architetturale, estensioni di vocabolario.
