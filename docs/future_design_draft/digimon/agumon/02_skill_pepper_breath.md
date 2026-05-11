# Agumon — Heavy Skill: `pepper_breath` (FSM stress test)

> **Goal**: stressare un nodo con multiple emit (damage + status + toughness) e validare che la composition Commands → KernelEffect non perda ordering / scaling.

## §1 — Intent

- **Cost:** **1 SP** (HSR baseline canon, vedi §5b in `00_identity.md`) — **Gen:** +25 Ult charge (`OnBasicAttack` accumulation trigger; conferma: vale anche per Skill o solo per Basic? **Vedi §6.5**)
- **Effect:** Damage Fire `18` su single primary; **+2 Heated stacks**; **ToughnessHit(10)** sul break bar
- **Atlas clip:** `heavy_attack` (source frames 23–36, count 14)

## §2 — FSM topology

4-nodo: `Inhale → Wind → Spit → Recovery`.

```
            ┌──────────┐ TimeInNode(3) ┌──────────┐ TimeInNode(2) ┌──────────┐ TimeInNode(4) ┌────────────┐
   commit →│ Inhale   │ ────────────▶ │  Wind    │ ────────────▶ │  Spit    │ ────────────▶ │ Recovery   │ ──▶ exit
            │frames: 3 │               │frames: 2 │               │frames: 4 │               │ frames: 5  │
            └────┬─────┘               └────┬─────┘               └────┬─────┘               └────────────┘
                 │ on_enter:                │                          │ on_enter:
                 │  SpawnParticle           │                          │  EmitDamage { hits:1, mul_param:"heavy_mul" }
                 │   ("inhale_glow","mouth")│ on_enter:                │  EmitStatus { id:"heated", chance_pct:100,
                 │                          │  SpawnParticle           │                dur_param:"heated_dur",
                 │                          │   ("fire_charge","mouth")│                target:Primary, stacks_param:"heated_skill_stacks" }
                 │                          │                          │  EmitDamage { hits:0, mul_param:"-", tough_break:10 }   ← §6.3
                 │                          │                          │  SpawnParticle ("fire_breath_cone","mouth")
                 │                          │                          │  Shake { intensity:2, duration_ms:120 }
```

> **Nota.** `tough_break` non è command esistente; vedi §6.3. Alternativa: `EmitDamage { hits:1, mul_param:"heavy_mul", tough_break:10 }` (singolo verbo con campo aggiuntivo).

## §3 — Nodes table

| Node | frames | atlas src range | ms (@12fps ref) | on_enter (Commands) |
|---|---|---|---|---|
| `Inhale` | 3 | 23–25 | 0–250 | `SpawnParticle("inhale_glow","mouth")` |
| `Wind` | 2 | 26–27 | 250–416 | `SpawnParticle("fire_charge","mouth")` |
| `Spit` | 4 | 28–31 | 416–750 | `EmitDamage(heavy, +tough_break:10)`, `EmitStatus(Heated, stacks=2)`, `SpawnParticle("fire_breath_cone")`, `Shake` |
| `Recovery` | 5 | 32–36 | 750–1166 | — (animazione di ritorno postura) |

**Frame budget:** 14 frames (= atlas clip esatto). No stretch.

## §4 — Command param resolution

| param | source | example |
|---|---|---|
| `heavy_mul` | `skills.ron` numbers + caster ATK + target DEF; current legacy `Damage(amount:18)` | scalar ≈ 18 |
| `heated_dur` | `skills.ron.params["heated_dur"]` | 3 turni (condiviso con basic) |
| `heated_skill_stacks` | `skills.ron.params["heated_skill_stacks"]` | 2 |
| `tough_break` (literal su EmitDamage) | `skills.ron.effects[].tough_break` (oggi `ToughnessHit(10)`) | 10 |

## §5 — Kernel events expected

Su `Spit.on_enter` → blueprint Agumon emette (in ordine):

1. `KernelEffect::Damage { target, amount: 18+, tag: Fire }` → `CombatEvent::DamageDealt`
2. `KernelEffect::ToughnessHit { target, amount: 10 }` → `CombatEvent::ToughnessReduced` / eventuale `CombatEvent::Broken`
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

2. **`EmitStatus` plurale `stacks`.** Oggi schema §C ha `EmitStatus { id, dur_param, chance_param, target }`. **Manca `stacks_param`** (Pepper Breath = +2). Opzioni:
   - Estendere schema con `stacks_param: Option<String>` (default 1).
   - Oppure forzare il blueprint a tradurre `EmitStatus` in N applicazioni (rumoroso, sporca logs).
   - **Decisione consigliata:** estendere.

3. **Ordering Commands `on_enter`.** Multipli emits nello stesso `on_enter` (damage + status + particle): l'ordine di esecuzione blueprint è dichiarativo (ordine RON) o engine-defined? **Action item §2.2b:** specificare. **Proposta:** ordine RON = ordine emission, deterministico.

4. **Particle anchor `"mouth"` vs entità.** Anchor è una stringa simbolica → blueprint la risolve sul rig Agumon. Headless ignora. OK ma serve specificare per il presentation contract: l'anchor è una stringa libera o un enum?

5. **Ult charge da skill?** `units.ron.ultimate_accumulation_trigger: OnBasicAttack` — letteralmente "basic attack". Allora **Pepper Breath non ricarica l'Ult**? Conferma da §combat_current.md — è una scelta o un bug? **Proposta design:** rinominare il campo o aggiungere `OnAnyAttack` come trigger; per Agumon, sia basic che heavy chargano Ult (+25 each). Senza, l'ult arriva troppo lentamente.

### 🟡 Aperte (non blocker)

- Heated cap (proposta 6) — Pepper Breath ne aggiunge 2, quindi 3 cast portano al cap. OK.
- Animation interrupt da kernel `Stun`/`Hurt`: in turn-based non succede durante il turno proprio, ma il design dell'FSM dovrebbe esplicitare "no preempt" per il caster owner durante la propria skill. **Skip M017.**

## §7 — Verdetto

Heavy skill espone 3 gap concreti da risolvere in §2.2b prima di M017:
- (a) Toughness break command
- (b) Stacks parametrizzato in `EmitStatus`
- (c) Order semantics di `on_enter` Commands batch

Tutti **risolvibili dentro il vocabolario chiuso** estendendolo di 1 campo + 1 specifica. **No nuovo verbo richiesto.**
