# Gabumon — Basic: `claw_attack`

> **Goal**: baseline FSM Ice, mirror di `agumon/sharp_claws`. Stress test minimo: la differenza è il tag (Ice) e lo status (Chilled).
>
> **Naming canon (v2):** rinominato `horn_strike` → `horn_attack` → **`claw_attack`** (dataset skill id 77 — "Attacks with its claws"). Reason: atlas clip `attack` mostra claw motion, non horn → match canon-anim coerente. Effetti **invariati** — solo ID + anchor change.
>
> **Gap §2.2b condivisi:** vedi `agumon/01-04` (params plumbing G1, source kind G5, ordering G4, ult charge G11). Qui solo gap nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult (`OnBasicAttack`)
- **Effect:** Damage Ice `≈7` su single primary; **+1 Chilled stack** (status, target-scoped)
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Windup → Strike → Recovery → exit`. Stesso shape di `sharp_claws`.

```
   commit → Windup(2f) → Strike(4f) → Recovery(3f) → exit
                        on_enter:
                          EmitDamage { hits:1, mul_param:"basic_mul" }
                          EmitStatus { id:"chilled", dur_param:"chilled_dur",
                                       chance_param:"chilled_chance", target:Primary }
                          SpawnParticle { name:"ice_claw_burst",  origin: SelfCenter,            motion: Static }   // weapon-side flash
                          SpawnParticle { name:"ice_chill_impact", origin: EntityCenter(Primary), motion: Static }  // NEW — impact on target
                          Shake { intensity:1, duration_ms:80 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter (Commands) |
|---|---|---|---|
| `Windup` | 2 | 0–1 | `Shake { intensity:1, duration_ms:60 }` |
| `Strike` | 4 | 2–5 | `EmitDamage`, `EmitStatus(Chilled)`, `SpawnParticle("ice_claw_burst", SelfCenter, Static)` (weapon-side flash), `SpawnParticle("ice_chill_impact", EntityCenter(Primary), Static)` (impact on target), `Shake` |
| `Recovery` | 3 | 6–8 | — |

Frame budget: 9 = atlas. No stretch.

## §4 — Kernel events expected

1. `DamageDealt { target, amount, tag: Ice, caster: Gabumon }`
2. `StatusApplied { target, status: Chilled, stacks: 1 }`
3. `SpEarned { actor: Gabumon, amount: 1 }`
4. `UltimateCharged { actor: Gabumon, amount: 25 }`

**Twin Core hook:** `StatusApplied(Chilled, caster:Gabumon)` viene letto dall'Agumon listener (`twin_core_fire`) → arma il buff fire-side. Vedi `agumon/04`.

## §5 — Open questions (nuovi)

1. **Chilled cap.** Proposta 6 stacks (mirror Heated). Conferma cap globale o per-source.
2. **`fur_cloak` triggers su basic?** Passive `fur_cloak` arma DR 20% self quando Gabumon `EmitStatus(Chilled)`. Il basic applica Chilled → DR self attivo già al primo basic? Coerente con identity §1 (DR-self on apply) ma costoso a regime. **Proposta:** trigger su Skill+Ult solo, basic no. Vedi `04_passive_fur_cloak.md`.
3. ~~**Animation anchor `"claws"`** è una stringa libera~~ **RISOLTO 2026-05-12 via §2.2d**: anchor body-part collassa a `origin: SelfCenter, motion: Static`. Flavor mantenuto via particle preset name. Vedi `02-02d_vfx_positioning.md`.

## §6 — Verdetto

Mirror pulito di sharp_claws. **Nessun gap nuovo architetturale.** L'unico dubbio è game-design (passive trigger).
