# Patamon — Skill: `patapata_hover` (single-target heal + cleanse)

> **Goal**: primo skill **ally-targeted** del roster. Stress test target shape `SingleAlly`, verbo `EmitHeal`, verbo `EmitCleanse`. Niente damage, niente nemico nel flow.
>
> **Canon:** Patapata Hover / パタパタホバー — DAPI: *"Flies through the sky"*. Atlas `heavy_attack` 14f = jump+fall anim — **canon non-literal** (Patapata Hover canon = sustained flight). **Reflavor heal:** jump-apex-descent = "blessing arc". VFX gold halo at apex + descent blessing trail rende heal-cast leggibile. Stretch giustificato: canon Patapata Hover è utility/flight non-damage → naturale evoluzione reflavor verso heal aura (canon-adjacent).
>
> **Gap §2.2b condivisi:** params G1, ordering G4. Qui nuovi.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult (`OnAnyAttack` o equivalente — vedi G11)
- **Effect:** Heal `~25% HP max` su single ally; **Cleanse 1 debuff** (FIFO oldest, tie-break ID alfabetico)
- **Self-target ammesso?** Sì (Patamon può curare sé stesso). Identity §7 lo conferma.
- **Atlas clip:** `heavy_attack` (frames 30–43, count 14) — jump+fall anim, reflavor blessing arc

## §2 — FSM topology

4-nodo: `Ascend → Apex → Descend → Recovery`. Frame budget 14 = 5+3+4+2.

```
commit → Ascend(5f) → Apex(3f) → Descend(4f) → Recovery(2f) → exit
                        │
                        │ Apex.on_enter (peak ascent — blessing absorbed):
                        │   EmitHeal { amount_param:"heal_pct_max", target:SingleAlly }
                        │   EmitCleanse { count:1, target:SingleAlly,
                        │                 priority:"oldest_first", filter:"debuff_only" }
                        │   // VFX layer 1 — apex holy halo (peak frame, telegraph "blessing absorbed")
                        │   SpawnParticle("apex_holy_halo",
                        │                 origin: SelfCenter, motion: Static)
                        │   // VFX layer 2 — descent blessing trail (Travel da apex a target)
                        │   SpawnParticle("descent_blessing_trail",
                        │                 origin: SelfCenter,
                        │                 motion: Travel { to: EntityCenter(SelectedAlly),
                        │                                  ease: EaseOut, ms: 250 })
                        │   // VFX layer 3 — heal burst su target ally (sync arrival)
                        │   SpawnParticle("holy_heal_burst",
                        │                 origin: EntityCenter(SelectedAlly),
                        │                 motion: RiseUp { ms: 600 })
                        │   Shake { intensity:1, duration_ms:60 }   ← ally shake
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Ascend` | 5 | 30–34 | `SpawnParticle("ascent_wind", origin: SelfCenter, motion: RiseUp { ms:400 })` (Patamon salta su) |
| `Apex` | 3 | 35–37 | heal + cleanse + halo + trail + burst (peak frame) |
| `Descend` | 4 | 38–41 | `SpawnParticle("light_feathers", origin: SelfCenter, motion: Static)` (caduta con piume Holy) |
| `Recovery` | 2 | 42–43 | — |

Frame budget: 14 = atlas. ✅

**VFX layering critico:** il significato heal viene venduto da 3 layer congiunti: (1) `apex_holy_halo` su Patamon al peak = "blessing channeled"; (2) `descent_blessing_trail` Travel da Patamon → target ally = "blessing transferred"; (3) `holy_heal_burst` su ally al sync arrival = "heal landed". Anim base jump+fall (canon-loose) diventa heal-cast leggibile.

## §4 — Kernel events expected

```
Apex.on_enter
  ├─ HealApplied { target:ally, amount, source:Patamon }
  └─ CleanseApplied { target:ally, removed:[debuff_id], count:1 }   (no-op se ally non ha debuff)

Listener side: nessuno (no Twin Core, no battery)
```

## §5 — Open questions

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
6. **P6 — Rev2 anim-canon stretch.** Atlas `heavy_attack` jump+fall ≠ canon Patapata Hover sustained flight literal. Stretch accettato perché: (a) canon Patapata = non-damage utility (closest a heal-tier in Patamon move pool); (b) VFX layering vendono significato heal a prescindere dall'anim sub-pattern; (c) precedent Renamon `tohakken` (claw → AoE via VFX). Riaprire se PixelLab atlas accetta variante hover-loop.

## §6 — Verdetto

Patapata Hover (ex `holy_breeze`) introduce **3 verbi nuovi** (EmitHeal, EmitCleanse, TargetShape::SingleAlly). Pattern cleanse + heal è universale per healer; vale la pena formalizzare ora il vocabolario.

**Tutti i 3 verbi sono "gameplay" (no presentation-only).** Headless deve eseguirli — niente drop §G.

**Rev2 rename (2026-05-12):** `holy_breeze` (invented) → `patapata_hover` (canon). Mechanic invariato. VFX rifirmato per leggere anim jump+fall come blessing arc (apex halo + descent trail + ally burst).
