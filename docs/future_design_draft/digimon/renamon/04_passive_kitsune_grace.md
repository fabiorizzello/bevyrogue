# Renamon — Passive: `kitsune_grace` (Full FSM + listener, sub-variant B: Reactive-proc)

> **Goal**: passive reattiva a **evento alleato** (`UltimateUsed` by ally). Primo listener cross-team che modifica `TurnOrder` di self via Command FSM-side.
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Sub-variant **B — Reactive-proc** standard (`Dormant → Proc → Resolve → Dormant`). Niente state hold persistente — solo edge transient — quindi **Ch1 mandatory, Ch2 omesso** per design (no aura semantically applicable). Coerente con `02-02e §E.1` (Reactive-proc: Ch1 mandatory, Ch2 optional).
>
> **Gap §2.2b condivisi:** dual-role (agumon/04), time-manip T1 (02_skill_koyosetsu.md). Qui nuovi.

## §1 — Intent

- **Trigger:** un alleato (qualsiasi, escluso Renamon stessa — vedi §5 K1) consuma Ult.
- **Effect:** `AdvanceTurn(self, 10%)` — Renamon avanza il proprio gauge.
- **Scope:** **Full FSM Reactive-proc** (vedi §1.5). Anim layer rimane `idle` — i frame range per nodo sono partizioni dello stesso loop idle per editor-inspectability.

## §1.5 — FSM topology (Full FSM mandate)

Sub-variant **B — Reactive-proc** classico (`Dormant → Proc → Resolve → Dormant`). Listener osserva `UltimateUsed { actor }`, valuta predicate (`is_ally && !is_self`), pusha signal nell'FSM.

```ron
// Pseudocode FSM (target: src/combat/blueprints/renamon.rs::kitsune_grace_fsm)
PassiveFsm {
    initial: Dormant,
    nodes: [
        Node {
            id: Dormant,
            clip: ("idle", 0..3),
            on_enter: [],
        },
        Node {
            id: Proc,
            clip: ("idle", 4..7),
            on_enter: [
                AdvanceTurn { target_ref: Self_, amount: 10 /* pct */ },
                SpawnParticle { preset:"kitsune_grace_flash",
                                origin: SelfCenter, motion: Static },
                SpawnParticle { preset:"kitsune_grace_link",
                                origin: EntityCenter(EventTarget),     // ally caster
                                motion: Travel { to: SelfCenter,        // "ruba tempo"
                                                 ease: EaseOut, ms: 200 } },
            ],
        },
        Node {
            id: Resolve,
            clip: ("idle", 8..11),
            on_enter: [],                                   // let FX tail out
        },
    ],
    edges: [
        // Dormant → Proc: alleato (non self) consuma Ult
        Edge { from: Dormant, to: Proc,
               on: KernelEvent(UltimateUsed),
               predicate: And(IsAlly(EventActor),
                              Not(IsSelf(EventActor))) },
        // Proc → Resolve: lascia respirare il VFX (3 frame ~ 0.25s @12fps)
        Edge { from: Proc, to: Resolve, on: TimeInNode(3) },
        // Resolve → Dormant: settle back, pronto al prossimo proc
        Edge { from: Resolve, to: Dormant, on: TimeInNode(2) },
    ],
}
```

**Edge predicate semantica:**
- `And(IsAlly(EventActor), Not(IsSelf(EventActor)))`: filter su `UltimateUsed.actor`. Stack additivo nel turno (Patamon ult + Agumon ult = 2× edge fires consecutivi → 2× passaggio Dormant→Proc→Resolve→Dormant in stessa frame window).
- Cap globale ±50% gestito a livello `AdvanceTurn` Command (clamp nel resolver), non a livello edge — l'FSM può procare N volte, il clamp gameplay-side limita l'effetto cumulativo.

**Channel mapping (`02-02e §A.1` Reactive-proc):**
- **Ch1 mandatory:** `kitsune_grace_flash` (golden chime burst su self) + `kitsune_grace_link` (chime arc travel **self-ward** dall'ally caster — semantica "ruba tempo", opposto ai link Travel outward standard). Entrambi `on_enter(Proc)`.
- **Ch2 omesso:** no buff persistente, no aura. Sub-variant B con Ch2 optional non instanziato. `02-02e §E.1` lo ammette esplicitamente ("Reactive-proc: Ch2 optional"). **K5 chiusura formale (vedi §5).**

**Headless determinism:** FSM tickabile headless. `AdvanceTurn` Command modifica il `TurnGauge` di self direttamente in pipeline gameplay; `SpawnParticle` no-op. Test integration osservano `TurnGauge` diff su `UltimateUsed { actor: <ally> }` event injection.

## §2 — Blueprint contract

```rust
impl BlueprintListener for RenamonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            CombatEvent::UltimateUsed { actor }
                if ctx.is_ally(actor) && !ctx.is_self(actor) => {
                ctx.emit_kernel_effect(KernelEffect::AdvanceTurn {
                    actor: ctx.self_id(),
                    pct: 10,
                });
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
turno T: Agumon casta baby_burner → CombatEvent::UltimateUsed { actor:Agumon }
  └─ Renamon listener: actor != self → trigger
     └─ KernelEffect::AdvanceTurn(Renamon, 10%)
        └─ TurnGauge Renamon −10%

turno successivo: gauge Renamon ridotto → agisce prima
  └─ se Renamon Ult: niente self-trigger (vedi §5)
```

## §4 — Trigger filter precision

- **Actor check:** alleato vivo, NON self (escluso per evitare loop).
- **Event:** `UltimateUsed` (verificare esiste in `src/combat/events.rs`. Probabilmente `CombatEventKind::UltimateUsed` o derivato da `UltimateCharge` consumption. **Action item se mancante:** aggiungere evento canonico `UltimateUsed { actor }` dopo consumo.
- **Cap:** stack additivo nel turno (Patamon ult + Agumon ult = 20% advance). Identity §5 dice cap ±50% per effetto, stack additivo clamp `[0, 200%]`.

## §4b — VFX (Ch1 only, `02-02e §A.1` sub-variant B Reactive-proc)

`kitsune_grace` è Reactive-proc puro: edge-event (`UltimateUsed` by ally) senza stato persistente lato Renamon. Nessun `Buff_*` da osservare via `Added/Removed`. **Ch2 omesso** (optional per Reactive-proc, `02-02e §E.1`). Tutti i VFX viaggiano su Ch1 via FSM `on_enter(Proc)` Commands (vedi §1.5) — gli `ctx.notify` mostrati sotto sono il path equivalente per cosmetic-only emits che NON pilotano transizioni FSM (es. flash decorativo aggiuntivo). Il path canonico per `kitsune_grace_flash` + `kitsune_grace_link` è ora **FSM Command-side** (§1.5), non listener-notify.

| Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|
| **Grace flash** | 1 | `ctx.notify` su `CombatEvent::UltimateUsed { actor }`, filter `is_ally(actor) && !is_self(actor)` | `kitsune_grace_flash` (golden chime burst) | `SelfCenter` (Renamon) | `Static` |
| **Time-link** *(opt.)* | 1 | stesso trigger sopra | `kitsune_grace_link` (chime arc, "ruba tempo") | `EntityCenter(EventTarget)` (ally caster) | `Travel { to: EntityCenter(Self), ease: EaseOut, ms: 200 }` |

**Note implementative:**

- **Niente Channel 2.** L'effetto è istantaneo (single `AdvanceTurn` kernel effect emesso da `on_enter(Proc)`). Nessuna aura persistente, nessun `Added/Removed` da osservare. Ch2 optional per sub-variant B Reactive-proc — non instanziato qui. K5 ✅ chiuso (vedi sotto).
- **`EntityRef::EventTarget`** mappa l'`actor` del `CombatEvent::UltimateUsed` → ally caster. Risolvibile nell'`on_enter(Proc)` dell'FSM perché il nodo è raggiunto via edge `KernelEvent(UltimateUsed)` — l'event payload è accessibile durante la transizione. Coerente con `02-02e §C` extension a passive FSM (event-scoped Commands su edge `KernelEvent(...)`).
- **Travel self-ward.** Direzione `→ SelfCenter` (verso Renamon), opposto ai Travel link precedenti (Tentomon battery side-channel, Gabumon twin_core, che vanno outward). Semantica directional intenzionale: "Renamon ruba tempo dal caster". Niente nuovo verbo — `VfxLocus::SelfCenter` esiste.
- **Headless gating.** FSM tickabile headless (`tick_passive_fsm`), `AdvanceTurn` Command emit gameplay-side identico; `SpawnParticle` no-op. Nessun bisogno di `cfg(feature = "windowed")` lato listener — l'intera logica vive nell'FSM uniform headless/windowed.

## §5 — Open questions (nuovi)

1. **K1 — Self-Ult triggera kitsune_grace?** ✅ **Chiusa (round-3, 2026-05-12, X12): opzione B `no-self`.**
   - Filter canonico: `!ctx.is_self(actor)`. Self-Ult (`tohakken`) **non triggera** la passive.
   - Rationale: vedi `renamon/00 §8 D6`. UX più leggibile ("ruba tempo dai compagni", non self-feedback). Loop teorico evitato a monte del clamp `±50%/call`.
   - Conforme a `§2` blueprint contract (`ev != self` predicate) e `§1.5` FSM edge (`And(IsAlly(EventActor), Not(IsSelf(EventActor)))`).
2. **K2 — `UltimateUsed` event quando è emesso?** Al `commit_action(Ult)` o a `Ult.Strike.on_enter` (consumo bar)? **Coerenza:** allineare a "consumo bar" → solo se l'Ult andata davvero. Cancellazioni (es. invalid target) non triggerano.
3. **K3 — Bound check.** Cap 50% per call (T1). 10% × 5 alleati Ult in stesso round → 50%, ok. Reale: 1-2 ult/round max. Safe.
4. **K4 — Compatibilità con `Blessed`.** Blessed (03_ult_tohakken.md) buffa damage e Ult charge gen. Niente double-dip con `kitsune_grace`: K1 reagisce all'**uso** dell'ult, non al **charge**. Distinto.
5. **K5 — Forma C single-channel variant.** ✅ **Chiuso (round-3, 2026-05-12):** la framing "Forma C variants by shape" è stata **abbandonata** in `02-02e §A.0` (Full FSM mandate v2) e sostituita dai **trigger sub-variant A/B/C** (`02-02e §A.1`). `kitsune_grace` è ora classificato come sub-variant **B — Reactive-proc** con **Ch1 mandatory + Ch2 optional** (codificato `02-02e §E.1`). Nessun "Forma C single-channel" speciale serve: la mancanza di Ch2 è la condizione default di Reactive-proc senza armed-state hold. Topology FSM uniforme con le altre passive (3+ nodi, edge, clip frame range — vedi §1.5).
6. **K6 — Travel self-ward semantica directional.** Il `kitsune_grace_link` punta `to: EntityCenter(Self)` invece di `EntityCenter(EventTarget)`. Niente nuovo verbo grammatica (`EntityRef::Self` già definito). Solo nota convenzionale: la direction del Travel è **semantically meaningful** ("steal" vs "grant") e i preset designer devono poterlo scegliere liberamente. Annotare in §2.2d esempi: i Travel link cross-unit non sono sempre outward dal caster.

## §6 — Verdetto

`kitsune_grace` consolida:
- **Event canonico `UltimateUsed`** (verificare o aggiungere `02-02b §R-Events`).
- **FSM emette `AdvanceTurn` Command** — primo caso di passive FSM che produce gameplay Command non-buff (modifica direttamente `TurnGauge`).
- **Full FSM Reactive-proc senza Ch2** — sub-variant B standard, Ch1 mandatory + Ch2 optional non instanziato (`02-02e §E.1`). K5 ✅ chiuso (Forma C abbandonata in favore di sub-variant trigger framework).

Pattern: FSM passiva può **emit kernel effect arbitrario** via Commands `on_enter`, non solo applicare buff. Generalizzazione utile per altre passive future (es. cleanse-on-event, advance-others, status-propagation).
