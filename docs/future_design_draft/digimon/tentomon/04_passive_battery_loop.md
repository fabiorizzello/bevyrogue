# Tentomon — Passive: `battery_loop` (Full FSM + listener, sub-variant B: Reactive-proc)

> **Goal**: passive **esistente** (`src/combat/blueprints/tentomon.rs::battery_loop`) con override identity §4: **+20% block reaction chance** (tank-lite hook).
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Sub-variant **B — Reactive-proc** dominante: edge gating su `IncomingDamage` + SP threshold + RNG roll → transient `BlockProc` node con clip override. SP-grant feedback path resta side-channel listener-only (`02-02e §A.1` boundary note: una passive può mixare sub-variant tra canali — qui mixate tra FSM vs listener-only emit).
>
> **Gap §2.2b condivisi:** dual-role (agumon/04), buff/cleanse interactions. Qui solo nuovi.

## §1 — Intent

> **Canon-source map (round-3, 2026-05-12, X10 consolidation):** questo doc è **fonte canonica unica** per il pattern Block Reaction. Riferimenti:
> - **Command `BlockReaction`** (firma + Args): `02-02b §C2` (riga aggiunta X10) + `agumon/04 §9 G-Verbs` (`BlockReactionArgs` struct).
> - **Event `BlockReactionTriggered`**: `02-02b §R-Events`.
> - **FSM topology** `Dormant/BlockReady/BlockProc`: §1.5 qui sotto (canonico).
> - **Damage pipeline ordering** (block reaction pre-DR + flinch override): §4 qui sotto (canonico).
> - **VFX channels** (Ch1 trigger-proc + Ch2 persistent-presence): §4b qui sotto (canonico).
> - **Identity-level summary**: `tentomon/00 §1`/`§4` (override path B) + `08 §8.3 Tentomon` (high-level kit).
> - **Passive sub-variant categorization**: `02-02e §A.1` row B Reactive-proc (mention only — examples include `fur_cloak` Ch1 e `battery_loop` Ch1).
>
> Niente duplicazione di regole stack-with-DR, flinch override, RNG seed source — risiedono qui in §4. Altri doc cross-ref-ano, NON ridefiniscono.

Battery loop esistente: reactive SP feedback su eventi team. Override: amplia la chance di **block reaction** quando subisce damage (anim clip `block`, frames 9–13).

- **Existing path (side-channel):** SP grant reattiva quando spese SP team. Resta listener-only emit (`ctx.emit_kernel_effect(EmitSpGrant{...})` + `ctx.notify`), niente transizione FSM dedicata. Vedi §1.5 note.
- **Override path (FSM-driven):** FSM `battery_loop_fsm` modella `Dormant ↔ BlockReady ↔ BlockProc`. Edge gating su `IncomingDamage` + SP threshold + RNG roll.

## §1.5 — FSM topology (Full FSM mandate)

Sub-variant **B — Reactive-proc** (block-reaction transient). Il listener osserva `SpEarned`/`SpSpent`/`IncomingDamage`, ricalcola SP threshold predicate, pusha signal nell'FSM.

```ron
// Pseudocode FSM (target: src/combat/blueprints/tentomon.rs::battery_loop_fsm)
PassiveFsm {
    initial: Dormant,
    nodes: [
        Node {
            id: Dormant,
            clip: ("idle", 0..3),
            on_enter: [],                                 // sp < 3, no block-ready
        },
        Node {
            id: BlockReady,
            clip: ("idle", 4..7),
            on_enter: [
                SpawnParticle { preset:"battery_ready_loop_arm",
                                origin: SelfCenter, motion: Static },
            ],
        },
        Node {
            id: BlockProc,
            clip: ("block", 0..4),                        // override clip (5f @12fps ~0.4s)
            on_enter: [
                BlockReaction { kind: All, target_ref: Self_, damage_mult: 0.50 },
                SpawnParticle { preset:"static_shield", origin: SelfCenter, motion: Static },
            ],
        },
    ],
    edges: [
        // Dormant → BlockReady: SP self attraversa soglia 3
        Edge { from: Dormant, to: BlockReady,
               on: KernelEvent(SpEarned | SpSpent),
               predicate: SpPctAtLeast { target_ref: Self_, sp: 3 } },
        // BlockReady → Dormant: SP scende sotto soglia (es. self ha speso)
        Edge { from: BlockReady, to: Dormant,
               on: KernelEvent(SpEarned | SpSpent),
               predicate: Not(SpPctAtLeast { target_ref: Self_, sp: 3 }) },
        // BlockReady → BlockProc: incoming damage + roll RNG success
        Edge { from: BlockReady, to: BlockProc,
               on: KernelEvent(IncomingDamage { target_is_self: true }),
               predicate: RngRollBelow {
                   chance_param: Snapshot("block_chance_base_plus_20pp"),
                   rng_source: TurnRng } },
        // BlockProc → BlockReady: clip block consumata
        Edge { from: BlockProc, to: BlockReady, on: TimeInNode(5) },
    ],
}
```

**SP-grant side-channel (no FSM):** il listener `on_kernel_event(SpEarned { actor: ally })` emette direttamente:
- `ctx.emit_kernel_effect(EmitSpGrant { amount_param: Snapshot("battery_grant_amount"), target_ref: Self_ })` (gameplay).
- `ctx.notify(NotifyParticle { preset:"battery_pulse", origin: SelfCenter, motion: Static })` (Ch1 cosmetic).
- `ctx.notify(NotifyParticle { preset:"battery_link_zap", origin: SelfCenter, motion: Travel{to: EntityCenter(EventTarget), ease: EaseOut, ms: 150} })` (Ch1 optional).

Boundary `02-02e §A.1`: la FSM modella **il gameplay state** (block-ready/block-proc); side-channel SP-grant è puro listener `notify` + kernel emit, niente state transition. Una passive può mixare FSM + listener-only.

**Edge predicate semantica:**
- `SpPctAtLeast { sp: 3 }`: predicato sul valore SP pool corrente di self. Read-only.
- `RngRollBelow { chance_param: ..., rng_source: TurnRng }`: roll deterministico via `TurnRng` seedato dal turn counter (§9 G-Sel). `chance_param` legge `block_chance_base + 0.20` (base ~10% + override 20pp).

**Stack con DR esistenti (ordering preservato §4):**
1. BlockReaction node `on_enter` → damage × 0.50 (pre-DR).
2. DR buff (es. `holy_aegis`) si applica downstream → damage × (1 − DR_total).
3. Final damage delivered + `BlockReactionTriggered` event emesso (chiude B9, vedi §5).

**Channel mapping (`02-02e §A.1`):**
- **Ch1 (trigger-proc):** `battery_ready_loop_arm` su BlockReady `on_enter`, `static_shield` su BlockProc `on_enter`. SP-grant side-channel aggiunge `battery_pulse` + `battery_link_zap` via `ctx.notify`.
- **Ch2 (persistent-presence):** presentation observer su `Added<BuffComponent_BlockReady>` / `RemovedComponents<...>` → `battery_ready_loop` aura. Il marker è inserito/rimosso dal kernel quando l'FSM entra/esce da `BlockReady` (vedi B8 chiusura §5).

**Headless determinism:** FSM tickabile headless. `BlockReaction` Command applica `damage_mult: 0.50` nella damage pipeline; `SpawnParticle` no-op. `RngRollBelow` usa `TurnRng` seedato (deterministico, test-friendly). Clip override `block` è solo presentation — headless skip.

## §2 — Blueprint contract (override only)

```rust
impl BlueprintListener for TentomonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        // existing battery_loop logic preserved...

        // override tank-lite
        match ev {
            CombatEvent::IncomingDamage { target, .. }
                if ctx.is_self(target) && ctx.self_sp() >= 3 => {
                // bumpa block reaction probability di +20pp (additivo a base ~10%)
                let roll = ctx.combat_rng().roll();
                if roll < ctx.block_chance_self() + 0.20 {
                    ctx.emit_kernel_effect(KernelEffect::BlockReaction {
                        actor: ctx.self_id(),
                        damage_mult: 0.50,
                    });
                }
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
nemico colpisce Tentomon → IncomingDamageRequest (pre-resolution)
  └─ battery_loop listener: SP self ≥3? sì → roll RNG → reaction triggered (es)
     └─ KernelEffect::BlockReaction(Tentomon, mult:0.50)
        └─ damage pipeline applica × 0.50
        └─ presentation: FSM Tentomon entra in `block` clip (5f)

durante combat: alleato spende SP
  └─ existing battery_loop path: emit SP grant a self (logica esistente, non modificata)
```

## §4 — Trigger filter precision

- **Self check:** strict `target == Tentomon`. No proc su altri.
- **SP gate:** `self_sp() >= 3`. Sotto, niente bump.
- **RNG:** deterministico via `ctx.combat_rng()` (seeded), test-friendly.
- **Stack con buff DR esistenti:** Block reaction è **separato** dal damage-pipeline DR (es. `holy_aegis`, `dr_self`). Ordine:
  1. Block reaction triggera → damage × 0.50 (pre-DR).
  2. DR buff applica → damage × (1 − DR_total).
  3. Final damage delivered.

## §4b — Anim hook (block clip)

L'FSM `battery_loop_fsm` (§1.5) modella `Dormant/BlockReady/BlockProc`. Il node `BlockProc` ha clip override (`block`, 0..4 = 5f @12fps) sopra al `idle` layer — preempts `hurt` flinch quando la reaction triggera:

```
Signal: KernelEffect::BlockReaction { actor: self, damage_mult: 0.50 }
  └─ AnimPlayer.play_one_shot(clip:"block", count:5f, ~0.4s @12fps)
     ├─ Origin: self
     ├─ Layer: above idle, preempts hurt clip (reaction wins over flinch)
     └─ Fallback completion → idle
```

`block` clip (5f): chitin guard sideways + wings fold. Match canon-flavor (Rolling Guard 374 "curls in ball to raise defense" — semantica leggermente diversa ma più vicino disponibile).

**Headless:** clip override skip (presentation-only).

## §4c — VFX (Ch1 + Ch2, `02-02e §A.1` sub-variant B Reactive-proc)

VFX viaggiano su due canali ortogonali: FSM `on_enter` `SpawnParticle` (override path BlockReady/BlockProc — vedi §1.5) per gli effetti gameplay-bound, side-channel listener `notify` per il path SP-grant non-FSM:

- **Channel 1** = `ListenerCtx::notify(NotifyParticle)` per one-shot event-bound (§2.2e §C).
- **Channel 2** = presentation observer su component diff (`Added`/`Removed`) per persistent state-bound (§2.2e §D).

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| 1 | **SP grant proc** (existing battery path) | 1 | `on_kernel_event(SpEarned { source: "battery_loop", actor: ally })` | `battery_pulse` (brief yellow shimmer) | `SelfCenter` (Tentomon) | `Static` |
| 2 | **SP grant link** *(opt.)* | 1 | stesso trigger di #1 | `battery_link_zap` (short bolt) | `SelfCenter` (Tentomon) | `Travel { to: EntityCenter(EventTarget), ease: EaseOut, ms: 150 }` |
| 3 | **Block-ready aura (entry)** | 2 | `Added<BuffComponent_BlockReady>` su Tentomon | `battery_ready_loop` (subtle pulsing yellow halo) | `SelfCenter` | `Static` |
| 4 | **Block reaction flash** | 1 | `on_kernel_event(BlockReactionTriggered { actor: self, .. })` | `static_shield` (short chitin-spark burst) | `SelfCenter` | `Static` |
| 5 | **Block-ready aura despawn** | 2 | `RemovedComponents<BuffComponent_BlockReady>` | — (manager despawned) | — | — |

### Note implementative

- **`BuffComponent_BlockReady`** — indicator-component non-buff (vedi §5 gap B8). Inserito/rimosso dal kernel SP-watcher quando `Tentomon.sp` attraversa la soglia `>=3 / <3`. Pure marker presentation, non c'è `BuffId` stringa equivalente (gameplay non lo ignora — non esiste lato gameplay).
- **Riga 1+2** sono entrambe **Channel 1** e ricevono `EventTarget` (l'alleato che ha speso SP) tramite `EntityRef::EventTarget` (§2.2e §C tabella, listener-side only).
- **Riga 2 opzionale:** se il link visivo è troppo rumoroso con SP frequenti, droppare e tenere solo riga 1.
- **Riga 4** richiede `CombatEvent::BlockReactionTriggered` (vedi §5 gap B9). Oggi c'è solo `KernelEffect::BlockReaction` Command, non l'event speculare consumato dal presentation observer.
- **Headless:** Channel 1 = `notify` no-op se `cfg(not(feature = "windowed"))`. Channel 2 = system gated dietro feature flag, mai schedulato in headless.

## §5 — Open questions (nuovi)

1. **B4 — `CombatEvent::IncomingDamage` esiste?** È un evento **pre-damage**. Verificare `src/combat/events.rs`. Se solo `DamageDealt` (post-fact), serve aggiungere `IncomingDamage` come pre-step. Action item §2.8 (cascade) lo abilita.
2. **B5 — Block reaction emessa via `KernelEffect::BlockReaction` o nuovo verbo?**
   - **A.** Verbo nuovo `BlockReaction { actor, damage_mult }` nel kernel effect set.
   - **B.** Buff temporaneo `dr_block` applicato a self con dur=this-damage-only.
   - **Decisione consigliata:** A. Reaction è effetto puntuale, non state che persiste.
3. **B6 — Block clip FSM trigger.** Quando reaction triggera, l'anim `block` (5f) entra. Chi orchestra?
   - **Proposta:** kernel emette `BlockReactionTriggered` event → presentation listener fa play della clip. Headless lo droppa (cosmetic).
4. **B7 — Battery loop existing logic verifica.** Identity §4 dice "esistente". `src/combat/blueprints/tentomon.rs` ha già il path SP grant; verificare se il design corrente coincide con l'identity sheet (mancano dettagli granulari sul trigger esatto). **Action item:** allineare doc con codice o codice con doc, decidere fonte di verità prima M017.
5. **B8 — Indicator-component senza `BuffId` (presentation-only marker).** ✅ **Chiuso (round-3, 2026-05-12):** decisione **A** formalizzata via Full FSM mandate (`02-02e §A.0`). Il marker `BuffComponent_BlockReady` è **derivato dall'FSM node**: kernel inserisce il componente su `on_enter(BlockReady)` (via `Commands.entity(...).insert(...)` lato kernel infra, non via `ApplyBuff` Command), rimuove su `on_exit`. Convenzione §2.2e §E estesa: **prefix `Buff_*`** per `BuffId`-mirror typed components (gameplay-bound); **prefix `BuffComponent_*`** per FSM-node-derived markers (presentation-bound only, no `BuffId` corrispondente). Codifica naming canon in `02-02e §E` quando si chiude il giro doc-side.
6. **B9 — `CombatEvent::BlockReactionTriggered` non esiste.** ✅ **Chiuso (round-3, 2026-05-12):** event speculare formalizzato in `02-02b §R-Events`. Il `BlockReaction` Command emesso da `BlockProc.on_enter` ha pattern Command→Event canon (cfr. `ApplyBuff` → `BuffApplied`): kernel applica `damage_mult` e emette `CombatEvent::BlockReactionTriggered { actor, damage_mult, attacker }`. Presentation observer Channel 1 consuma l'event via `ctx.notify` lato listener. Patch `02-02b §R-Events` da finalizzare doc-side (action item permanente, non blocca M017 — l'event può essere aggiunto incrementalmente). Specchio gameplay-side già coperto: `agumon/04` §9 G-Verbs ha `BlockReaction` Command, l'event downstream è il pendant naturale.

## §6 — Verdetto

Override tank-lite + Full FSM mandate (`02-02e §A.0`) consolidano:
- **FSM 3 nodi** (Dormant/BlockReady/BlockProc) + 4 edge ispezionabili editor-side, sub-variant B Reactive-proc.
- **Side-channel SP-grant** resta listener-only `notify` + kernel emit, niente FSM transition — coerente con `02-02e §A.1` boundary note (mix sub-variant intra-passive).
- **Evento `IncomingDamage` pre-step** nel cascade pipeline (B4, gap §2.8).
- **Verbo `BlockReaction`** come kernel effect (B5).
- **Event `BlockReactionTriggered`** ✅ chiuso (B9, formalizzato `02-02b §R-Events`).
- **Marker `BuffComponent_*`** ✅ chiuso (B8, decisione A formalizzata `02-02e §E`).
- **RNG seeded shared** tra blueprint (D1 di tentomon/03 e B4 qui).

Battery loop existing logic resta intoccato; il Full FSM modella **l'override path** (block-react state machine), il side-channel SP-grant resta listener-only. Pattern dual-path FSM+listener generalizzabile (cfr. Gabumon `fur_cloak`+`twin_core_ice` con due FSM paralleli).
