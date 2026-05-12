# Agumon — Ult: `baby_burner` (FSM stress test)

> **Goal**: stressare il caso più complesso del kit Agumon — Ult con **edge reattivo** (`OnKill → ReactiveDetonate`), **QTE HitCheck** v1 durante windup (singolo press, esito binario), **splash AoE secondario**, e modifier-firma `Detonate(Heated)`.

> **VFX positioning:** `SpawnParticle` usa `origin: VfxLocus + motion: VfxMotion` per `§2.2d` (`02-02d_vfx_positioning.md`).

## §1 — Intent

- **Cost:** 0 SP, **consuma full ult bar** (`ultimate_trigger=100` raggiunto). Non charga sé stesso. **Lanciabile anytime off-turn** (HSR-style) — non occupa lo slot azione del turno.
- **Effect base:** Damage Fire alto `50` su primary; splash 50% sui 2 adj (Blast shape).
- **Modifier-firma:** `OnKill→Detonate(Heated)` — se il primary muore dall'Ult hit, **gli stack rimanenti di `Heated`** del primary vengono detonati come damage aggiuntivo sui 2 adj.
- **QTE v1 (semplice):** Hit Check — singolo press dentro la window di Windup. **Success** → splash damage moltiplicato per `ult_splash_mul_boosted` (≈ +25%). **Fail / no input** → splash base `ult_splash_mul`. Nessun mash, nessuna scala continua. Headless default = `success` (deterministic test).
- **Atlas clip:** `skill` (source frames 50–66, count 17)

## §2 — FSM topology (4-node con edge reattivo + QTE)

```
                        QTE success → splash_boosted
        ┌──────────┐                           ┌──────────┐
commit→ │  Windup  │ ──StartQTE→ Suspend  ──── │  Strike  │
        │ frames:5 │     |                     │ frames:4 │
        └────┬─────┘     │ YieldResolved       └────┬─────┘
             │           │ (UserInput/headless)    │
             │           ▼                         │ on_enter:
             │   (resume Strike)                   │  EmitDamage(primary, hits:1, mul:"ult_mul", tough_break:30)
             │                                     │  EmitDamage(adj_l,  hits:1, mul:"ult_splash_mul")
             │                                     │  EmitDamage(adj_r,  hits:1, mul:"ult_splash_mul")
             │                                     │  EmitStatus(Heated, target:Primary, stacks:1) ← optional
             │                                     │  SpawnParticle("baby_burner_blast",  origin: EntityCenter(Primary), motion: Radial { range_tiles: 1.5, ms: 200 })   ← AoE shape esplicito
             │                                     │  SpawnParticle("baby_burner_impact", origin: EntityCenter(Primary), motion: Static)                                ← primary impact
             │                                     │  SpawnParticle("baby_burner_splash", origin: EntityCenter(AdjLeft),  motion: Static)                               ← per-adj impact
             │                                     │  SpawnParticle("baby_burner_splash", origin: EntityCenter(AdjRight), motion: Static)                               ← per-adj impact
             │                                     │  Shake { intensity:4, duration_ms:200 }
             │                                     │
        on_enter:                                  ├── edge A: KernelEvent(UnitDied {target: primary}) prio:10
         SpawnParticle("baby_burner_charge",       │            ──▶ ReactiveDetonate
            origin: SelfCenter, motion: Static)    │
         StartQTE { kind:"HitCheck", ... }         └── edge B: TimeInNode prio:0
                                                                 ──▶ Recovery
        ┌────────────────────┐                     ┌────────────┐
        │ ReactiveDetonate    │ TimeInNode(2)      │  Recovery  │
        │ frames: 3           │ ──────────────────▶│ frames: 5  │ ──▶ exit
        └────┬────────────────┘                    └────────────┘
             │ on_enter:
             │  EmitDamage(adj_l, hits:1, mul:"detonate_per_stack", stacks_ref:"heated_consumed")
             │  EmitDamage(adj_r, hits:1, mul:"detonate_per_stack", stacks_ref:"heated_consumed")
             │  SpawnParticle("heated_detonate", origin: EntityCenter(AdjLeft),  motion: Static)    ← per-target detonate (primary è morto, niente VFX su di lui)
             │  SpawnParticle("heated_detonate", origin: EntityCenter(AdjRight), motion: Static)
             │  Shake { intensity:3, duration_ms:160 }
```

## §3 — Nodes table

| Node | frames | atlas src range | ms (@12fps ref) | Note |
|---|---|---|---|---|
| `Windup` | 5 | 50–54 | 0–416 | apertura, occhi che brillano. **`on_enter`**: `SpawnParticle("baby_burner_charge", origin: SelfCenter, motion: Static)` (windup glow — sells il pre-cast, senza la QTE arriverebbe nuda visivamente), poi `StartQTE { kind:"HitCheck", window_param:"qte_window", headless_default_param:"qte_default_headless" }` (v1 simple, vedi §8 G8) |
| `Strike` | 4 | 55–58 | 416–750 | il blast vero. `EmitDamage(primary/adj_l/adj_r)` + `EmitStatus(Heated primary)` + 4× `SpawnParticle` (blast `Radial` AoE shape su Primary, primary impact, 2× splash impact su adj) + `Shake`. Vedi §2 |
| `ReactiveDetonate` | 3 | 59–61 | 750–1000 | **solo se edge A taken**. Splash secondario da stacks Heated rimasti. Detonate VFX **per-target** sui 2 adj (primary è morto, no VFX su di lui) |
| `Recovery` | 5 | 62–66 | 1000–1416 | postura ritorno (anche se ReactiveDetonate fired, recovery riprende dopo) |

**Frame budget se no detonate:** 5+4+5 = 14 frames (≠ atlas 17). **3 frames "orfani"** (62-66 → 62-64 da skippare). Vedi §6.4.

**Frame budget se detonate:** 5+4+3+5 = 17 frames. ✅ matcha atlas.

## §4 — Param resolution snapshot-once (al commit)

| param | source | example |
|---|---|---|
| `ult_mul` | ATK + ult bonus | `50` legacy |
| `ult_splash_mul` | derived 50% di `ult_mul` | `25` |
| `ult_splash_mul_boosted` | derived 62.5% di `ult_mul` (`ult_splash_mul × 1.25`) — usato se `last_qte_result=="success"` | `31` |
| `detonate_per_stack` | TBD design (proposta `8` per stack) | scalar per `heated_consumed` |
| `heated_consumed` | **live read on `ReactiveDetonate.on_enter`** (NON snapshot — è il count residuo al momento della morte) | 0..6 (cap canon, vedi §0 + 02§7) |
| `qte_window` | `skills.ron.params["qte_window"]` | 500ms metadata UI (≈ durata di Windup @12fps) |
| `qte_default_headless` | `skills.ron.params["qte_default_headless"]` | `"success"` (deterministic per test) |

**Conflitto con §F snapshot-once.** `heated_consumed` non può essere snapshot al commit perché dipende dal kernel state nel momento esatto della morte del primary durante `Strike`. **Soluzione:** §F dice "kernel events live durante la skill" → il blueprint legge il count dal `CombatEvent::UnitDied` payload (`heated_remaining`). Quindi `KernelEvent` filter porta payload utilizzabile dalle Commands. Vedi §6.5.

## §5 — Kernel events flow

```
Strike.on_enter
  ├─ EmitDamage(primary)        → CombatEvent::DamageDealt(primary, 50, Fire)
  │                              [primary HP scende a 0]
  │                              → CombatEvent::UnitDied { unit: primary,
  │                                                         status_remaining: [Heated × N] }
  ├─ EmitDamage(adj_l)          → CombatEvent::DamageDealt(adj_l, 25, Fire)
  ├─ EmitDamage(adj_r)          → CombatEvent::DamageDealt(adj_r, 25, Fire)
  ├─ EmitStatus(Heated primary) → ??? primary è morto; skip / drop
  ├─ SpawnParticle              → presentation
  └─ Shake                       → presentation

FSM tick successiva:
  edge A `KernelEvent(UnitDied{target:primary})` matches → go ReactiveDetonate
  edge B `TimeInNode(4)` falls through → ignored

ReactiveDetonate.on_enter
  ├─ EmitDamage(adj_l, mul:detonate_per_stack × N)
  ├─ EmitDamage(adj_r, mul:detonate_per_stack × N)
  └─ ...
```

## §6 — Stress test findings

### ✅ Cosa funziona

- L'edge reattivo `KernelEvent(UnitDied) → ReactiveDetonate` è esattamente il caso d'uso per cui §2.2b vince contro la lista piatta §2.2. Il design **regge**.
- 4-nodi con fallback su `TimeInNode` evita il deadlock (se primary non muore, l'FSM avanza comunque a Recovery).
- QTE come `StartQTE → Suspend → YieldResolved` riusa il meccanismo §2.6 esistente, niente nuovo verbo.

### ⚠️ Contraddizioni / gap (più ricchi di skill base)

1. **`KernelEvent` payload come param source.** §2.2b §F dice "kernel events live"; §C non specifica che il **payload dell'evento** è leggibile dalle Commands successive. **Action item:** §H contratto interprete deve esporre `last_kernel_event_payload` allo step `on_enter` del nodo target. Es. `mul_param: "$event.heated_remaining * 8"` oppure più pulito: param `stacks_ref:"heated_consumed"` dove `heated_consumed` è un **derived value** che il blueprint risolve leggendo l'ultimo `UnitDied` event. **Decisione consigliata:** vocabolario param ha 2 source kinds: `Snapshot` (statico, al commit) e `EventPayload` (live, dall'ultimo edge predicate matched).

2. **Edge priority resolution.** §D dice "priority u8 descending, tie-break dichiarazione". **Caso da chiarire**: cosa succede se nello stesso tick FSM matchano *contemporaneamente* `KernelEvent(UnitDied)` E `TimeInNode(4)`? Se Strike è 4 frames e al frame 4 il primary muore, entrambi predicati sono true. Priority 10 vs 0 → A vince. **OK come scritto.** ✓ Conferma necessaria nel doc esempio.

3. **Splash damage shape.** §C `EmitDamage { hits, mul_param, ... }` ha `hits: u8` (N hits sullo stesso target). **Non ha targeting**: come gestiamo damage su adj_l/adj_r? Opzioni:
   - **A.** 3 Commands `EmitDamage` separate, una per target risolto al commit (primary/adj_l/adj_r). Verboso ma esplicito.
   - **B.** `EmitDamage { target_shape: TargetShape, mul_per_target }` — sposta targeting nel verbo. Più complesso.
   - **Decisione consigliata:** A. Targeting risolto dal blueprint al commit (snapshot dei target effettivi); FSM emette N command per N target. Resta dichiarativa.

4. **Frame orfani su path non-detonate.** Se primary sopravvive, la FSM va `Strike → Recovery` (frames 14) ma l'atlas ha 17. Discrepanza visibile. Opzioni:
   - **A.** Recovery padded a 8 frames quando no-detonate (stretch logico). Add edge: `KernelEvent(UnitDied) →` `Recovery(short)`, fallback `Recovery(long)`.
   - **B.** Strike dura 7 frames se no detonate (perde frames disponibili al `ReactiveDetonate`).
   - **C.** Recovery sempre 5 frames; gli ultimi 3 frames atlas sono spostati a un nodo `Hold` post-detonate sintetico.
   - **Decisione consigliata:** A. Recovery come 2 varianti via edge priority, 5 vs 8 frames.

5. **Ult charge accumulation post-cast.** Cosa succede al `ultimate_trigger` bar dopo l'ult? Si svuota (`-100`) o rimane al `ultimate_cap=150` overflow trigger? **Legacy units.ron:** `ultimate_trigger=100`, `ultimate_cap=150`. Implica overflow stored ma capped a 150 — overcap usato per "rapid second ult"? Confermare design.

6. **QTE windup window vs animation frames.** `Windup` dura 5 frames (~416ms @12fps). **V1 semplice:** `qte_window = 500ms` ≈ durata Windup. Il QTE è un singolo press (`HitCheck`), risolto a `success`/`fail` entro la finestra; nessuna estensione del nodo. `StartQTE` emesso `on_enter Windup` → FSM `Suspend` (frame counter pausato) → input entro window o timeout → `YieldResolved(result)` → `Resume`. Headless: `qte_default_headless` snapshot risolve istantaneamente senza sospensione visibile. **Vedi §H per il contratto preciso.**

7. **`StartQTE` in headless test.** Test headless determinico richiede `qte_default_headless`. ✓ §G. Ma **dove è memorizzato?** In `skills.ron.params`? Sì. **Action item:** estendere schema con `params: { qte_default_headless: "success" }`.

8. **Modifier-firma "OnKill→Detonate(Heated)" è una skill-tree variant?** §8 roster minimal lo cita come "modifier-firma" della Ult. Nel design §I (skill_tree.ron file separato), questo modifier sarebbe definito **dentro skill_tree.ron** o come **default Ult node behavior**? Senza skill_tree (in M017), il modifier è inline nella FSM (edge A). Quando skill_tree arriva, il modifier diventa un overlay che attiva/disattiva l'edge A. **Quesito design:** la FSM base ha l'edge sempre o solo con unlock? **Proposta canon §8:** sempre attivo (è la modifier-firma di default, non opzione). Confermare.

### 🟡 Aperte (non blocker M017)

- **Heated cap = 6 (canon, vedi 00§5 + 02§7).** Detonate consuma tutti gli stack rimasti.

### 🔧 Decisioni round-3 (2026-05-12, HSR consolidation VFX)

- **U-VFX-Charge — windup glow esplicito.** `Windup.on_enter` aggiunge `SpawnParticle("baby_burner_charge", origin: SelfCenter, motion: Static)`. Senza, la QTE arrivava visivamente nuda (solo "occhi che brillano" come placeholder testuale). Allinea il pre-cast all'urgenza della Ult.
- **U-VFX-Blast — origin `EntityCenter(Primary)` + motion `Radial`.** `baby_burner_blast` cambia da `TargetCenter` Static → `EntityCenter(Primary)` `Radial { range_tiles: 1.5, ms: 200 }`. Sells la shape AoE: il blast si espande dal target centrale verso i due adj.
- **U-VFX-Splash — per-target impact su adj.** Aggiunti due `SpawnParticle("baby_burner_splash", origin: EntityCenter(AdjLeft/AdjRight), motion: Static)`. Prima i due adj prendevano damage number senza visual feedback ("dove è arrivato il colpo?"). Ora il loop è chiuso: 3 impact frame, 3 damage event.
- **U-VFX-Detonate — per-target su adj.** `heated_detonate` cambia da `TargetCenter` Static (sbagliato, il primary è morto) → 2 emit Static su `EntityCenter(AdjLeft/AdjRight)`. Il detonate è semanticamente *sui due adj*, non sul centro morto del primary.

## §7 — Verdetto

Ult espone 5 gap architetturali (1, 2, 3, 4, 6) di cui **2 nuovi** non emersi in basic/skill:
- (1) Param source kind: `Snapshot` vs `EventPayload` (vocabolario param map a 2 cluster)
- (3) Multi-target damage: 3 Commands separate vs targeting nel verbo
- (4) Frame budget mismatch per path optional (atlas vs FSM)
- (6) QTE timing che eccede il nodo emittente

Gap (2) si chiarisce. Gap (5) è game-design, non FSM.

**Tutti risolvibili nel framework §2.2b senza inventare un nuovo modello.** L'Ult è il **caso d'uso più forte** per la FSM contro la lista piatta — l'edge reattivo `OnKill` è esattamente ciò che la lista non può fare.

## §8 — Decisioni risolte (round-2)

### G5 — Param source kind: `Snapshot` vs `EventPayload` **[ALTA]** ✅

**Decisione canon:** vocabolario param ha **2 source kinds**, esposti dal blueprint al match arm dei Command:

```rust
pub enum ParamRef {
    /// Risolto al `commit_action` (static per la durata della skill).
    /// Esempio: `Snapshot("basic_mul")` → cerca `skill_def.params["basic_mul"]`.
    Snapshot(String),

    /// Risolto live al momento dell'`on_enter` del nodo, dall'ultimo
    /// `KernelEvent` matched da un edge predicate. Path dot-notation sul payload.
    /// Esempio: `EventPayload("heated_remaining")` → ultimo `UnitDied.heated_remaining`.
    EventPayload(String),
}
```

**Regole:**
- I Command che accettano `param_name: String` diventano `param_ref: ParamRef`.
- Default in RON: shorthand stringa `"basic_mul"` → `Snapshot("basic_mul")`. Per `EventPayload` syntax esplicita: `"$event.heated_remaining"` o `EventPayload("heated_remaining")` (decisione: prefisso `$event.` in RON per leggibilità).
- `EventPayload` valido **solo** in nodi raggiunti via edge `KernelEvent(...)` predicate. Validazione al load: nodi senza edge kernel-event in input non possono usare `EventPayload` → fatal config error.
- Interprete blueprint mantiene un `last_matched_event: Option<KernelEventPayload>` per il nodo corrente; resetta al `node_enter`.

**Esempio Baby Burner `ReactiveDetonate`:**

```ron
ReactiveDetonate: (
    frames: 3,
    on_enter: [
        EmitDamage(
            target_ref: AdjLeft,
            hits: 1,
            mul_param: "$event.heated_remaining * detonate_per_stack",   // EventPayload
            tag: Fire,
        ),
        // ... idem adj_r
    ],
)
```

Nota: la moltiplicazione inline (`* detonate_per_stack`) richiede mini-expression DSL nel `mul_param`. **Decisione semplificata:** invece di DSL, il blueprint ha un `multiplier_chain: Vec<ParamRef>` su `EmitDamage`:

```rust
pub struct EmitDamageArgs {
    pub multiplier_chain: Vec<ParamRef>,  // moltiplicati tra loro: result = ∏
    pub tag:              DamageTag,
    pub target_ref:       TargetRef,
    pub tough_break_param: Option<ParamRef>,
}
```

`ReactiveDetonate` → `multiplier_chain: [EventPayload("heated_remaining"), Snapshot("detonate_per_stack")]`. Pulito, no parser.

### G6 — Multi-target damage **[MEDIA]** ✅

**Scelta A canon:** il blueprint emette N `EmitDamage` separati, **uno per target** risolto al commit. Il `TargetShape` (Blast, Bounce, AoE) sta nel **blueprint**, non nel verbo Command.

```rust
pub enum TargetRef {
    // statici al commit
    Primary,
    Self_,
    AdjLeft,
    AdjRight,
    AllySlot(u8),

    // risolti al commit via `TargetShape` resolver
    AdjLowestHp,
    LowestHpPctAlive,
    RandomEnemyAlive,           // RNG seed: TurnRng (deterministico)

    // live, dal kernel event
    EventTarget,                // ultimo event.target
}
```

`TargetShape` (livello skill) resta separato e vive nel blueprint per espandere a un set di `TargetRef`:

```rust
// at commit_action, il blueprint chiama:
fn resolve_shape(shape: TargetShape, ctx: &CommitCtx) -> Vec<TargetRef>;
```

Es. `TargetShape::Blast(primary)` → `[Primary, AdjLeft, AdjRight]` → il blueprint emette **3 EmitDamage** distinti su `Strike.on_enter`, con `mul_param` snapshot dei rispettivi `ult_mul` / `ult_splash_mul`.

Rationale: vocabolario Command resta semplice (single-target), shape resolver è blueprint-side e testabile in isolamento.

### G7 — Frame budget mismatch (atlas vs FSM) **[MEDIA]** ✅

**Scelta A canon:** Recovery in 2 varianti, scelta via edge priority dal nodo `Strike`. Nessun frame "orfano".

Topology aggiornata:

```
Strike ──(edge A: KernelEvent(UnitDied), prio:10)──▶ ReactiveDetonate ──TimeInNode──▶ Recovery_short (5f)
       └─(edge B: TimeInNode, prio:0)─────────────▶ Recovery_long (8f)
```

| Path | Strike | Detonate | Recovery | Total | Atlas (17f) |
|---|---|---|---|---|---|
| Kill primary | 4 | 3 | 5 (short) | 12 ❌ | 17 |
| No kill | 4 | — | 8 (long, 5+3 padding) | 12 ❌ | 17 |

Wait, 12 ≠ 17. Correzione: include Windup (5f). 5+4+3+5 = 17 (kill path) e 5+4+8 = 17 (no-kill, Recovery_long padded). ✅ Atlas matcha.

**Regola generale:** se l'atlas ha più frame del FSM-min, padding va nell'ultimo nodo della path più corta. Doc §2.2b §G aggiungere nota "frame padding allocation policy".

### G8 — QTE timing (window vs node frames) **[MEDIA]** ✅

**Decisione canon (V1 simple — HitCheck):**

- Kind QTE supportato in V1: **uno solo**, `HitCheck` (singolo press, esito binario `success`/`fail`). Niente mash, niente scala continua, niente multi-input. Sufficiente per validare contract `StartQTE`/`Suspend`/`YieldResolved`/`Resume` senza inventare gameplay aggiuntivo.
- `StartQTE` emesso `on_enter Windup` → kernel mette FSM in `Suspend` state: frame counter pausato, no edge eval.
- Window canon: `qte_window = 500ms` (≈ durata di `Windup` @12fps reference). Se l'input non arriva entro window → auto-`fail`. Se arriva → `success`.
- Quando `YieldResolved(qte_result)` arriva (input utente o `headless_default_param` snapshot), FSM `Resume` riprende dal frame **snapshot pre-suspend** (= frame all'istante dello `StartQTE`).
- Risultato QTE memorizzato in `blueprint_state["last_qte_result"]` (string: `"success"`/`"fail"`), leggibile come `ParamRef::BlueprintState("last_qte_result")` (extension §9 cross-roster) da Command successive.

**Effetto game-side V1 (semplice):** il blueprint Agumon su `Strike.on_enter` sceglie lo splash multiplier in base a `last_qte_result`:

```ron
Strike: (
    frames: 4,
    on_enter: [
        // primary full
        EmitDamage(target_ref: Primary, multiplier_chain: [Snapshot("ult_mul")], tag: Fire, tough_break_param: Some("ult_tough_break")),
        // splash: blueprint sceglie ult_splash_mul_boosted vs ult_splash_mul su last_qte_result
        EmitDamage(target_ref: AdjLeft,  multiplier_chain: [BlueprintState("splash_mul_selected")], tag: Fire),
        EmitDamage(target_ref: AdjRight, multiplier_chain: [BlueprintState("splash_mul_selected")], tag: Fire),
        // VFX layered: shape AoE su Primary + per-target impact (sells "AoE da 3 target", non un poof generico)
        SpawnParticle(name: "baby_burner_blast",  origin: EntityCenter(Primary), motion: Radial { range_tiles: 1.5, ms: 200 }),
        SpawnParticle(name: "baby_burner_impact", origin: EntityCenter(Primary), motion: Static),
        SpawnParticle(name: "baby_burner_splash", origin: EntityCenter(AdjLeft),  motion: Static),
        SpawnParticle(name: "baby_burner_splash", origin: EntityCenter(AdjRight), motion: Static),
        Shake(intensity: 4, duration_ms: 200),
    ],
)
```

Il blueprint pre-computa `splash_mul_selected = if last_qte_result=="success" { ult_splash_mul_boosted } else { ult_splash_mul }` al `Resume`. **No DSL inline, no branch nel verbo Command** — la scelta è blueprint-side.

**Out-of-scope V1** (rimandato post-M017): mash QTE, timing rings, multi-input chord, perfect/good/fail tiers. Quando arrivano, si aggiunge un nuovo `kind` allo schema `StartQTE` (es. `"PowerCharge"`) senza rompere `HitCheck`.

Doc §2.2b §H esempio dedicato "QTE windup HitCheck" con timeline frame.

### G13 — Heated cap canon = 6 ✅

**Decisione canon (vedi 00§5, 02§7 open):** `Heated` ha cap a **6 stacks per-target**. Apply oltre cap → no-op silente (no error, no overflow). Detonate (`OnKill→ReactiveDetonate`) consuma tutti gli stack residui in un'unica risoluzione (`EventPayload("heated_remaining")` cap-naturale a 6). Decay: -1/turno (TBD M018) — non blocker per M017 stress test.

### G12 — Modifier-firma `OnKill→Detonate` come default ✅

Canon §8 conferma: **sempre attivo** come default behaviour della Ult, parte della FSM base. Edge A `KernelEvent(UnitDied)` esiste **sempre** in `baby_burner.fsm`. Quando arriverà `skill_tree.ron` (post-M017), il modifier diventa overlay opzionale ma di default ON; lo skill tree può solo aggiungere upgrade (es. "Detonate also stuns adj"), non rimuovere l'edge base.
