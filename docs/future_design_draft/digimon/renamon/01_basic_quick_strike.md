# Renamon — Basic: `quick_strike`

> **Goal**: baseline Holy single-target. Snappy (Renamon è la più veloce del roster).
>
> **Gap §2.2b condivisi:** params G1, ordering G4, ult charge G11. Qui nuovi.

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult
- **Effect:** Damage Holy `≈8` su single primary; **no status**.
- **Atlas clip:** `attack` (frames 0–9, count 10)

## §2 — FSM topology

3-nodo: `Dash → Strike → Recovery`. Frame budget 10 = 3+4+3.

```
commit → Dash(3f) → Strike(4f) → Recovery(3f) → exit
                      │
                      │ on_enter:
                      │   EmitDamage { hits:1, mul_param:"basic_mul" }
                      │   SpawnParticle("holy_slash","claw")
                      │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Dash` | 3 | 0–2 | `SpawnParticle("dash_blur","feet")` |
| `Strike` | 4 | 3–6 | damage + particle + shake |
| `Recovery` | 3 | 7–9 | — |

Frame budget: 10 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt(target, ≈8, Holy)`
2. `SpEarned(Renamon, 1)`
3. `UltimateCharged(Renamon, 25)`

**Time-manip:** basic **non** triggera `AdvanceTurn`/`DelayTurn`. Solo skill/ult. Conferma identity §4.

## §5 — Open questions (nuovi)

1. **R1 — Speed=115 e basic snappy.** Frame 10 atlas è uguale ad altri basic, no anim più veloce. La "velocità" Renamon è solo nel `speed` stat (turn gauge), non nella FSM. Coerente con identity §3 ("FSM determinismo invariato").
2. **R2 — Blessed buff sull'alleato che usa Ult non applicato qui.** Solo `fox_drive` lo applica (vedi 03). Basic è "vuoto".

## §6 — Verdetto

Basic minimale. **Zero gap nuovi.** Mirror di patamon/01 ma con Holy "offensivo" (damage 8 vs 6).
