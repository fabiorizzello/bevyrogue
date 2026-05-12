# Gabumon — Skill: `bubble_blast`

> **Goal**: validare modifier reattivo `OnStatusApplied→Echo(Chilled)` su adj lowest-HP. Primo caso d'uso di **edge reattivo che emette su un terzo bersaglio** (non primary, non self).
>
> **Gap §2.2b condivisi:** params plumbing (G1), tough_break (G2), stacks_param (G3), order (G4). Qui gap nuovi.

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult (se confermato `OnAnyAttack`, vedi G11)
- **Effect:** Damage Ice `≈16` su primary; **+2 Chilled**; **ToughnessHit(8)**; **Modifier `OnStatusApplied(Chilled)→Echo`**: +1 Chilled sull'adj lowest-HP%
- **Self side-effect:** `fur_cloak` listener arma DR 20% self 1 turno (post-skill)
- **Atlas clip:** `heavy_attack` (frames 27–37, count 11)

## §2 — FSM topology

4-nodo: `Inhale → Hold → Burst → Recovery` con edge reattivo su `Burst`.

```
commit → Inhale(2f) → Hold(2f) → Burst(4f) → Recovery(3f) → exit
                                    │
                                    │ on_enter:
                                    │   EmitDamage { hits:1, mul_param:"skill_mul", tough_break:8 }
                                    │   EmitStatus { id:"chilled", stacks_param:"chilled_skill_stacks",
                                    │                target:Primary }
                                    │   SpawnParticle("ice_bubble_burst","mouth")
                                    │   Shake { intensity:2, duration_ms:100 }
                                    │
                                    └── edge A: KernelEvent(StatusApplied{Chilled, caster:self}) prio:10
                                                ──▶ Echo(2f)
                                                    on_enter:
                                                      EmitStatus { id:"chilled", stacks_param:"echo_stacks",
                                                                   target:AdjLowestHpPct }
                                                      SpawnParticle("ice_echo","adj_pivot")
                                                    fallback TimeInNode → Recovery
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Inhale` | 2 | 27–28 | `SpawnParticle("frost_charge","mouth")` |
| `Hold` | 2 | 29–30 | — |
| `Burst` | 4 | 31–34 | damage + status + tough + particle + shake |
| `Echo` | 2 | 35–36 | (solo se edge A taken) emit Chilled adj lowest-HP |
| `Recovery` | 3 | 35–37 / 37 | atlas reuse se no-Echo (3 frame), se Echo: solo frame 37 (1f stretch) |

Frame budget: con Echo 2+2+4+2+1 = 11 = atlas. Senza Echo: 2+2+4+3 = 11. Match in entrambi i path. ✅

## §4 — Kernel events expected

```
Burst.on_enter
  ├─ DamageDealt(primary, ≈16, Ice)
  ├─ ToughnessReduced(primary, 8)  → possibile Broken
  └─ StatusApplied(primary, Chilled, +2)   ← trigger edge A

edge A → Echo.on_enter
  └─ StatusApplied(adj_lowest_hp, Chilled, +1)

Listener side (kernel-side, non FSM):
  └─ fur_cloak: applica BuffSelf(DR20, dur:1) su Gabumon
  └─ twin_core_fire (Agumon): arma TwinCoreActive se in team
```

## §5 — Open questions (nuovi)

1. **AdjLowestHpPct target shape.** `EmitDamage`/`EmitStatus` `target` non ha questa modalità in §2.2b §C. Estensione richiesta:
   - `TargetShape::AdjLowest { metric: HpPct | Hp | Raw }` (sintassi proposta)
   - Tie-break deterministico: tie su HP% → slot index ascending.
   - **Action item §2.2b:** aggiungere target-shape al vocabolario o spostarlo nel param-resolver (blueprint risolve `target_ref:"adj_lowest_hp_pct"` al commit per Echo).
2. **Chain echo prevention.** Echo emette `StatusApplied(Chilled)` di nuovo → l'edge A matchassa di nuovo? **Filter necessario:** edge A predicate include `caster_node == "Burst"` (non `Echo`). Oppure: edge A flag `once_per_skill: true`.
3. **DR `fur_cloak` listener path.** Listener applica buff su Gabumon **post** `StatusApplied`. Race: la skill è ancora in `Burst`. Il buff è attivo già durante la stessa FSM o solo al turno successivo? **Proposta:** buff applicato immediatamente, ma `expires_on: NextOwnerTurnEnd` → effetto pratico = mitigation nel turno successivo (quando subirà il colpo nemico).

## §6 — Verdetto

Bubble Blast è il **primo edge reattivo a 3° bersaglio** del roster. Espone 1 gap nuovo (target-shape `AdjLowestX`) **non emerso** in agumon stress. Risolvibile con estensione param-resolver o con `TargetShape` esteso. Decisione consigliata: estendere `TargetShape` per coerenza dichiarativa.
