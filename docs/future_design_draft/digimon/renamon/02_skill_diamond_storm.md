# Renamon — Skill: `diamond_storm` (AoE Holy + self AdvanceTurn)

> **Goal**: stress test **AoE enemies** + **time-manip command** (`AdvanceTurn`). Primo skill che tocca `TurnOrder` come effect.
>
> **Gap §2.2b condivisi:** params G1, multi-target G6, ordering G4. Qui nuovi.

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult
- **Effect:** Damage medio Holy `≈14` su **tutti i nemici vivi** (AoE); **`AdvanceTurn(self, 25%)`** — Renamon avanza il proprio gauge.
- **No crit, no Confused, no status.** Identity §1 esplicita.
- **Atlas clip:** `heavy_attack` (frames 20–29, count 10)

## §2 — FSM topology

3-nodo: `Spin → Storm → Recovery`.

```
commit → Spin(3f) → Storm(4f) → Recovery(3f) → exit
                      │
                      │ on_enter: (G6 path A — expand by blueprint)
                      │   for enemy in enemies_alive:
                      │     EmitDamage { hits:1, mul_param:"skill_mul", target:Single(enemy) }
                      │   AdvanceTurn { actor:Self, pct_param:"self_advance_pct" }
                      │   SpawnParticle("crystal_burst","center_pivot")
                      │   Shake { intensity:2, duration_ms:100 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Spin` | 3 | 20–22 | `SpawnParticle("crystal_charge","tail")` |
| `Storm` | 4 | 23–26 | N × EmitDamage + AdvanceTurn + particle + shake |
| `Recovery` | 3 | 27–29 | — |

Frame budget: 10 = atlas. ✅

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
