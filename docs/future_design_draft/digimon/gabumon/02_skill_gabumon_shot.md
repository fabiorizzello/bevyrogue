# Gabumon — Skill: `gabumon_shot`

> **Goal**: validare reactive signature `OnStatusApplied→Echo(Chilled)` su adj lowest-HP. Primo caso d'uso di **edge reattivo che emette su un terzo bersaglio** (non primary, non self).
>
> **Naming canon:** rinominato da `bubble_blast` → `gabumon_shot` per match canon (dataset skill id 72 — "Emits a small blast from the mouth"). Element-neutral, reflavor Ice OK. FSM Inhale→Hold→Burst→Echo→Recovery resta semanticamente coerente con "blast from mouth". Effetti **invariati**.
>
> **Gap §2.2b condivisi:** params plumbing (G1), tough_break (G2), stacks_param (G3), order (G4). Qui gap nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

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
                                    │   SpawnParticle("ice_bubble_travel",
                                    │     origin: SelfCenter,
                                    │     motion: Travel { to: EntityCenter(Primary),
                                    │                       ease: EaseOut, ms: 120 })
                                    │   SpawnParticle("ice_bubble_impact",
                                    │     origin: EntityCenter(Primary), motion: Static)
                                    │   Shake { intensity:2, duration_ms:100 }
                                    │
                                    └── edge A: KernelEvent(StatusApplied{Chilled, caster:self}) prio:10
                                                ──▶ Echo(2f)
                                                    on_enter:
                                                      EmitStatus { id:"chilled", stacks_param:"echo_stacks",
                                                                   target: EntityCenter(FromParamSnapshot("echo_target")) }
                                                      SpawnParticle("ice_echo_link",
                                                        origin: EntityCenter(Primary),
                                                        motion: Travel { to: EntityCenter(FromParamSnapshot("echo_target")),
                                                                          ease: EaseOut, ms: 90 })
                                                      SpawnParticle("ice_echo_impact",
                                                        origin: EntityCenter(FromParamSnapshot("echo_target")),
                                                        motion: Static)
                                                    fallback TimeInNode → Recovery
```

**Snapshot key `echo_target`**: risolto **a edge-commit time** (non a skill-commit), perché dipende dal primary che ha appena ricevuto Chilled (la resolution di `AdjLowestHpPct` filtra adj del primary, non del caster). Vedi §5 C5.

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Inhale` | 2 | 27–28 | `SpawnParticle("frost_charge", origin: SelfCenter, motion: Static)` |
| `Hold` | 2 | 29–30 | — |
| `Burst` | 4 | 31–34 | damage + status + tough + `ice_bubble_travel` (Travel `SelfCenter`→`EntityCenter(Primary)`) + `ice_bubble_impact` (Static su Primary) + shake |
| `Echo` | 2 | 35–36 | (solo se edge A taken) emit Chilled `EntityCenter(FromParamSnapshot("echo_target"))` + `ice_echo_link` (Travel Primary→echo) + `ice_echo_impact` (Static su echo) |
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

1. **AdjLowestHpPct target shape.** ✅ **Chiuso (round-3, 2026-05-12, X17): formalizzato in `02-02b §C3` come `AdjLowest { metric: HpPctMin | HpMin | RawHpMin, side: Side }`.** Blueprint-side resolver (non kernel-side, coerente con `02-02b §C3` regola 1). Tie-break deterministico: tie su HP% → slot index ascending (canonizzato qui, da propagare a §C3 se altre skill ne richiedono varianti).
2. **Chain echo prevention.** Echo emette `StatusApplied(Chilled)` di nuovo → l'edge A matchassa di nuovo? **Filter necessario:** edge A predicate include `caster_node == "Burst"` (non `Echo`). Oppure: edge A flag `once_per_skill: true`.
3. **DR `fur_cloak` listener path.** Listener applica buff su Gabumon **post** `StatusApplied`. Race: la skill è ancora in `Burst`. Il buff è attivo già durante la stessa FSM o solo al turno successivo? **Proposta:** buff applicato immediatamente, ma `expires_on: NextOwnerTurnEnd` → effetto pratico = mitigation nel turno successivo (quando subirà il colpo nemico).
4. **C5-extended — snapshot scope: skill-commit vs edge-commit.** Tentomon `petit_thunder` ha introdotto la convention key `hopN_target` snapshot-once **a skill-commit time** (G5). `gabumon_shot.echo_target` è strutturalmente diverso: la risoluzione `AdjLowestHpPct` dipende dal **primary chilled**, quindi può essere risolta solo **dopo** che il primary ha effettivamente ricevuto `Chilled` — cioè **a edge-commit time**. Due scope distinti:
   - **Skill-commit snapshot** (es. `hopN_target`): tutti i target risolti una sola volta quando la skill viene committata. Stabile, immutabile per tutta la durata FSM.
   - **Edge-commit snapshot** (es. `echo_target`): target risolto al commit dell'edge reattivo che lo richiede; dipende da side-effect prodotti dai nodi precedenti.
   - **Action item §2.2b §G5:** aggiungere distinzione scope al vocabolario param snapshot. Convention naming: lo scope è una proprietà del param entry, non del nome chiave (es. `params.snapshot.echo_target = { scope: "edge:A", resolver: "adj_lowest_hp_pct(primary)" }`).
   - **Travel-on-death policy (§2.2d §H.4):** se `echo_target` muore tra edge-commit e arrival del `ice_echo_link`, il preset continua con snapshot del Transform a edge-commit time (stesso pattern di `hopN_target` in petit_thunder).

## §6 — Verdetto

`gabumon_shot` è il **primo edge reattivo a 3° bersaglio** del roster. Espone 1 gap nuovo (target-shape `AdjLowestX`) **non emerso** in agumon stress. ✅ **Chiuso (X17): `TargetShape` esteso canonizzato in `02-02b §C3` con variante `AdjLowest { metric, side }`** — decisione coerenza dichiarativa adottata.
