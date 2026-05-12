# Patamon — Basic: `boom_bubble`

> **Goal**: baseline Holy damage low, slot basic per non-skippare turno. Stress test minimo.
>
> **Gap §2.2b condivisi:** vedi `agumon/01-04` (params G1, ordering G4, ult charge G11). Qui solo nuovi.

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult
- **Effect:** Damage Holy `≈6` (basso intenzionale) su single primary; **no status applicato**
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Inhale → Pop → Recovery`.

```
commit → Inhale(2f) → Pop(4f) → Recovery(3f) → exit
                       │
                       │ on_enter:
                       │   EmitDamage { hits:1, mul_param:"basic_mul" }
                       │   SpawnParticle("bubble_pop","mouth")
                       │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Inhale` | 2 | 0–1 | `SpawnParticle("bubble_charge","mouth")` |
| `Pop` | 4 | 2–5 | damage + particle + shake |
| `Recovery` | 3 | 6–8 | — |

Frame budget: 9 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt { target, amount, tag: Holy, caster: Patamon }`
2. `SpEarned { actor: Patamon, amount: 1 }`
3. `UltimateCharged { actor: Patamon, amount: 25 }`

**No status.** Listener `holy_aegis` non si arma su damage (è passive sempre-on, non reattivo).

## §5 — Open questions (nuovi)

1. **Damage tag `Holy` impatta `Blessed` (Renamon)?** Identity §6 dice no — Blessed è buff alleato, separato dal damage tag. Conferma esplicita: il tag Holy non chiama altri listener cross-roster oltre weakness check standard.
2. **Patamon weakness `Dark`** → Dorumon all-in vs Patamon. Game-design issue, non FSM.
3. **Ult charge `+25` su basic con damage così basso (≈6) — ratio damage/charge sproporzionato.** Pattern intenzionale (heal/cleanse vale di più). Confermare in playtest.

## §6 — Verdetto

Basic minimale, no edge reattivo, no status. **Zero gap architetturali nuovi.** Conferma che la FSM regge anche skill "vuote" (solo damage + particle).
