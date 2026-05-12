# Renamon — Basic: `kokaishu`

> **Goal**: baseline Holy single-target. Snappy (Renamon è la più veloce del roster).
>
> **Canon:** Kokaishū / 狐回蹴 / "Fox Spin Kick" — DAPI: *"Attacks with multiple roundhouse kicks"*. Atlas `attack` (10f) regge un singolo roundhouse + recovery, non multi-kick. Riduzione fedele.
>
> **Gap §2.2b condivisi:** params G1, ordering G4, ult charge G11. Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult
- **Effect:** Damage Holy `≈8` su single primary; **no status**.
- **Atlas clip:** `attack` (frames 0–9, count 10)

## §2 — FSM topology

3-nodo: `Wind → Kick → Recovery`. Frame budget 10 = 3+4+3.

```
commit → Wind(3f) → Kick(4f) → Recovery(3f) → exit
                      │
                      │ on_enter:
                      │   EmitDamage { hits:1, mul_param:"basic_mul" }
                      │   SpawnParticle("holy_impact_ring", origin: EntityCenter(Primary), motion: Static)
                      │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Wind` | 3 | 0–2 | `SpawnParticle("foot_blur", origin: SelfCenter, motion: Static)` (leg cocked) |
| `Kick` | 4 | 3–6 | damage + holy ring particle on target + shake (roundhouse impact) |
| `Recovery` | 3 | 7–9 | — |

Frame budget: 10 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt(target, ≈8, Holy)`
2. `SpEarned(Renamon, 1)`
3. `UltimateCharged(Renamon, 25)`

**Time-manip:** basic **non** triggera `AdvanceTurn`/`DelayTurn`. Solo skill/ult. Conferma identity §4.

## §5 — Open questions (nuovi)

1. **R1 — Speed=115 e basic snappy.** Frame 10 atlas è uguale ad altri basic, no anim più veloce. La "velocità" Renamon è solo nel `speed` stat (turn gauge), non nella FSM. Coerente con identity §3 ("FSM determinismo invariato").
2. **R2 — Blessed buff sull'alleato che usa Ult non applicato qui.** Solo `tohakken` lo applica (vedi 03). Basic è "vuoto".
3. **R3 — Canon multi-kick reso single.** Kokaishū canon = "multiple roundhouse kicks". Atlas 10f budget = single kick + wind/recovery. Per restare canon-faithful potremmo splittare in 2 kick (Wind=2, Kick1=3, Kick2=3, Recovery=2) — costo: anim leggibilità ridotta a 12fps. **Decisione corrente:** single kick, canon-loose. Riaprire se PixelLab atlas accetta variante 12+f.
4. **R4 — Grammar rename `TargetCenter` → `EntityCenter(Primary)`.** Allineamento §2.2d vocab corrente (`EntityRef::Primary | FromParamSnapshot | EventTarget | Caster | Self`). Semantica invariata su single-target. Solo manutenzione.

## §6 — Verdetto

Basic minimale. **Zero gap nuovi.** Mirror di patamon/01 ma con Holy "offensivo" (damage 8 vs 6). Rename `quick_strike → kokaishu` (canon Fox Spin Kick).
