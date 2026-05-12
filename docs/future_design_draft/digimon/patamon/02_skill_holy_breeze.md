# Patamon — Skill: `holy_breeze` (single-target heal + cleanse)

> **Goal**: primo skill **ally-targeted** del roster. Stress test target shape `SingleAlly`, verbo `EmitHeal`, verbo `EmitCleanse`. Niente damage, niente nemico nel flow.
>
> **Gap §2.2b condivisi:** params G1, ordering G4. Qui nuovi.

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult (`OnAnyAttack` o equivalente — vedi G11)
- **Effect:** Heal `~25% HP max` su single ally; **Cleanse 1 debuff** (FIFO oldest, tie-break ID alfabetico)
- **Self-target ammesso?** Sì (Patamon può curare sé stesso). Identity §7 lo conferma.
- **Atlas clip:** `heavy_attack` (frames 30–43, count 14)

## §2 — FSM topology

4-nodo: `Gather → Cast → Bless → Recovery`.

```
commit → Gather(3f) → Cast(3f) → Bless(4f) → Recovery(4f) → exit
                                   │
                                   │ on_enter:
                                   │   EmitHeal { amount_param:"heal_pct_max", target:SingleAlly }
                                   │   EmitCleanse { count:1, target:SingleAlly,
                                   │                 priority:"oldest_first", filter:"debuff_only" }
                                   │   SpawnParticle("holy_aura","target_ally_pivot")
                                   │   Shake { intensity:1, duration_ms:60 }   ← ally shake
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Gather` | 3 | 30–32 | `SpawnParticle("holy_gather","wings")` |
| `Cast` | 3 | 33–35 | `SpawnParticle("holy_beam","mouth")` |
| `Bless` | 4 | 36–39 | heal + cleanse + particle |
| `Recovery` | 4 | 40–43 | — |

Frame budget: 14 = atlas. ✅

## §4 — Kernel events expected

```
Bless.on_enter
  ├─ HealApplied { target:ally, amount, source:Patamon }
  └─ CleanseApplied { target:ally, removed:[debuff_id], count:1 }   (no-op se ally non ha debuff)

Listener side: nessuno (no Twin Core, no battery)
```

## §5 — Open questions (nuovi)

1. **P1 — `EmitHeal` non nel vocabolario §2.2b §C.** Verbo nuovo richiesto.
   - Schema: `EmitHeal { amount_param: string, target: TargetShape, kind: PctMax | Flat }`.
   - Headless: applica direttamente (gameplay command, non cosmetic).
2. **P2 — `EmitCleanse` non nel vocabolario.** Schema:
   - `EmitCleanse { count: u8, target: TargetShape, priority: OldestFirst | NewestFirst | RandomSeeded, filter: DebuffOnly | All }`
   - **Filter strict:** mai rimuove buff alleati (vedi `Blessed` Renamon, identity §6).
3. **P3 — `target: SingleAlly` shape.** Non esiste in §2.2b §C (oggi: Single/Primary, Adjacent, AoE). Estensione:
   - `TargetShape::SingleAlly { selector: Manual | LowestHpPct | ... }`
   - Per M017: `Manual` (user picker) baseline. Auto-target (lowest-HP%) come unlock futuro.
4. **P4 — Heal su unit già full HP.** No-op? Spillover? **Proposta:** no-op (HP capped a max). Niente overheal stack. Cleanse fired indipendentemente (non gated da heal landing).
5. **P5 — Cleanse su unit senza debuff.** No-op silenzioso. Niente refund SP (decision: cost paid even on no-op cleanse, evita stalling con cleanse-spam).

## §6 — Verdetto

Holy Breeze introduce **3 verbi nuovi** (EmitHeal, EmitCleanse, TargetShape::SingleAlly). Pattern cleanse + heal è universale per healer; vale la pena formalizzare ora il vocabolario.

**Tutti i 3 verbi sono "gameplay" (no presentation-only).** Headless deve eseguirli — niente drop §G.
