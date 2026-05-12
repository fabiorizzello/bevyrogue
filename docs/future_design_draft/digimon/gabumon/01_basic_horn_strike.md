# Gabumon — Basic: `horn_strike`

> **Goal**: baseline FSM Ice, mirror di `agumon/claw_strike`. Stress test minimo: la differenza è il tag (Ice) e lo status (Chilled).
>
> **Gap §2.2b condivisi:** vedi `agumon/01-04` (params plumbing G1, source kind G5, ordering G4, ult charge G11). Qui solo gap nuovi.

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult (`OnBasicAttack`)
- **Effect:** Damage Ice `≈7` su single primary; **+1 Chilled stack** (status, target-scoped)
- **Atlas clip:** `attack` (frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Windup → Strike → Recovery → exit`. Stesso shape di `claw_strike`.

```
   commit → Windup(2f) → Strike(4f) → Recovery(3f) → exit
                        on_enter:
                          EmitDamage { hits:1, mul_param:"basic_mul" }
                          EmitStatus { id:"chilled", dur_param:"chilled_dur",
                                       chance_param:"chilled_chance", target:Primary }
                          SpawnParticle { name:"ice_horn_burst", anchor:"horn" }
                          Shake { intensity:1, duration_ms:80 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter (Commands) |
|---|---|---|---|
| `Windup` | 2 | 0–1 | `Shake { intensity:1, duration_ms:60 }` |
| `Strike` | 4 | 2–5 | `EmitDamage`, `EmitStatus(Chilled)`, `SpawnParticle("ice_horn_burst")`, `Shake` |
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
3. **Animation anchor `"horn"`** è una stringa libera (vedi gap §2.2b §4 di agumon/02). Confermare contratto presentation.

## §6 — Verdetto

Mirror pulito di claw_strike. **Nessun gap nuovo architetturale.** L'unico dubbio è game-design (passive trigger).
