# Agumon — Ult: `nova_blast` (FSM stress test)

> **Goal**: stressare il caso più complesso del kit Agumon — Ult con **edge reattivo** (`OnKill → ReactiveDetonate`), **QTE Power Charge** durante windup, **splash AoE secondario**, e modifier-firma `Detonate(Heated)`.

## §1 — Intent

- **Cost:** 0 SP, **consuma full ult bar** (`ultimate_trigger=100` raggiunto). Non charga sé stesso. **Lanciabile anytime off-turn** (HSR-style) — non occupa lo slot azione del turno.
- **Effect base:** Damage Fire alto `50` su primary; splash 50% sui 2 adj (Blast shape).
- **Modifier-firma:** `OnKill→Detonate(Heated)` — se il primary muore dall'Ult hit, **gli stack rimanenti di `Heated`** del primary vengono detonati come damage aggiuntivo sui 2 adj.
- **QTE:** Power Charge (mash durante Windup → aumenta blast radius / damage splash).
- **Atlas clip:** `skill` (source frames 50–66, count 17)

## §2 — FSM topology (4-node con edge reattivo + QTE)

```
                        QTE success → boost_radius
        ┌──────────┐                           ┌──────────┐
commit→ │  Windup  │ ──StartQTE→ Suspend  ──── │  Strike  │
        │ frames:5 │     |                     │ frames:4 │
        └──────────┘     │ YieldResolved       └────┬─────┘
                          │ (UserInput/headless)    │
                          ▼                         │ on_enter:
                  (resume Strike)                   │  EmitDamage(primary, hits:1, mul:"ult_mul", tough_break:30)
                                                    │  EmitDamage(adj_l, hits:1, mul:"ult_splash_mul")
                                                    │  EmitDamage(adj_r, hits:1, mul:"ult_splash_mul")
                                                    │  EmitStatus(Heated, target:Primary, stacks:1) ← optional
                                                    │  SpawnParticle("nova_burst","primary_pivot")
                                                    │  Shake { intensity:4, duration_ms:200 }
                                                    │
                                                    ├── edge A: KernelEvent(UnitDied {target: primary}) prio:10
                                                    │            ──▶ ReactiveDetonate
                                                    │
                                                    └── edge B: TimeInNode prio:0
                                                                 ──▶ Recovery
        ┌────────────────────┐                     ┌────────────┐
        │ ReactiveDetonate    │ TimeInNode(2)      │  Recovery  │
        │ frames: 3           │ ──────────────────▶│ frames: 5  │ ──▶ exit
        └────┬────────────────┘                    └────────────┘
             │ on_enter:
             │  EmitDamage(adj_l, hits:1, mul:"detonate_per_stack", stacks_ref:"heated_consumed")
             │  EmitDamage(adj_r, hits:1, mul:"detonate_per_stack", stacks_ref:"heated_consumed")
             │  SpawnParticle("heated_detonate","primary_pivot")
             │  Shake { intensity:3, duration_ms:160 }
```

## §3 — Nodes table

| Node | frames | atlas src range | ms (@12fps ref) | Note |
|---|---|---|---|---|
| `Windup` | 5 | 50–54 | 0–416 | apertura, occhi che brillano. **`on_enter: StartQTE { kind:"PowerCharge", window_param:"qte_window", headless_default_param:"qte_default_headless" }`** |
| `Strike` | 4 | 55–58 | 416–750 | il blast vero. 3-4 emit, vedi §2 |
| `ReactiveDetonate` | 3 | 59–61 | 750–1000 | **solo se edge A taken**. Splash secondario da stacks Heated rimasti |
| `Recovery` | 5 | 62–66 | 1000–1416 | postura ritorno (anche se ReactiveDetonate fired, recovery riprende dopo) |

**Frame budget se no detonate:** 5+4+5 = 14 frames (≠ atlas 17). **3 frames "orfani"** (62-66 → 62-64 da skippare). Vedi §6.4.

**Frame budget se detonate:** 5+4+3+5 = 17 frames. ✅ matcha atlas.

## §4 — Param resolution snapshot-once (al commit)

| param | source | example |
|---|---|---|
| `ult_mul` | ATK + ult bonus | `50` legacy |
| `ult_splash_mul` | derived 50% di `ult_mul` | `25` |
| `detonate_per_stack` | TBD design (proposta `8` per stack) | scalar per `heated_consumed` |
| `heated_consumed` | **live read on `ReactiveDetonate.on_enter`** (NON snapshot — è il count residuo al momento della morte) | 0..6 |
| `qte_window` | `skills.ron.params["qte_window"]` | 800ms metadata UI |
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

6. **QTE windup window vs animation frames.** `Windup` dura 5 frames (~416ms @12fps). QTE `window_param: 800ms` è **più lungo** del nodo. Soluzioni:
   - QTE può sospendere il playhead (=`Suspend` mette in pausa FSM frame counter). Sì, è quello che §2.6 fa. ✓ OK ma confermare: il QTE estende `Windup` o sospende prima di entrare in `Strike`?
   - **Decisione:** `StartQTE` emesso `on_enter Windup` → FSM yield → user input arriva → `YieldResolved` resumes → playhead riparte dove era (frame `Windup.start+0`?). **Vedi §H per il contratto preciso.**

7. **`StartQTE` in headless test.** Test headless determinico richiede `qte_default_headless`. ✓ §G. Ma **dove è memorizzato?** In `skills.ron.params`? Sì. **Action item:** estendere schema con `params: { qte_default_headless: "success" }`.

8. **Modifier-firma "OnKill→Detonate(Heated)" è una skill-tree variant?** §8 roster minimal lo cita come "modifier-firma" della Ult. Nel design §I (skill_tree.ron file separato), questo modifier sarebbe definito **dentro skill_tree.ron** o come **default Ult node behavior**? Senza skill_tree (in M017), il modifier è inline nella FSM (edge A). Quando skill_tree arriva, il modifier diventa un overlay che attiva/disattiva l'edge A. **Quesito design:** la FSM base ha l'edge sempre o solo con unlock? **Proposta canon §8:** sempre attivo (è la modifier-firma di default, non opzione). Confermare.

### 🟡 Aperte (non blocker M017)

- Power Charge QTE → boost radius: implementa via `EmitDamage { mul_param }` letto come `"ult_splash_mul_boosted"` se QTE success, `"ult_splash_mul"` se fail. Risoluzione param dipende da `last_qte_result`. Stessa famiglia di gap §1.
- Heated cap discusso in 02 (proposta 6). Detonate consuma tutti gli stack rimasti.

## §7 — Verdetto

Ult espone 5 gap architetturali (1, 2, 3, 4, 6) di cui **2 nuovi** non emersi in basic/skill:
- (1) Param source kind: `Snapshot` vs `EventPayload` (vocabolario param map a 2 cluster)
- (3) Multi-target damage: 3 Commands separate vs targeting nel verbo
- (4) Frame budget mismatch per path optional (atlas vs FSM)
- (6) QTE timing che eccede il nodo emittente

Gap (2) si chiarisce. Gap (5) è game-design, non FSM.

**Tutti risolvibili nel framework §2.2b senza inventare un nuovo modello.** L'Ult è il **caso d'uso più forte** per la FSM contro la lista piatta — l'edge reattivo `OnKill` è esattamente ciò che la lista non può fare.
