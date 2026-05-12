# Dorumon — Basic: `bite`

> **Goal**: baseline Dark single-target. Niente status, niente edge — pure damage compresso (Dorumon è snappy, frame budget 28 totale).
>
> **Gap §2.2b condivisi:** params G1, ordering G4, ult charge G11. Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult
- **Effect:** Damage Dark `≈9` su single primary; **no status**.
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Lunge → Bite → Recovery`. Più aggressivo dei lunchi (Dorumon "executor veloce" — anticipation breve).

```
commit → Lunge(2f) → Bite(4f) → Recovery(3f) → exit
                      │
                      │ on_enter:
                      │   EmitDamage { hits:1, mul_param:"basic_mul" }
                      │   SpawnParticle("fang_glint",  origin: SelfCenter,            motion: Static)
                      │   SpawnParticle("bite_impact", origin: EntityCenter(Primary), motion: Static)  # NEW — impact flash su target
                      │   Shake { intensity:1, duration_ms:70 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Lunge` | 2 | 0–1 | `SpawnParticle("dash_dust", origin: SelfCenter, motion: Static)` |
| `Bite` | 4 | 2–5 | `EmitDamage(primary)` + `SpawnParticle("fang_glint", origin: SelfCenter, motion: Static)` + `SpawnParticle("bite_impact", origin: EntityCenter(Primary), motion: Static)` + shake |
| `Recovery` | 3 | 6–8 | — |

Frame budget: 9 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt(target, ≈9, Dark)`
2. `SpEarned(Dorumon, 1)`
3. `UltimateCharged(Dorumon, 25)`

**Predator Loop listener** (esistente): legge `DamageDealt(target, *)` → aggiorna tracking del lowest-HP target. Vedi `04_passive_predator_loop.md`.

## §5 — Open questions (nuovi)

1. **E1 — Tag Dark esiste?** Verificare `DamageTag`. Probabile sì (presente nel design discusso). Confermare.
2. **E2 — Basic potrebbe già armare Predator state?** Identity §5: "Entry: quando un nemico cade sotto HP threshold". Il basic damage può portare a HP threshold → Predator state armato. **OK come comportamento esistente** (passive listener generico, non basic-specifico).
3. **E3 — Speed=110 + basic 9f = HSR-style snappy.** Frame budget ridotto (28 totali per kit FSM vs ~37 Tentomon) è coerente con identity §2.

## §6 — Verdetto

Basic minimale. **Nessun gap nuovo.** Mirror di agumon/sharp_claws senza Heated apply.
