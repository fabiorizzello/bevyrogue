# Agumon — Basic: `sharp_claws` (FSM stress test)

> **Goal**: validare baseline FSM con la skill più semplice. Se la FSM non riesce a esprimere un basic attack pulito, abbiamo già un problema.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP — **Gen:** +1 SP, +25 Ult charge (via `OnBasicAttack`)
- **Effect:** Damage Fire `≈8` su single primary; **+1 Heated stack** (status, target-scoped)
- **Atlas clip:** `attack` (source frames 0–8, count 9)

## §2 — FSM topology

3-nodo: `Windup → Strike → Recovery → (exit to Idle)`.

```
            ┌───────────┐  TimeInNode(2)   ┌──────────┐  TimeInNode(2)  ┌────────────┐
   commit →│  Windup    │ ───────────────▶ │  Strike  │ ──────────────▶ │  Recovery  │ ──▶ exit
            │ frames: 2 │                  │ frames: 4│                 │ frames: 3  │
            └───────────┘                  └────┬─────┘                 └────────────┘
                                                │
                                              on_enter:
                                                EmitDamage { hits:1, mul_param:"basic_mul" }
                                                EmitStatus { id:"heated", dur_param:"heated_dur",
                                                             chance_param:"heated_chance", target:Primary }
                                                SpawnParticle { name:"fire_claw_trail",  origin: SelfCenter,            motion: Static }
                                                SpawnParticle { name:"fire_claw_impact", origin: EntityCenter(Primary), motion: Static }   # NEW impact flash
                                                SpawnParticle { name:"heated_mark",      origin: EntityCenter(Primary), motion: Static }   # NEW stack pulse
                                                Shake { intensity:1, duration_ms:80 }
```

## §3 — Nodes table

| Node | frames | atlas src range | ms (@12fps ref) | on_enter (Commands) | exit edge |
|---|---|---|---|---|---|
| `Windup` | 2 | 0–1 | 0–166 | `Shake { intensity:1, duration_ms:60 }` (anticipation tick) | `TimeInNode` → `Strike` (prio 0) |
| `Strike` | 4 | 2–5 | 166–500 | `EmitDamage`, `EmitStatus(Heated)`, `SpawnParticle("fire_claw_trail")` (`SelfCenter` Static), `SpawnParticle("fire_claw_impact")` (`EntityCenter(Primary)` Static), `SpawnParticle("heated_mark")` (`EntityCenter(Primary)` Static), `Shake { intensity:1, duration_ms:80 }` | `TimeInNode` → `Recovery` (prio 0) |
| `Recovery` | 3 | 6–8 | 500–750 | — | `TimeInNode` → exit (FSM completes; AnimGraph idle resumes) |

**Frame budget:** 9 frames (= atlas clip esatto). No stretch.

## §4 — Command param resolution (snapshot-once)

Parameters risolti dal blueprint **al commit_action** (snapshot-once, §2.2b §F):

| param | source | esempio value |
|---|---|---|
| `basic_mul` | `skills.ron` numbers + caster ATK + target DEF | scalar damage finale ≈ 8 |
| `heated_dur` | `skills.ron` (TBD: aggiungere `params: { heated_dur: 3 }`) | 3 turni |
| `heated_chance` | `skills.ron` (TBD: 100% per basic) | 100 (= certain) |

> **Gap rilevato:** `skills.ron` schema attuale (`Effect::Damage{amount,target}`, `Effect::ToughnessHit(n)`) **non ha campo `params`**. Per FSM Commands con `mul_param`/`dur_param` serve estendere `SkillDef`. Vedi §6.

## §5 — Kernel events expected (post-skill)

Sul kernel bus dopo `Strike.on_enter` (= dopo che il blueprint traduce Commands):

1. `CombatEvent::DamageDealt { target, amount, tag: Fire, caster: Agumon }`
2. `CombatEvent::StatusApplied { target, status: Heated, stacks: 1 }`
3. `CombatEvent::SpEarned { actor: Agumon, amount: 1 }` (da `OnBasicAttack`)
4. `CombatEvent::UltimateCharged { actor: Agumon, amount: 25 }` (da `OnBasicAttack`)

**Twin Core listener (passive):** se Gabumon è in team e ha `Chilled` applicato su qualcuno questo turno, il listener Agumon (file 04) può intercettare l'evento `StatusApplied(Chilled)` e modificare il damage di `DamageDealt` next-hit. Vedi `04_passive_twin_core_fire.md`.

## §6 — Stress test findings

### ✅ Cosa funziona

- 3 nodi sono **largamente sufficienti** per un basic. Il caso degenere (1-2 nodi) sarebbe troppo povero per timing particle/shake; 3 nodi danno un anticipation/impact/recovery leggibile.
- Tutte le Commands usate (`EmitDamage`, `EmitStatus`, `SpawnParticle`, `Shake`) sono nel vocabolario chiuso §2.2b §C. **No drift**.
- Predicate solo `TimeInNode` → headless deterministic OK senza altri input.

### ⚠️ Contraddizioni / gap

1. **Param plumbing mancante.** Il vocabolario `EmitDamage { mul_param:"basic_mul" }` presuppone che `skills.ron` esponga un campo `params: HashMap<String, Value>`. Oggi `SkillDef` ha solo `effects: Vec<Effect>`. **Action item:** §2.1 (data/logic separation) deve definire `params` map come parte di `SkillDef`. Senza, le Commands o sono inline-literal (rompe data/logic separation) o sono opache.

2. **`heated_chance` da chi viene risolto?** Se il blueprint lo legge da `params`, OK. Se è hard-coded "basic = 100%, skill = 100%, ult = derived", allora la Command `EmitStatus { chance_param }` ha `chance_param` opzionale o si rinomina `chance_pct: u8`? **Proposta:** in M017 lo schema Command include sia `chance_pct` (literal) che `chance_param` (deref), mutex.

3. **`SpawnParticle` headless drop.** §2.2b §G dice "cosmetic Commands sono no-op headless". OK in teoria, ma `EmitDamage` segue lo stesso frame del `SpawnParticle`: chi garantisce che il blueprint headless **non droppi anche `EmitDamage`**? **Action item:** la classificazione Command → cosmetic/gameplay deve essere statica nel match arm (e.g. enum tagging), non un toggle runtime sul blueprint.

4. **Ult charge timing.** `OnBasicAttack` accumula 25 al cap-min. **Domanda:** evento `UltimateCharged` parte allo `Strike.on_enter` (al `EmitDamage`) o al completamento FSM (`Recovery.exit`)? Se durante `Strike`, e nello stesso turno l'ult bar arriva a `ultimate_trigger=100`, il giocatore può ulta-mente subito al turno dopo. Se al `Recovery.exit`, c'è un delay percepito. **Proposta:** ult charge è effect del kernel su `DamageDealt`, non FSM-emit. La FSM emette `EmitDamage`, il kernel applica e contestualmente accumula. **Consistency:** allineare con `agumon_ult` charge rules (la skill ult **non** charga sé stessa).

5. **Shake duration in ms.** §2.2b §C: `Shake { intensity, duration_ms }`. **Conflitto con §G** (frame counter, no wall-clock)? Soluzione: shake è effect **presentation-only**, headless lo droppa, quindi ms in `Shake` è metadata UI come per QTE `window_ms`. Conferma esplicita nel doc §2.2b. ← **da aggiungere** come nota.

### 🟡 Aperte (non blocker)

- **Heated cap = 6 (canon, vedi 03 §8 G13).** Basic +1 → 6 basic puri saturano il cap; cap raggiungibile più rapidamente via mix con `baby_flame` (+2/cast).
- `Hurt` interrupt: se Agumon viene colpito durante `Windup` (rare, non c'è sim. attack), la FSM viene preempted dal kernel? Probabilmente no in turn-based (è sempre il turno di Agumon durante la sua skill). **Skip per ora.**

## §7 — Verdetto

Baseline FSM **funziona** per un basic attack. I gap (1–4) sono **architetturali, non basic-specifici** — emergeranno anche per Heavy/Ult. Risolverli su 02/03.

## §8 — Decisioni risolte (round-2)

### G1 — `SkillDef.params` plumbing **[ALTA]** ✅

**Decisione canon:** estendere lo schema `skills.ron` con un campo `params: HashMap<String, ParamValue>` su `SkillDef`. Le Commands FSM referenziano i param **per nome** (es. `mul_param: "basic_mul"`).

```rust
// src/data/skills_ron.rs (schema target post-G1)
pub struct SkillDef {
    pub id: String,
    pub cost: SpCost,
    pub effects: Vec<Effect>,
    pub params: HashMap<String, ParamValue>, // ← NEW
}

pub enum ParamValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}
```

**Regole:**
- Naming: snake_case (`basic_mul`, `heated_dur`, `heated_chance`, `qte_window`).
- Lookup at-commit: il blueprint risolve `mul_param: "basic_mul"` come `skill_def.params["basic_mul"]` al `commit_action` (G5 cluster `Snapshot`).
- Param **assente** = error fatal al load di `skills.ron` (`validate_skill_params()` in `src/data/units_ron.rs` o equivalente).
- Literal inline (es. `Shake { intensity: 1 }`) restano consentiti per costanti presentation; tutto ciò che è gameplay-tunable passa per `params`.

**Esempio `skills.ron` sharp_claws post-G1:**

```ron
SkillDef(
    id: "agumon_basic_sharp_claws",
    cost: SP(0),
    effects: [/* legacy o derivati */],
    params: {
        "basic_mul":      Float(8.0),
        "heated_dur":     Int(3),
        "heated_chance":  Int(100),
    },
),
```

### G3-prev (chance vs param) ✅

`EmitStatus` accetta **entrambi** `chance_pct: u8` (literal) e `chance_param: String` (deref), **mutuamente esclusivi** (validato al load). Stessa regola per `dur_pct` vs `dur_param`. Rationale: literal per status hard-coded 100%, param per skill che variano per tier/upgrade.

### G3-cosmetic (Shake/Particle headless drop) ✅

**Tagging statico in enum Command**, non runtime toggle. Schema target:

```rust
pub enum Command {
    // gameplay (eseguite sempre, anche headless)
    EmitDamage(EmitDamageArgs),
    EmitStatus(EmitStatusArgs),
    EmitHeal(EmitHealArgs),
    // ... (vedi §9 in agumon/04)

    // presentation (no-op headless, ufficiale via cfg/feature)
    Shake(ShakeArgs),
    SpawnParticle(SpawnParticleArgs),
    PlaySound(PlaySoundArgs),
}
```

Interprete: match arm cosmetic-set → `#[cfg(feature = "windowed")]` o branch headless drop. Niente toggle dinamico sul blueprint.

### G11 — Ult charge trigger ✅

`OnBasicAttack` rinominato → `OnAnyAttack` in `units.ron`. Sia basic che heavy (skill) accumulano +25 Ult charge. Ult non charga sé stessa (ricarica solo via basic/skill). Hook nel kernel su `CombatEvent::DamageDealt` filtrato per `skill_kind ∈ {Basic, Heavy}`.

### G4 — Ordering Commands `on_enter` ✅

**Ordine RON = ordine di emission, deterministico.** Il blueprint itera la `Vec<Command>` del nodo in ordine di dichiarazione e emette `KernelEffect` sequenziali. Da formalizzare nel doc §2.2b §H come contratto.
