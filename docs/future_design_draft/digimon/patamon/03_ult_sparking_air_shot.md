# Patamon — Ult: `sparking_air_shot` (AoE damage + AoE team heal + cleanse)

> **Goal**: caso hybrid damage+heal del kit Patamon. Triplo effect AoE: damage enemies + heal allies + cleanse allies. Canon-anim literal match (atlas `skill` = boom bubble cinematic).
>
> **Canon:** Sparking Air Shot / スパーキングエアショット — DAPI: *"A glittering, powered-up version of Air Shot"*. Air Shot canon (= Boom Bubble Adventure dub) = Patamon signature damage move. Sparking variant = "glittering" → giustifica reflavor heal-splash (glittering shrapnel disperses → mist healing splatter dopo bubble burst). Atlas `skill` 15f boom bubble anim = **canon Air Shot literal**, "Sparking" distinguer via gold-glitter VFX.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6 (3 emit cluster). Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar (`ultimate_trigger=100`). Anytime off-turn.
- **Effect (hybrid):**
  - **Damage Holy ~25** a tutti i nemici vivi
  - **Heal ~20% HP max** a tutti gli alleati vivi (ridotto da 35% pure-heal per compensare damage)
  - **Cleanse 1 debuff** (oldest) per ally
- **Atlas clip:** `skill` (frames 62–76, count 15) — boom bubble cheek-inflate + projectile cinematic

## §2 — FSM topology

4-nodo: `Inhale → Glow → Expel → Recovery`. Frame budget 15 = 4+3+5+3.

```
commit → Inhale(4f) → Glow(3f) → Expel(5f) → Recovery(3f) → exit
                                  │
                                  │ Expel.on_enter (G6 path A — 3 cluster):
                                  │   // damage cluster — AoE enemies
                                  │   for enemy in enemies_alive:
                                  │     EmitDamage { hits:1, mul_param:"ult_dmg_mul", target:Single(enemy),
                                  │                  tough_break_param:"ult_tough_break" }
                                  │   // heal cluster — AoE allies
                                  │   for ally in team_alive:
                                  │     EmitHeal { amount_param:"ult_heal_pct", target:SingleAlly(ally) }
                                  │     EmitCleanse { count:1, target:SingleAlly(ally),
                                  │                   priority:"oldest_first", filter:"debuff_only" }
                                  │   // VFX layer 1 — sparking holy bubble Travel a combat center
                                  │   SpawnParticle("sparking_holy_bubble",
                                  │                 origin: SelfMouth,
                                  │                 motion: Travel { to: WorldGrid(combat_center),
                                  │                                  ease: EaseOut, ms: 300 })
                                  │   // VFX layer 2 — bubble bursts → radial split (canon "glittering shrapnel")
                                  │   SpawnParticle("holy_burst_split",
                                  │                 origin: WorldGrid(combat_center),
                                  │                 motion: RadialOut { range_tiles: 6.0, ms: 250 })
                                  │   // VFX layer 3a — per-enemy damage burst (canon-stem damage side)
                                  │   for enemy_i in enemies_alive:
                                  │     SpawnParticle("holy_damage_burst",
                                  │                   origin: EntityCenter(<iter:enemy_i>), motion: Static)
                                  │   // VFX layer 3b — per-ally heal mist splash (reflavor heal side)
                                  │   for ally_j in team_alive:
                                  │     SpawnParticle("holy_heal_mist",
                                  │                   origin: EntityCenter(<iter:ally_j>),
                                  │                   motion: RiseUp { ms: 600 })
                                  │   Shake { intensity:3, duration_ms:180 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Inhale` | 4 | 62–65 | `SpawnParticle("holy_charge_glow", origin: SelfCenter, motion: Static)` (cheek-inflate, Holy energy gathers) |
| `Glow` | 3 | 66–68 | `SpawnParticle("sparking_glitter_charge", origin: SelfMouth, motion: Static)` (telegraph "this is Sparking variant, not basic Air Shot") |
| `Expel` | 5 | 69–73 | 3 cluster emit (damage + heal + cleanse) + 4 VFX layer (bubble Travel + burst split + per-enemy damage + per-ally heal mist) + shake |
| `Recovery` | 3 | 74–76 | — |

Frame budget: 15 = atlas. ✅

**VFX layering critico (hybrid driver):** anim base = boom bubble canon literal. Hybrid damage+heal viene venduto da 4 layer:
1. `sparking_holy_bubble` Travel da Patamon → combat center (canon Air Shot ranged signature, gold-glitter distinguer)
2. `holy_burst_split` RadialOut da combat center (canon "glittering shrapnel" → mid-air burst split)
3. `holy_damage_burst` per-enemy Static (sells damage side)
4. `holy_heal_mist` per-ally RiseUp (sells heal side, mist gentle vs damage burst harsh)
Senza questa pipeline VFX, anim Air Shot single-projectile non vende AoE dual-effect. Vedi §2.2d positioning.

## §4 — Kernel events expected

```
Expel.on_enter (team N alleati, M nemici)
  for each enemy_i:
    ├─ DamageDealt(enemy_i, ≈25, Holy)
    └─ ToughnessReduced(enemy_i, X)
  for each ally_j:
    ├─ HealApplied(ally_j, ≈20% HP max, Patamon)
    └─ CleanseApplied(ally_j, [debuff_id_or_none])
```

Self-target incluso? Sì (Patamon è in `team_alive`).

## §5 — Open questions

1. **P6 — `target_shape: AoE(AllyTeam)` + `AoE(EnemyTeam)`.** Stesso skill emette su entrambi side. Decisione:
   - **Path A (G6):** blueprint resolver espande in N `EmitDamage` + M `EmitHeal`/`EmitCleanse` separati al commit. Uniformità con damage AoE puro.
   - Skill RON contiene 3 `effects: [...]` cluster ordinati: damage → heal → cleanse.
2. **P7 — Ordering 3 cluster.** Damage → heal → cleanse. Importa?
   - Damage prima: nemici prendono damage prima che alleati cure (timeline naturale)
   - Heal prima di cleanse intra-ally: ally HP topped → cleanse debuff dopo (cleanse oggetto debuff resta corretto, indipendente da HP)
   - **Decisione:** ordine RON = ordine emission (G4): damage AoE → for each ally (heal + cleanse). Documentare.
3. **P8 — Patamon morto Ult auto-cast?** No — se Patamon è morto, Ult non castabile (anytime off-turn richiede caster vivo). Nessun caso "revive". Identity sheet non promette revive.
4. **P9 — Anti-spam check.** Ult a team a 100% HP = heal sprecato ma damage ancora utile. **Hybrid risolve "ult dead" del pure-heal:** anche con team full HP, vale castare per il damage AoE. Win-condition per player.
5. **P10 — Damage scala con `Blessed` (Renamon).** Cross-roster combo: Renamon ult applica Blessed (+15% damage dealt) → Patamon ult Holy damage +15% sui nemici. Sinergia gratis, nessun gap nuovo (Blessed era già listener damage-boost).
6. **P11 — Rev2 hybrid trade-off.** Pure heal 35% → 20% heal + 25 damage. Total team-value ult cresce (sustain + tempo kill enemy). Damage canon-stem riconosce identità Boom Bubble. Identity §1 shift "pure healer" → "support-healer con damage burst".

## §6 — Verdetto

Sparking Air Shot (ex `celestial_light`) consolida:
- **3 cluster AoE pattern** (damage enemies + heal allies + cleanse allies sullo stesso skill). Estensione naturale di Renamon `tohakken` 3-cluster pattern (damage + delay + buff).
- **Hybrid damage+heal slot** — primo skill con dual-side effect (entrambi `EnemyTeam` + `AllyTeam`).
- **Canon-anim literal match**: boom bubble cinematic = Air Shot/Sparking Air Shot canon. Hybrid mechanic riconcilia anim damage canon con identity healer.

**Rev2 swap (2026-05-12):**
- Rename `celestial_light` (invented) → `sparking_air_shot` (canon Sparking Air Shot)
- Atlas clip invariato (`skill` 62–76 = boom bubble cinematic)
- Mechanic: pure-heal 35% AoE → **hybrid 25 damage AoE + 20% heal AoE + cleanse**
- Identity §1: "pure healer" → "support-healer con damage burst ult"
