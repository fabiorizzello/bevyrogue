# Gabumon — Ult: `arctic_torrent`

> **Goal**: ult single-target con `OnHit→DR_self`. Caso "self-buff applicato dall'on_enter dello stesso nodo che emette damage". Stress test ordering Commands con side-effect self.
>
> **Gap §2.2b condivisi:** params (G1), event-payload (G5), pre-damage hook (G9), order (G4). Qui solo nuovi.

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
                        │   EmitSpGrant { amount:1, target:Team }   ← gap S1
                        │   ApplySelfBuff { id:"dr_self", value_param:"ult_dr",
                        │                   dur_param:"ult_dr_dur" }  ← gap S2
                        │   SpawnParticle("arctic_geyser","primary_pivot")
                        │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Charge` | 4 | 50–53 | `SpawnParticle("frost_aura","ground")` |
| `Release` | 5 | 54–58 | damage + slowed + sp_grant + self_buff + particle + shake |
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

1. **S1 — `EmitSpGrant` non esiste nel vocabolario §2.2b §C.** Tentomon ult ha lo stesso pattern (vedi `tentomon/03`). **Proposta canon:** nuovo verbo `EmitSpGrant { amount_param, target: Team|Single }` per il roster, cap-aware (rispetta `RoundSpTracker`).
2. **S2 — `ApplySelfBuff` distinto da `EmitStatus`.** Buff alleato ≠ debuff su nemico. Opzioni:
   - **A.** `EmitStatus` generalizzato su `target: SelfOrAlly` con flag `kind: Buff|Debuff`.
   - **B.** Verbo separato `ApplySelfBuff { id, value_param, dur_param }`.
   - **Decisione consigliata:** B. Buff hanno value (mult/percent) mentre debuff hanno stacks. Vocabolari distinti pulisce semantica.
3. **DR stack con `fur_cloak`.** Se `fur_cloak` (DR 20%) è già attivo quando l'Ult applica DR 30%, regola?
   - **A.** Refresh massimo (30% sostituisce 20%, durata aggiornata)
   - **B.** Stack moltiplicativo (1 − 0.2)·(1 − 0.3) = 0.56 → DR effettivo 44%
   - **C.** Stack additivo cap 50% (20+30 = 50, clamp)
   - **Decisione consigliata:** A (max-replace), evita layering complesso al primo pass. Identity sheet §6 lo flag come open.
4. **Ult charge da Skill/Basic non implicito.** Vedi G11. Se `OnAnyAttack` non è confermato, l'Ult Gabumon arriva solo via basic spam. **Game-design**, non FSM.

## §6 — Verdetto

Ult Gabumon introduce **2 verbi mancanti** (S1, S2). Pattern si ripete in Tentomon (SP grant) e Patamon (heal/cleanse). Da risolvere come parte di §2.2b round-2.
