# Renamon — Ult: `fox_drive` (AoE damage + AoE delay + team Blessed)

> **Goal**: caso più ricco del kit Renamon. Triplo effect AoE: damage enemies + delay enemies + buff allies. Nessun edge reattivo (decisione identity §8: niente OnBreak→Detonate).
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, time-manip T1, buff verbo S2 (vedi gabumon/03), ordering G4. Qui solo nuovi.

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar (`ultimate_trigger=120`, più alto del resto). Anytime off-turn.
- **Effect:** Damage Holy `≈40` su tutti i nemici; **`DelayTurn(all enemies, 30%)`**; **Apply `Blessed`** a tutti gli alleati (2 turni).
- **Atlas clip:** `skill` (frames 45–56, count 12)

## §2 — FSM topology

3-nodo: `Channel → Drive → Recovery`.

```
commit → Channel(4f) → Drive(5f) → Recovery(3f) → exit
                         │
                         │ on_enter: (3 cluster di emit)
                         │   // damage cluster
                         │   for enemy in enemies_alive:
                         │     EmitDamage { hits:1, mul_param:"ult_mul", target:Single(enemy),
                         │                  tough_break_param:"ult_tough_break" }
                         │   // turn-manip cluster
                         │   DelayTurn { target_shape:AoE(EnemyTeam), pct_param:"enemy_delay_pct" }
                         │   // ally buff cluster
                         │   for ally in team_alive:
                         │     ApplyAllyBuff { id:"blessed", target:Single(ally),
                         │                     dur_param:"blessed_dur" }
                         │   SpawnParticle("fox_fire_pillar","center_pivot")
                         │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Channel` | 4 | 45–48 | `SpawnParticle("nine_tails_glow","tails")` |
| `Drive` | 5 | 49–53 | 3 cluster emit + particle + shake |
| `Recovery` | 3 | 54–56 | — |

Frame budget: 12 = atlas. ✅

## §4 — Kernel events expected

```
Drive.on_enter (team N alleati, M nemici)
  for each enemy_i:
    ├─ DamageDealt(enemy_i, ≈40, Holy)
    └─ ToughnessReduced(enemy_i, X)
  └─ TurnGaugeShifted { actors: all_enemies_alive, delta_pct: +30 }
  for each ally_j:
    └─ BuffApplied(ally_j, Blessed, dur:2)
```

## §5 — Open questions (nuovi)

1. **R3 — `ApplyAllyBuff` (target alleato) vs `ApplySelfBuff` (gabumon/03 §5.S2).** Stessa famiglia. Decisione: **unico verbo `ApplyBuff { id, target:TargetShape, value_param, dur_param }`** dove `TargetShape::SingleAlly | AoE(AllyTeam) | Self` discrimina.
2. **R4 — `Blessed` come buff cleanse-immune.** Identity §6 dice non-cleansable da nemici, e cleanse Patamon filtra solo debuff. **Implementazione:** flag `kind: Buff` su `ApplyBuff` → cleanse pipeline lo skippa by-design.
3. **R5 — Ordering 3 cluster.** Damage → delay → buff. Conta? Se delay è applicato **prima** del damage, i nemici potrebbero non agire dopo l'ult comunque. Se buff è applicato **dopo** il damage, gli alleati al loro turno successivo godono di Blessed. **Decisione:** ordine RON = ordine emission (G4): damage → delay → buff. Documentare.
4. **R6 — Blessed value tuning (+15% damage, +1 Ult gen/action) e dur=2.** Game-design, non FSM. Validare a playtest.
5. **R7 — Ult charge 120 vs 100.** Vedi identity §8. Non FSM, è economia Ult.
6. **R8 — AoE on dead enemies.** `enemies_alive` filter: i morti non vengono toccati. Trivial ma esplicitare nel blueprint resolver.

## §6 — Verdetto

Fox Drive consolida il bisogno di:
- **Verbo unificato `ApplyBuff`** con `kind:Buff` (R3, R4).
- **Verbo `DelayTurn` con target_shape** (allinea a T1).
- **Espansione blueprint multi-cluster** (3 emit-loop sullo stesso `on_enter`, ordinati).

Nessuna nuova famiglia di gap. **3 verbi nuovi** (`ApplyBuff`, `AdvanceTurn`, `DelayTurn`) sono ora richiesti dal vocabolario.
