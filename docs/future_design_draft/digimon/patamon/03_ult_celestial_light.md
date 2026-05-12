# Patamon — Ult: `celestial_light` (AoE team heal + cleanse)

> **Goal**: stress test AoE ally + multi-emit (heal × N + cleanse × N). Caso degenere "Commands a target plurale".
>
> **Gap §2.2b condivisi:** params G1, multi-target G6 (3 emit separati). Qui nuovi.

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar. Anytime off-turn.
- **Effect:** Heal `~35% HP max` a tutta la squadra; **Cleanse 1 debuff** per ogni alleato (FIFO oldest).
- **Atlas clip:** `skill` (frames 62–76, count 15)

## §2 — FSM topology

3-nodo: `Ascend → Radiate → Descend`.

```
commit → Ascend(5f) → Radiate(5f) → Descend(5f) → exit
                        │
                        │ on_enter:  (decisione G6: 1 emit per alleato vivo)
                        │   for ally in team_alive:
                        │     EmitHeal { amount_param:"ult_heal_pct", target:SingleAlly(ally) }
                        │     EmitCleanse { count:1, target:SingleAlly(ally),
                        │                   priority:"oldest_first", filter:"debuff_only" }
                        │   SpawnParticle("celestial_pillar","center_pivot")
                        │   Shake { intensity:3, duration_ms:160 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Ascend` | 5 | 62–66 | `SpawnParticle("holy_uplift","feet")` |
| `Radiate` | 5 | 67–71 | N×(heal+cleanse) + particle + shake |
| `Descend` | 5 | 72–76 | `SpawnParticle("holy_settle","ground")` |

Frame budget: 15 = atlas. ✅

## §4 — Kernel events expected

```
Radiate.on_enter  (team size = N, vivi)
  for each ally i:
    ├─ HealApplied(ally_i, amount, Patamon)
    └─ CleanseApplied(ally_i, [debuff_id_or_none])
```

Self-target incluso? Sì (Patamon è in `team_alive`).

## §5 — Open questions (nuovi)

1. **P6 — `target_shape: AoE(AllyTeam)`.** Estensione di `TargetShape`:
   - `TargetShape::AoE { side: EnemyTeam | AllyTeam | All, exclude_dead: bool }`
   - **Resolution G6 path A:** blueprint espande in N `EmitHeal`/`EmitCleanse` separati al commit. Coerente con agumon/03 §6.3.
   - **Resolution alternativa:** `EmitHeal { target: AoE(AllyTeam) }` con kernel che cicla. Più compatto in RON, più magico nell'engine. **Decisione:** path A (espansione blueprint), uniformità con damage AoE.
2. **P7 — Ordering heal+cleanse intra-alleato.** Per ally_i: heal prima o cleanse prima?
   - **Decisione:** cleanse **prima** (oldest debuff potrebbe essere `Slowed`/`Paralyzed` che non scalano heal, ma sintatticamente meglio: stato pulito → heal applicato). Ordine RON = ordine di emissione, ma documentare.
3. **P8 — Patamon morto Ult auto-cast?** No — se Patamon è morto, Ult non castabile (anytime off-turn richiede caster vivo). Nessun caso "revive". Identity sheet non promette revive.
4. **P9 — Anti-spam check.** Ult heal a team a 100% HP = sprecato. Player decide quando. Nessuna validazione design-side, è game-feel.

## §6 — Verdetto

Conferma il pattern AoE-ally come **espansione blueprint** (G6 path A). Nessun nuovo verbo oltre quelli del skill (heal/cleanse già introdotti in `02_skill_holy_breeze`).

**TargetShape::AoE { side, exclude_dead }** è l'unica estensione nuova qui — utile anche per Renamon `diamond_storm` (AoE enemies).
