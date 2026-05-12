# Renamon — Ult: `tohakken` (AoE damage + AoE delay + team Blessed) — *"Power Paw"*

> **Goal**: caso più ricco del kit Renamon. Triplo effect AoE: damage enemies + delay enemies + buff allies. Nessun edge reattivo (decisione identity §8: niente OnBreak→Detonate).
>
> **Canon:** Tōhakken / 燈火拳 / "Tohachi Game" — DAPI: *"Ignites its hands and feet in blue fire and attacks the enemy with them. Hits the enemy with a powerful hand strike."*. EN dub Tamers = **Power Paw**, signature finisher Renamon. Reskin Holy: blue fire → golden Holy fire, hand-ignition genera shockwave radiale Holy che vende AoE pur su anim claw single-strike.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, time-manip T1, buff verbo S2 (vedi gabumon/03), ordering G4. Qui solo nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar (`ultimate_trigger=120`, più alto del resto). Anytime off-turn.
- **Effect:** Damage Holy `≈40` su tutti i nemici; **`DelayTurn(all enemies, 30%)`**; **Apply `Blessed`** a tutti gli alleati (2 turni).
- **Atlas clip:** `heavy_attack` (frames 20–29, count **10**) — claw + hand-ignition. AoE veicolata via **VFX shockwave radiale** (Power Paw blue fire reskin Holy).

## §2 — FSM topology

3-nodo: `Ignite → Strike → Recovery`. Frame budget 10 = 3+4+3 (compressed da 12 → 10 vs rev1).

```
commit → Ignite(3f) → Strike(4f) → Recovery(3f) → exit
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
                        │   // VFX layer 1 — radial AoE shockwave (driver visivo AoE)
                        │   SpawnParticle("holy_shockwave",
                        │                 origin: SelfCenter,
                        │                 motion: RadialOut { range_tiles: 5.0, ms: 200 })
                        │   // VFX layer 2 — phantom paw projectile per-enemy (canon "Power Paw" volante)
                        │   for enemy_i in enemies_alive:
                        │     SpawnParticle("phantom_paw_projectile",
                        │                   origin: SelfHands,
                        │                   motion: Travel { to: EntityCenter(<iter:enemy_i>),
                        │                                    ease: EaseOut, ms: 100 })
                        │     SpawnParticle("phantom_paw_impact",
                        │                   origin: EntityCenter(<iter:enemy_i>), motion: Static)
                        │   // VFX layer 3 — blessed motes per-ally (invariato)
                        │   for ally_j in team_alive:
                        │     SpawnParticle("blessed_motes",
                        │                   origin: EntityCenter(<iter:ally_j>),
                        │                   motion: RiseUp { ms: 800 })
                        │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Ignite` | 3 | 20–22 | `SpawnParticle("holy_fire_ignite", origin: SelfHands, motion: Static)` (hands glow gold/azure, telegraph) |
| `Strike` | 4 | 23–26 | 3 cluster emit + 3 VFX layer (radial shockwave + N phantom_paw_projectile Travel + N phantom_paw_impact + M blessed_motes RiseUp) + shake |
| `Recovery` | 3 | 27–29 | — |

Frame budget: 10 = atlas. ✅

**VFX layering critico:** la AoE viene **comunicata visualmente** da tre layer congiunti: (1) `holy_shockwave` radial expand da `SelfCenter` raggiunge tutti i nemici (driver AoE); (2) `phantom_paw_projectile` Travel da `SelfHands` a ogni `EntityCenter(<iter:enemy_i>)` sells canon "Power Paw" che vola; (3) `phantom_paw_impact` Static su ogni nemico sincronizzato all'arrival del Travel (impact frame). Senza questa pipeline VFX, l'anim claw single-strike non legge AoE. Vedi §2.2d positioning.

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
7. **R9 — `EntityCenter(<iter:enemy_i>)` grammar.** Shortcut precedenti (`EnemyTeamPerEntity`/`AllyTeamPerEntity`) non sono in §2.2d. Notazione esplicita: `<iter:loop_var>` è syntactic sugar per "G6 path A — blueprint resolver espande N emit, ognuna con `EntityRef` risolto a binding time per iter step i". Allinea al G6 path già documentato. **Action item:** formalizzare in §2.2d §B come terza modalità di `EntityRef` (oltre a `Primary`/`FromParamSnapshot`/`EventTarget`/`Caster`/`Self`). Gap N8 nuovo.
8. **R10 — Travel sync impact.** Il pair `phantom_paw_projectile` (Travel 100ms) + `phantom_paw_impact` (Static) richiede che l'impact spawn sia **timing-locked** all'arrival del Travel. Se il VFX system non garantisce sync (es. spawn at on_enter T+0 entrambi → impact appare prima del Travel arrival), serve una primitiva `Travel { …, on_arrive: SpawnParticle(...) }` o spawn delayed (T+100ms). Convention §2.2d da chiarire. Per ora: assumiamo spawn separato a stesso frame e il preset `phantom_paw_impact` ha un delay interno animato (4f wait + impact).

## §6 — Verdetto

Tōhakken (ex `fox_drive`) consolida il bisogno di:
- **Verbo unificato `ApplyBuff`** con `kind:Buff` (R3, R4).
- **Verbo `DelayTurn` con target_shape** (allinea a T1).
- **Espansione blueprint multi-cluster** (3 emit-loop sullo stesso `on_enter`, ordinati).

Nessuna nuova famiglia di gap. **3 verbi nuovi** (`ApplyBuff`, `AdvanceTurn`, `DelayTurn`) sono ora richiesti dal vocabolario.

**Rev2 swap (2026-05-12):** atlas `heavy_attack` 20–29 (10f) sostituisce `skill` 45–56 (12f). Frame budget compresso 12→10. **Effetto meccanico invariato** — sola differenza è VFX-driven AoE veicolata da `holy_shockwave` radial + per-entity bursts, non da anim wide sweep. Rename da `fox_drive` (non-canon) a `tohakken` (Power Paw, canon Tamers).
