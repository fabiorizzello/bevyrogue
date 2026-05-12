# Patamon — Basic: `tai_atari` — *"Body Blow"*

> **Goal**: baseline Holy damage low, slot basic per non-skippare turno. Stress test minimo.
>
> **Canon:** Tai Atari / 体当たり / "Body Blow" — DAPI: *"Rushes at the enemy and body slams it"*. Atlas `attack` 9f = roll/headbutt anim = **canon literal match** (body-slam rush). Zero stretch.
>
> **Gap §2.2b condivisi:** vedi `agumon/01-04` (params G1, ordering G4, ult charge G11). Qui solo nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult
- **Effect:** Damage Holy `≈6` (basso intenzionale) su single primary; **no status applicato**
- **Atlas clip:** `attack` (frames 0–8, count 9) — roll/headbutt rush

## §2 — FSM topology

3-nodo: `Charge → Slam → Recovery`. Frame budget 9 = 2+4+3.

```
commit → Charge(2f) → Slam(4f) → Recovery(3f) → exit
                       │
                       │ on_enter:
                       │   EmitDamage { hits:1, mul_param:"basic_mul" }
                       │   SpawnParticle("holy_impact", origin: EntityCenter(Primary), motion: Static)
                       │   Shake { intensity:1, duration_ms:60 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Charge` | 2 | 0–1 | `SpawnParticle("rush_blur", origin: SelfCenter, motion: Static)` (Patamon prepares roll) |
| `Slam` | 4 | 2–5 | damage + `SpawnParticle("holy_impact", origin: EntityCenter(Primary), motion: Static)` + shake (headbutt collision) |
| `Recovery` | 3 | 6–8 | — |

Frame budget: 9 = atlas. ✅

## §4 — Kernel events expected

1. `DamageDealt { target, amount, tag: Holy, caster: Patamon }`
2. `SpEarned { actor: Patamon, amount: 1 }`
3. `UltimateCharged { actor: Patamon, amount: 25 }`

**No status.** Listener `holy_aegis` non si arma su damage (è passive sempre-on, non reattivo).

## §5 — Open questions

1. **Damage tag `Holy` impatta `Blessed` (Renamon)?** Identity §6 dice no — Blessed è buff alleato, separato dal damage tag. Conferma esplicita: il tag Holy non chiama altri listener cross-roster oltre weakness check standard.
2. **Patamon weakness `Dark`** → Dorumon all-in vs Patamon. Game-design issue, non FSM.
3. **Ult charge `+25` su basic con damage così basso (≈6) — ratio damage/charge sproporzionato.** Pattern intenzionale (heal/cleanse + ult dual-axis valgono di più). Confermare in playtest.
4. **Rev2 canon shift.** Ex `boom_bubble` ora è `sparking_air_shot` (ult). Atlas `attack` = roll/headbutt → canon Tai Atari literal. Boom Bubble anim atlas `skill` clip cinematic → ult slot. Anim-canon match più forte di rev1.

## §6 — Verdetto

Basic minimale, no edge reattivo, no status. **Zero gap architetturali nuovi.** Conferma che la FSM regge anche skill "vuote" (solo damage + particle). Rename `boom_bubble → tai_atari` (canon Body Blow, anim roll/headbutt literal).
