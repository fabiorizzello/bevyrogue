# Renamon — Skill: `koyosetsu` (AoE Holy + self AdvanceTurn) — *"Diamond Storm"*

> **Goal**: stress test **AoE enemies** + **time-manip command** (`AdvanceTurn`). Primo skill che tocca `TurnOrder` come effect.
>
> **Canon:** Koyōsetsu / 狐葉雪 / "Fox Leaf Arrowheads" — DAPI: *"Fires a barrage of sharpened leaves at the opponent"*. EN dub Tamers = **Diamond Storm**, signature move usata ogni episodio. Reskin Holy: foglie/diamanti → diamond shards golden.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, ordering G4. Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult
- **Effect:** Damage medio Holy `≈14` su **tutti i nemici vivi** (AoE); **`AdvanceTurn(self, 25%)`** — Renamon avanza il proprio gauge.
- **No crit, no Confused, no status.** Identity §1 esplicita.
- **Atlas clip:** `skill` (frames 45–56, count **12**) — sweep cinematico, match signature DS

## §2 — FSM topology

3-nodo: `Channel → Storm → Recovery`. Frame budget 12 = 4+5+3.

```
commit → Channel(4f) → Storm(5f) → Recovery(3f) → exit
                         │
                         │ on_enter: (G6 path A — expand by blueprint)
                         │   for enemy in enemies_alive:
                         │     EmitDamage { hits:1, mul_param:"skill_mul", target:Single(enemy) }
                         │   AdvanceTurn { actor:Self, pct_param:"self_advance_pct" }
                         │   // VFX layer 1 — volumetric rain (canon "diamond shards rain down")
                         │   SpawnParticle("diamond_shards_rain", origin: WorldGrid(enemy_team_center), motion: Static)
                         │   // VFX layer 2 — per-enemy impact flash (readability: chi è stato colpito)
                         │   for enemy_i in enemies_alive:
                         │     SpawnParticle("diamond_shard_impact",
                         │                   origin: EntityCenter(<iter:enemy_i>), motion: Static)
                         │   Shake { intensity:2, duration_ms:120 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Channel` | 4 | 45–48 | `SpawnParticle("crystal_charge", origin: SelfCenter, motion: Static)` (Renamon raises hands, shards form) |
| `Storm` | 5 | 49–53 | N × EmitDamage + AdvanceTurn + diamond rain (volumetric) + N × diamond_shard_impact (per-enemy) + shake |
| `Recovery` | 3 | 54–56 | — |

Frame budget: 12 = atlas. ✅

## §4 — Kernel events expected

```
Storm.on_enter
  for each enemy_i alive:
    └─ DamageDealt(enemy_i, ≈14, Holy)
  └─ TurnGaugeShifted { actor:Renamon, delta_pct: -25 }   ← convenzione: advance = riduzione gauge
```

## §5 — Open questions (nuovi)

1. **T1 — `AdvanceTurn`/`DelayTurn` verbo nuovo.** Vocabolario §2.2b §C non lo include. Schema:
   - `AdvanceTurn { actor: ActorRef, pct_param: string }` — sottrae pct dal gauge.
   - `DelayTurn { target_shape: TargetShape, pct_param: string }` — aggiunge pct.
   - Headless: gameplay command (NON drop, deterministic).
   - Cap ±50% per chiamata, clamp gauge `[0, 200]`.
2. **T2 — Race con `TurnOrder`.** L'effect deve essere atomico dopo `resolve_action_system`. Mid-FSM è ammesso? Identity §5 dice "modifiche atomiche dopo resolution; nessuna reorder mid-action". **Decisione:** il kernel applica `AdvanceTurn` solo dopo che la FSM corrente exit (queue interno fino a `Recovery.exit`).
3. **T3 — AdvanceTurn(self) → Renamon agisce due volte di fila?** Possibile se la mossa è abbastanza. Cap 25% non basta per skip totale ma combina con `kitsune_grace` (10%) → 35% chunk. Bilanciato a playtest.
4. **T4 — AoE damage scaling.** No-falloff (full damage a ogni nemico). Identity §4 non specifica falloff; mantieni piatto.

## §6 — Verdetto

Diamond Storm introduce **2 verbi nuovi** (`AdvanceTurn`, `DelayTurn` come coppia). Sono **gameplay-critical**, non cosmetici. Vocabolario §2.2b va esteso prima di M017+turn-manip features.

Conferma G6 path A (espansione blueprint AoE).

**Rev2 swap (2026-05-12):** atlas `skill` 45–56 (12f) sostituisce `heavy_attack` 20–29 (10f). Cinema cinematico DS canon + frame budget 12f permette `Channel` espanso (4f vs 3f) → telegraph leggibile prima del rain. Effetto meccanico invariato.
