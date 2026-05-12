# Tentomon — Ult: `super_shocker` (AoE Electric + Paralyzed random + SP team grant)

> **Goal**: ult AoE con effetto **random target** (Paralyzed su 1 nemico random) + **SP grant team** (battery moment).
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, ApplyBuff/EmitSpGrant (gabumon/03 S1-S2), ordering G4. Qui nuovi.

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar. Anytime off-turn.
- **Effect:** Damage Electric medio `≈25` su tutti i nemici vivi; **`Paralyzed`** su 1 nemico random (seeded); **+1 SP team** (cap-aware, vedi gabumon/03 S1).
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
                          │   // status cluster (random target)
                          │   EmitStatus { id:"paralyzed", dur_param:"para_dur",
                          │                target:RandomEnemyAlive(seed:"combat_rng") }
                          │   // SP grant cluster
                          │   EmitSpGrant { amount:1, target:Team }
                          │   SpawnParticle("lightning_storm","sky_pivot")
                          │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `BuildUp` | 5 | 49–53 | `SpawnParticle("massive_charge","antennae")` |
| `Discharge` | 6 | 54–59 | damage AoE + paralyzed random + SP grant + particle + shake |
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
3. **D3 — Cleanse Patamon vs Paralyzed.** Cleanse rimuove Paralyzed (debuff filter, vedi patamon/02). Sinergia anti-Tentomon-allied. Game-design: Tentomon è in team con Patamon raramente useremo `super_shocker` sullo stesso nemico cleansable. Stop, è team enemy-side.
4. **D4 — Paralyzed dur su Ult.** Più lungo del skill (2 turni vs 1?). Game-design.
5. **D5 — SP grant `Team`** include Tentomon? **Decisione:** sì (lui è parte del team). Self-feed valido.

## §6 — Verdetto

Conferma bisogno di:
- **`EmitSpGrant`** verbo formalizzato (S1).
- **`TargetShape::RandomEnemyAlive`** con seed deterministic (D1).
- **`RoundSpTracker.max_grants_per_round`** se decidiamo di cappare grant (D2).

Nessun nuovo concetto architetturale, estensioni di vocabolario.
