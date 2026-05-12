# Gabumon — Ult: `blue_cyclone`

> **Goal**: ult single-target con `OnHit→DR_self`. Caso "self-buff applicato dall'on_enter dello stesso nodo che emette damage". Stress test ordering Commands con side-effect self.
>
> **Naming canon:** rinominato da `arctic_torrent` → `blue_cyclone` per match canon (dataset skill id 76 — "Spins around while spitting out blue fire from its mouth"). Reflavor: blue=cold/ice (designer fiction su elemento, name canon). Anim 14f rotation = match perfetto. Effetti **invariati**.
>
> **Gap §2.2b condivisi:** params (G1), event-payload (G5), pre-damage hook (G9), order (G4). Qui solo nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP, consuma full ult bar (`ultimate_trigger=100`). Anytime off-turn.
- **Effect:** Damage Ice massivo `~55` su primary; **Slowed 2 turni**; **+1 SP team** (cap-aware); **`OnHit→DR 30% self 1 turno`** (auto-buff)
- **Atlas clip:** `skill` (frames 50–63, count 14)

## §2 — FSM topology

3-nodo (no QTE per ora): `Charge → Release → Recovery`.

```
commit → Charge(4f) → Release(5f) → Recovery(5f) → exit
                        │
                        │ on_enter:
                        │   EmitDamage { hits:1, mul_param:"ult_mul", tough_break:25 }
                        │   EmitStatus { id:"slowed", dur_param:"slowed_dur", target:Primary }
                        │   EmitSpGrant { amount:1, target:Team }
                        │   ApplyBuff { id:"dr_self", target: Self_, kind: DR,
                        │               value_param:"ult_dr",
                        │               dur_param:"ult_dr_dur" }  ← S2 (vedi §5)
                        │   SpawnParticle("blue_cyclone_spit",
                        │     origin: SelfCenter,
                        │     motion: Travel { to: EntityCenter(Primary),
                        │                       ease: EaseIn, ms: 180 })
                        │   SpawnParticle("blue_cyclone_impact",
                        │     origin: EntityCenter(Primary), motion: Static)
                        │   Shake { intensity:4, duration_ms:200 }
```

**Canon-fidelity:** `Charge` mantiene `blue_spin_aura` su `SelfCenter` (vortex sul caster prima dello sputo); `Release` proietta il fuoco blu **dal caster verso il primary** con `Travel` (EaseIn = accelerazione consistente con uno sputo), seguito da impact statico sull'arrival. Riflette la canon "spits blue fire from mouth toward target" invece di materializzare la fiamma direttamente sul nemico.

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Charge` | 4 | 50–53 | `SpawnParticle("blue_spin_aura", origin: SelfCenter, motion: Static)` |
| `Release` | 5 | 54–58 | damage + slowed + sp_grant + self_buff + `blue_cyclone_spit` (Travel `SelfCenter`→`EntityCenter(Primary)`, EaseIn 180ms) + `blue_cyclone_impact` (Static su Primary) + shake |
| `Recovery` | 5 | 59–63 | — |

Frame budget: 14 = atlas. ✅

## §4 — Kernel events expected

```
Release.on_enter
  ├─ DamageDealt(primary, ≈55, Ice)
  ├─ ToughnessReduced(primary, 25)
  ├─ StatusApplied(primary, Slowed, dur:2)
  ├─ SpEarned(team_actor, +1)  × N (cap-aware)
  └─ BuffApplied(Gabumon, dr_self, dur:1)
```

## §5 — Open questions (nuovi)

1. **S1 — `EmitSpGrant` formalizzato.** ✅ **Chiuso (round-3, 2026-05-12):** verbo promosso a **kernel-known** in `02-02b §C2` come `EmitSpGrant { amount_param, target }` → `KernelEffect::GrantSp`. **Cap-aware sul lato ricevente** (`SpPool.add` clamp al cap, vedi `src/combat/sp.rs`); **non** passa dal `RoundSpTracker.max_non_basic_per_round` (grant ≠ spend, allineato a tentomon/00 §7 D1). Roster source: Gabumon ult + Tentomon ult; override `+2 SP` Tentomon basic resta data-side via `units.ron.sp_gen_per_basic` (tentomon/01 B1 decisione A).
2. **S2 — Buff applicato a self ≠ debuff su nemico.** ✅ **Chiuso (round-3, 2026-05-12):** **no verbo separato**. `ApplySelfBuff` è alias di `ApplyBuff { target: Self_, kind: DR|Aura|Haste|... }` — vocabolario unico `ApplyBuff` con `target ∈ { Self_, Ally(idx), Team, Primary, ... }` e `kind` tipizzato per stacking rules. Vedi `02-02b §C2` (kernel-known verbs) + `02-08 §H` (status/buff/DR taxonomy). Buff e debuff restano distinguibili dal `kind` (Buff=DR/Aura/Haste/…, Debuff=Status), non dal verbo.
3. **DR stack con `fur_cloak`.** ✅ **Chiuso (round-3, 2026-05-12):** **A (max-replace, intra-unit)** — allineato a `02-08 §H.3` ("intra-unit replace-max" per stacking di buff dello stesso `kind:DR` sulla stessa entity). Esempio: `fur_cloak` (DR 20%, dur 1) + ult (DR 30%, dur 1) → buff finale `dr_self` con `value=0.30, dur=1`. Cross-unit (Patamon `holy_aegis` 10%) è invece **additivo con clamp 0.5** (`02-08 §H.3`).
4. **Ult charge da Skill/Basic non implicito.** Vedi G11. Se `OnAnyAttack` non è confermato, l'Ult Gabumon arriva solo via basic spam. **Game-design**, non FSM.

## §6 — Verdetto

Ult Gabumon ha esposto **2 gap verbi** in round-1, entrambi ✅ chiusi round-3 (2026-05-12): **S1** (`EmitSpGrant`) promosso a kernel-known in `02-02b §C2` (cap-aware ricevente, fuori `RoundSpTracker`); **S2** alias di `ApplyBuff { target: Self_, kind }` (vocabolario unico `ApplyBuff`). DR stacking ✅ chiuso (intra-unit max-replace, `02-08 §H.3`).
