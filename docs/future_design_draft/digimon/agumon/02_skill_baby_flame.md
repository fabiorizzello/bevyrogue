# Agumon — Heavy Skill: `baby_flame` (FSM stress test)

> **Goal**: stressare un nodo con multiple emit (damage + status + toughness) e validare che la composition Commands → KernelEffect non perda ordering / scaling.

## §1 — Intent

- **Cost:** **1 SP** (HSR baseline canon, vedi §5b in `00_identity.md`) — **Gen:** +25 Ult charge (`OnBasicAttack` accumulation trigger; conferma: vale anche per Skill o solo per Basic? **Vedi §6.5**)
- **Effect:** Damage Fire `18` su single primary; **+2 Heated stacks**; **ToughnessHit(20)** sul break bar (alzato da 10 — HSR baseline: skill standard ≈ 30 tough; 20 è coerente con 1-SP single-target, baseline `units.ron` `toughness_max=50` ⇒ 2-3 cast per break, niente saturazione meme)
- **Target shape:** **single-target** (no cone, no AoE). Canon "Baby Flame" = small fireball spit dalla bocca su un nemico. In line-combat HSR-style i cone non hanno semantica positional pulita: single bolt è leggibile e canon-fedele.
- **Atlas clip:** `heavy_attack` (source frames 23–36, count 14)

## §2 — FSM topology

4-nodo: `Inhale → Wind → Spit → Recovery`.

```
            ┌──────────┐ TimeInNode(3) ┌──────────┐ TimeInNode(2) ┌──────────┐ TimeInNode(4) ┌────────────┐
   commit →│ Inhale   │ ────────────▶ │  Wind    │ ────────────▶ │  Spit    │ ────────────▶ │ Recovery   │ ──▶ exit
            │frames: 3 │               │frames: 2 │               │frames: 4 │               │ frames: 5  │
            └────┬─────┘               └────┬─────┘               └────┬─────┘               └────────────┘
                 │ on_enter:                │                          │ on_enter:
                 │  SpawnParticle           │                          │  EmitDamage { hits:1, mul_param:"heavy_mul",
                 │   ("inhale_glow",        │                          │              tough_break_param:"tough_break", target:Primary }
                 │    origin:SelfCenter,    │ on_enter:                │  EmitStatus { id:"heated", chance_pct:100,
                 │    motion:Static)        │  SpawnParticle           │                dur_param:"heated_dur",
                 │                          │   ("fire_charge",        │                target:Primary, stacks_param:"heated_skill_stacks" }
                 │                          │    origin:SelfCenter,    │  SpawnParticle("fire_breath_bolt",   origin: SelfCenter,
                 │                          │    motion:Static)        │                                       motion: Travel { to: EntityCenter(Primary), ease: EaseOut, ms: 120 })
                 │                          │                          │  SpawnParticle("fire_breath_impact", origin: EntityCenter(Primary), motion: Static)   ← arrival impact sync
                 │                          │                          │  SpawnParticle("heated_mark",        origin: EntityCenter(Primary), motion: Static)   ← stack pulse (2 stack)
                 │                          │                          │  Shake { intensity:2, duration_ms:120 }
```

> **Nota.** `tough_break_param` è il campo opzionale su `EmitDamage` introdotto in §8 G2 (`tough_break_param: Option<ParamRef>`), dereferenziato a `skills.ron.params["tough_break"]` = `20`. Niente verbo separato.

## §3 — Nodes table

| Node | frames | atlas src range | ms (@12fps ref) | on_enter (Commands) |
|---|---|---|---|---|
| `Inhale` | 3 | 23–25 | 0–250 | `SpawnParticle("inhale_glow", origin: SelfCenter, motion: Static)` |
| `Wind` | 2 | 26–27 | 250–416 | `SpawnParticle("fire_charge", origin: SelfCenter, motion: Static)` |
| `Spit` | 4 | 28–31 | 416–750 | `EmitDamage(heavy, target:Primary, tough_break_param:"tough_break"→20)`, `EmitStatus(Heated, target:Primary, stacks=2)`, `SpawnParticle("fire_breath_bolt", origin: SelfCenter, motion: Travel { to: EntityCenter(Primary), ease: EaseOut, ms: 120 })`, `SpawnParticle("fire_breath_impact", origin: EntityCenter(Primary), motion: Static)`, `SpawnParticle("heated_mark", origin: EntityCenter(Primary), motion: Static)`, `Shake { intensity:2, duration_ms:120 }` |
| `Recovery` | 5 | 32–36 | 750–1166 | — (animazione di ritorno postura) |

**Frame budget:** 14 frames (= atlas clip esatto). No stretch.

## §4 — Command param resolution

| param | source | example |
|---|---|---|
| `heavy_mul` | `skills.ron` numbers + caster ATK + target DEF; current legacy `Damage(amount:18)` | scalar ≈ 18 |
| `heated_dur` | `skills.ron.params["heated_dur"]` | 3 turni (condiviso con basic) |
| `heated_skill_stacks` | `skills.ron.params["heated_skill_stacks"]` | 2 |
| `tough_break` | `skills.ron.params["tough_break"]` (legacy `ToughnessHit(10)` → bumped a 20 per HSR baseline) | 20 |

## §5 — Kernel events expected

Su `Spit.on_enter` → blueprint Agumon emette (in ordine):

1. `KernelEffect::Damage { target, amount: 18+, tag: Fire }` → `CombatEvent::DamageDealt`
2. `KernelEffect::ToughnessHit { target, amount: 20 }` → `CombatEvent::ToughnessReduced` / eventuale `CombatEvent::Broken`
3. `KernelEffect::ApplyStatus { target, status: Heated, stacks: 2, dur: 3 }` → `CombatEvent::StatusApplied`

**Order matters** se il primary va in `Broken` dal `ToughnessHit`: il `StatusApplied(Heated)` arriva post-break. Va bene perché Heated stacka indipendentemente dal break state.

**Listener side-effects:**
- Gabumon listener Twin Core ascolta `StatusApplied(Heated)` (se passive bidirezionale lo richiede). Decisione §K canon: Twin Core fire-side = Agumon legge `StatusApplied(Chilled)` da Gabumon (vedi file 04).
- Tentomon listener (battery) NON triggera su skill, solo basic. Skip.

## §6 — Stress test findings

### ✅ Cosa funziona

- 4 nodi danno windup leggibile (Inhale 250ms → Wind 166ms → Spit 333ms → Recovery 416ms). Total 1166ms = sweet spot per skill non-ult.
- Commands single-frame su `Spit` sono 4 (damage, status, particle, shake) — gestibile in un `on_enter` batch.

### ⚠️ Contraddizioni / gap

1. **`tough_break` come command vs payload.** §2.2b §C vocabolario lista solo `EmitDamage { hits, mul_param, status?, chance_pct?, dur? }`. **Manca toughness break.** Opzioni:
   - **A.** Estendere `EmitDamage` con `tough_break: u32` (✅ preferito, mantiene vocabolario chiuso piccolo)
   - **B.** Aggiungere verbo `EmitBreak { amount }` (cresce vocabolario, ma più ortogonale)
   - **C.** Status `Broken` come `EmitStatus(Broken, threshold=10)` (rompe modello — break è risorsa, non status)
   - **Decisione consigliata:** A.

2. **`EmitStatus` plurale `stacks`.** Oggi schema §C ha `EmitStatus { id, dur_param, chance_param, target }`. **Manca `stacks_param`** (Baby Flame = +2). Opzioni:
   - Estendere schema con `stacks_param: Option<String>` (default 1).
   - Oppure forzare il blueprint a tradurre `EmitStatus` in N applicazioni (rumoroso, sporca logs).
   - **Decisione consigliata:** estendere.

3. **Ordering Commands `on_enter`.** Multipli emits nello stesso `on_enter` (damage + status + particle): l'ordine di esecuzione blueprint è dichiarativo (ordine RON) o engine-defined? **Action item §2.2b:** specificare. **Proposta:** ordine RON = ordine emission, deterministico.

4. **Particle anchor `"mouth"` vs entità.** ~~Anchor stringa simbolica → blueprint la risolve sul rig Agumon~~ **RISOLTO 2026-05-12 via §2.2d**: no rig esiste, sostituito con `VfxLocus + VfxMotion`. Anchor body-part-specific collassa a `SelfCenter` (flavor mantenuto via particle name preset). Vedi `02-02d_vfx_positioning.md`.

5. **Ult charge da skill?** `units.ron.ultimate_accumulation_trigger: OnBasicAttack` — letteralmente "basic attack". Allora **Baby Flame non ricarica l'Ult**? Conferma da §combat_current.md — è una scelta o un bug? **Proposta design:** rinominare il campo o aggiungere `OnAnyAttack` come trigger; per Agumon, sia basic che heavy chargano Ult (+25 each). Senza, l'ult arriva troppo lentamente.

### 🟡 Aperte (non blocker)

- **Heated cap = 6 (canon, vedi 03 §8 G13).** Baby Flame +2 → 3 cast saturano il cap; apply oltre cap = no-op silente.
- Animation interrupt da kernel `Stun`/`Hurt`: in turn-based non succede durante il turno proprio, ma il design dell'FSM dovrebbe esplicitare "no preempt" per il caster owner durante la propria skill. **Skip M017.**

### 🔧 Decisioni round-3 (2026-05-12, HSR consolidation)

- **C-Shape — single-target bolt, no cone.** In line-combat HSR-style un "cone" non ha semantica positional (le slot sono fixed, no free movement). Single-target è canon-fedele ("small fireball spit dalla bocca") e HSR-pulito (parallelo a Jingliu/Asta/Welt basic). Particle name `fire_breath_cone` → `fire_breath_bolt`.
- **C-Tough — `tough_break` 10 → 20.** Baseline `units.ron.toughness_max=50` rendeva il 10 troppo basso (5 cast per break = saturazione meme). 20 è HSR-aligned per skill 1-SP single-target.
- **C-VFX — arrival/impact sync.** Aggiunti `fire_breath_impact` (Static su `EntityCenter(Primary)` dopo Travel 120ms) e `heated_mark` (pulse pure su Primary, una entry per ogni stack apply). Senza, lo stack accumulava invisibile e il bolt si "perdeva" senza impact frame.

## §7 — Verdetto

Heavy skill espone 3 gap concreti da risolvere in §2.2b prima di M017:
- (a) Toughness break command
- (b) Stacks parametrizzato in `EmitStatus`
- (c) Order semantics di `on_enter` Commands batch

Tutti **risolvibili dentro il vocabolario chiuso** estendendolo di 1 campo + 1 specifica. **No nuovo verbo richiesto.**

## §8 — Decisioni risolte (round-2)

### G2 — `EmitDamage` toughness break **[MEDIA]** ✅

**Decisione A canon:** estendere `EmitDamage` con campo opzionale `tough_break_param: Option<String>`. No nuovo verbo.

```rust
pub struct EmitDamageArgs {
    pub hits:              u8,
    pub mul_param:         String,                  // deref skill_def.params
    pub tag:               DamageTag,
    pub target_ref:        TargetRef,               // vedi G6
    pub tough_break_param: Option<String>,          // ← G2: deref param (None=no break)
}
```

`skills.ron` baby_flame:

```ron
params: {
    "heavy_mul":           Float(18.0),
    "heated_dur":          Int(3),
    "heated_skill_stacks": Int(2),
    "tough_break":         Int(20),     // bumped from 10 for HSR baseline (toughness_max=50, ≈ 2-3 cast per break)
},
```

Rationale: vocabolario chiuso piccolo; break è risorsa accoppiata al damage event, non status indipendente. Verbi B (`EmitBreak`) e C (`EmitStatus(Broken)`) scartati.

### G3 — `EmitStatus` stacks **[MEDIA]** ✅

Estendere lo schema `EmitStatus` con `stacks_param: Option<String>` (default 1 se assente).

```rust
pub struct EmitStatusArgs {
    pub id:            StatusId,
    pub target_ref:    TargetRef,
    pub chance_pct:    Option<u8>,        // mutex con chance_param (G3-prev)
    pub chance_param:  Option<String>,
    pub dur_pct:       Option<u8>,
    pub dur_param:     Option<String>,
    pub stacks_param:  Option<String>,    // ← G3 NEW (None = 1)
}
```

Baby Flame: `stacks_param: "heated_skill_stacks"` → 2 stack.

### G4 — Order semantics `on_enter` ✅

Confermato in 01/§8. Ordine RON dichiarativo = ordine emission. Baby Flame `Spit.on_enter` esegue: `EmitDamage(heavy)` → `EmitStatus(Heated)` → `SpawnParticle` → `Shake`. Se primary va `Broken` dal damage emit, `StatusApplied(Heated)` arriva post-break (Heated stacka indipendentemente, OK).

### G11 — `OnAnyAttack` (rename trigger) ✅

Vedi 01/§8. Baby Flame emette `+25 Ult charge` via kernel hook su `DamageDealt(kind=Heavy)`. Niente da fare nel blueprint Baby Flame stesso; è kernel-side.

### Particle anchor (gap §6.4) ✅ **SUPERSEDED 2026-05-12 → vedi `02-02d_vfx_positioning.md`**

~~`anchor: AnchorRef` enum chiuso~~ **NON valido**: nessun sprite rig esiste (atlas frame 2D piatti, no bone, no per-frame anchor data). Anchorare a "body parts" (Mouth/Weapon/ClawTip) era fake precision.

**Nuovo modello (`§2.2d`):** `SpawnParticle { name, origin: VfxLocus, motion: VfxMotion }`:
- `VfxLocus = SelfCenter | SelfAbove | TargetCenter | Adj(i8) | WorldGrid(IVec2)`
- `VfxMotion = Static | Travel { to, ease, ms } | Radial { range_tiles, ms }`

Risoluzione deterministica via bbox + line topology + grid. No rig table.

Baby Flame esempio post-migration (snapshot 2026-05-12): `SpawnParticle { name: "fire_breath_bolt", origin: SelfCenter, motion: Travel { to: EntityCenter(Primary), ease: EaseOut, ms: 120 } }` + `SpawnParticle { name: "fire_breath_impact", origin: EntityCenter(Primary), motion: Static }`. Rename `fire_breath_cone` → `fire_breath_bolt` riflette la decisione single-target HSR-style (no cone in line-combat); `TargetCenter` → `EntityCenter(Primary)` allinea alla grammatica nuova (single-target equivalente semanticamente).
