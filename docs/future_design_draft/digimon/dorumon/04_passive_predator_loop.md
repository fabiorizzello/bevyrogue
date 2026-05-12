# Dorumon — Passive: `predator_loop` (Full FSM + listener, sub-variant C: State-watch)

> **Goal**: passive **già implementata** (`src/combat/blueprints/dorumon.rs::PredatorLoopState`, `PredatorLoopResolved` event). Allineamento del design doc al codice esistente; identificare gap se l'identity sheet diverge dal comportamento attuale.
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Sub-variant **C — State-watch** dominante (hp-threshold predicate `tracked_target.hp_pct < threshold`), con un sotto-blocco di **A — Aura-loop** sul tracking continuo (idle scan) e edge transient di **B — Reactive-proc** sul chain consume. §A.1 boundary note: una passive può mixare sub-variant tra canali — qui mixate tra nodi.
>
> **Gap §2.2b condivisi:** dual-role (agumon/04). Memory note: "PredatorLoopState must explicitly track a target before Dorumon transitions are emitted in a headless runtime test". Qui solo nuovi gap.

## §1 — Intent

- **Tracking:** scan continuo lowest-HP% enemy alive; aggiorna `tracked_target`. **FSM node `IdleScan`.**
- **Entry:** quando `tracked_target.hp_pct < threshold` → `predator_active = true` per N turni. **Edge `IdleScan → Armed`.**
- **Exit:** `tracked_target` muore (chain consumato in `dash_metal`) o N turni expira. **Edge `Armed → IdleScan`.**
- **Chain consume:** transient `ChainPrimed` node durante `dash_metal` consume. **Edge `Armed → ChainPrimed → IdleScan`.**
- **Effect:** abilita edge A su `dash_metal` (chain on kill); cambia threshold ult bonus a `<30%`.

## §1.5 — FSM topology (Full FSM mandate)

Sub-variant **C — State-watch** principale (hp-threshold predicate). Il listener osserva `DamageDealt`/`UnitDied`/`TurnEnded`, ricomputa `tracked_target` lowest-HP%, mutua `PredatorLoopState`, e pusha signal nell'FSM. Edge consumano signal su `KernelEvent(...)` filtrati per predicate.

```ron
// Pseudocode FSM (target: src/combat/blueprints/dorumon.rs::predator_loop_fsm)
PassiveFsm {
    initial: IdleScan,
    nodes: [
        Node {
            id: IdleScan,
            clip: ("idle", 0..3),
            on_enter: [],                                 // pure tracking, no VFX
        },
        Node {
            id: Armed,
            clip: ("idle", 4..7),
            on_enter: [
                ApplyBuff { id:"predator_active", target_ref: Self_,
                            mul_param: None, dur: Turns(3) /* config.duration_turns */ },
                SpawnParticle { preset:"predator_lock", origin: SelfCenter,
                                motion: Travel { to: EntityCenter(FromBlueprintState(
                                    "predator_loop.tracked_target")),
                                    ease: EaseOut, ms: 150 } },
            ],
        },
        Node {
            id: ChainPrimed,
            clip: ("idle", 8..11),
            on_enter: [],                                 // chain consume VFX coperto da dash_metal FSM
        },
    ],
    edges: [
        // IdleScan → Armed: tracked_target hp_pct sotto soglia
        Edge { from: IdleScan, to: Armed,
               on: KernelEvent(DamageDealt | UnitDied),
               predicate: HpPctBelow {
                   target_ref: FromBlueprintState("predator_loop.tracked_target"),
                   pct: 50 /* config.entry_threshold */ } },
        // Armed → IdleScan: timeout o tracked_target died (no chain)
        Edge { from: Armed, to: IdleScan,
               on: KernelEvent(TurnEnded),
               predicate: BlueprintState { state_key:"predator_loop.expires_in",
                                           expected: Int(0) },
               on_exit: [SpawnParticle { preset:"predator_aura_dissipate",
                                         origin: SelfCenter, motion: Static }] },
        Edge { from: Armed, to: IdleScan,
               on: KernelEvent(UnitDied),
               predicate: BlueprintState { state_key:"predator_loop.tracked_target_died_without_chain",
                                           expected: Bool(true) } },
        // Armed → ChainPrimed: tracked dies during dash_metal active (chain consume)
        Edge { from: Armed, to: ChainPrimed,
               on: KernelEvent(UnitDied),
               predicate: BlueprintState { state_key:"dash_metal_in_progress",
                                           expected: Bool(true) } },
        Edge { from: ChainPrimed, to: IdleScan, on: TimeInNode(1) },
    ],
}
```

**Edge predicate semantica:**
- `HpPctBelow { target_ref: FromBlueprintState("predator_loop.tracked_target"), pct: 50 }`: il `tracked_target` è risolto via blueprint state lookup live (`02-02d §B.1` N8b). `tracked_target = None` → predicate false (no spawn).
- `BlueprintState`: legge stato custom impostato dal listener (es. `expires_in` decrementato a 0 su `TurnEnded`). Read-only, niente side-effect dall'edge eval (§9 G-Pred).
- Force-entry via `metal_cannon` Ult (identity §F5): `SetBlueprintState("predator_loop.predator_active", true)` dal Spit node bypassa edge `IdleScan → Armed`. Listener observa cambio state e pusha signal sintetico nell'FSM (alternativa: emit `predator_force_active` evento kernel, FSM consuma su quello).

**Channel mapping (`02-02e §A.1`):**
- **Ch1 (trigger-proc):** `predator_lock` su Armed `on_enter`, `predator_aura_dissipate` su `on_exit`, `predator_mark_fade` su `UnitDied{tracked}` (vedi §5b mapping). Reactive-proc style.
- **Ch2 (persistent-presence):** **due osservabili**:
  1. `Added/Removed<Buff_PredatorActive>` → `predator_aura_loop` su Dorumon (state-watch).
  2. `Changed<DorumonBlueprint>` → `predator_mark_loop` migrating sul tracked_target via `observe_predator_mark` (template `02-02e §D`). Despawn-respawn (non emitter retarget) quando tracked cambia, perché il preset ha loop state da resettare.

**Headless determinism:** FSM tickabile headless. `ApplyBuff(predator_active)` gira identico; `SpawnParticle` no-op. Listener-side state mutations (`PredatorLoopState.recompute_tracked`, `expires_in` decrement) sono già headless-safe nel codice corrente. Test integration osservano `PredatorLoopResolved` event payload + `predator_active` boolean diff via `tick_passive_fsm` + listener.

## §2 — Blueprint contract

```rust
impl BlueprintListener for DorumonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            // tracking: ogni damage o death, ricomputa lowest
            CombatEvent::DamageDealt { target, .. }
            | CombatEvent::UnitDied { unit: target } => {
                self.state.recompute_tracked(ctx);
                if let Some(t) = self.state.tracked_target {
                    if ctx.unit_hp_pct(t) < self.config.entry_threshold {
                        self.state.predator_active = true;
                        self.state.expires_in = self.config.duration_turns;
                        ctx.emit_kernel_event(CombatEvent::PredatorLoopResolved {
                            tracked: t,
                            active: true,
                        });
                    }
                }
            }
            // turn-end tick: decrementa durata
            CombatEvent::TurnEnded { .. } if self.state.predator_active => {
                self.state.expires_in -= 1;
                if self.state.expires_in == 0 {
                    self.state.predator_active = false;
                }
            }
            _ => {}
        }
    }
}
```

(Pseudocode; reale è in `src/combat/blueprints/dorumon.rs`.)

## §3 — Activation flow

```
nemico subisce damage (qualsiasi caster)
  └─ DamageDealt event
     └─ predator_loop listener: recompute tracked = lowest-HP%
        └─ if tracked.hp_pct < threshold: predator_active = true (N turni)
           └─ emit PredatorLoopResolved event

Dorumon casta dash_metal
  └─ edge A predicate: BlueprintState(predator_active) AND UnitDied(primary)
     └─ se entrambi: ChainStrike fires → consume state

oppure: ult metal_cannon forza state on hit (vedi 03 F5)
```

## §4 — Trigger filter precision

- **Tracking scope:** EnemyTeam alive.
- **Entry:** condition gating, no manual override (eccetto F5 ult force).
- **Exit:** condizioni:
  - `tracked_target.died` ed era il target → reset state e ricalcola lowest.
  - `expires_in == 0` → state off.
  - Manual force off da blueprint? No (decisione: solo timeout o consume).
- **Memory note constraint:** PredatorLoopState **deve** tracciare un target prima di emettere transizioni. Headless test deve setup il tracking explicitly o il kernel rifiuta con `InvalidTarget`.

## §5 — Open questions (nuovi)

1. **G1 — Allineamento doc vs codice.** Identity §5 dice "Predator state" e "Exit: target tracked muore o timeout turni". Verificare in `src/combat/blueprints/dorumon.rs` se entrambe le condizioni di exit sono implementate o solo una. **Action item:** se manca, aggiungere; se non manca, sync identity.
2. **G2 — UI visibility (identity §6).** ✅ **Risolto via §6b VFX:** tracked target è visivamente marcato dal preset `predator_mark_loop` (Channel 2, presentation observer su `PredatorLoopState.tracked_target`). Niente HSR-style debuff badge UI dedicato — il mark VFX **è** la visibility. Hook event `PredatorLoopResolved` resta come signal per logging/debug, non per UI separata.
3. **G3 — Threshold value.** Identity §5 dice "X%". Codice esistente probabilmente ha valore default (es. 50%). Confermare e documentare nel config.
4. **G4 — Force-state via Ult (F5) interagisce con timeout?** Se Ult forza `predator_active=true` con `dur:N`, e tracking lowest-HP% già attivo con `dur:M`, qual è la durata finale? Max-replace? Refresh? **Decisione consigliata:** max(N, M) — più generoso al player.
5. **G5 — Chain interaction con Twin Core / Heated / Chilled.** Metal Cannon interaction (identity §6): "Metal Cannon interaction con Twin Core / status altrui — bonus o trasparente?" **Decisione consigliata:** trasparente. Dark damage non scala su Heated/Chilled di Agumon/Gabumon. Mantiene Dorumon **single-target executor pure**, non status-dipendente. Niente sinergie cross-roster. ✅ **Chiuso in identity §6 D3** (round-3 Dorumon).
6. **G6 — Parametric `EntityRef` variant `FromBlueprintState(key)` + `<iter:loop_var>`.** ✅ **Chiuso (round-3, 2026-05-12):** entrambe le manifestation N8a/N8b sono **formalizzate in `02-02d §B.1`** come terzo/quarto modo di risolvere `EntityRef` oltre a `Primary`/`FromParamSnapshot`/`EventTarget`/`Caster`/`Self`:
   - **N8a — `<iter:loop_var>`** (Renamon `tohakken`, Agumon `baby_burner` splash, Dorumon `dash_metal` non-chain): risolto da **loop expansion** del blueprint resolver al binding time, prima dello spawn del particle.
   - **N8b — `FromBlueprintState(state_key)`** (Dorumon `predator_loop` mark/aura/Travel.to): risolto da **blueprint state lookup live** al spawn time del particle, con re-resolve su `Changed<BlueprintComponent>` per migrating mark (template §2.2e §D `observe_predator_mark`).
   - **Snapshot-once policy (`02-02d §H.4`)** si applica a entrambe per consistency: mark resolved on spawn, mark survives target death con last-known position via despawn-cleanup observer (`02-02e §D`). Mark **re-spawn** (non emitter retarget) quando `tracked_target` cambia — pop animation del preset `predator_mark_loop` va replayed.

## §5b — VFX (Channel 1 + Channel 2, §2.2e)

> No clipmontage, no `SpawnParticle` Command. Predator Loop è listener-only con state machine interna (`PredatorLoopState`). VFX seguono il pattern §2.2e: **Channel 1** per i transition flash (entry, chain consume, dissipate), **Channel 2** per le componenti persistenti (mark che segue il tracked, aura su Dorumon mentre `predator_active=true`). Il mark migra con `tracked_target` — l'observer detecta `Changed<DorumonBlueprint>` e diffa want vs have.

### Mapping (per FX)

| Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|
| **Mark armed / migrate** | 2 | `Changed<DorumonBlueprint>` con `predator_loop.tracked_target` diverso da quello tracciato dall'observer | `predator_mark_loop` | `EntityCenter(FromBlueprintState("predator_loop.tracked_target"))` (re-resolved each tick) | `Static` |
| **Mark cleared** | 2 | `tracked_target == None` (sia per death, sia per recompute → no eligible enemy) | (manager `VfxEmitter` despawned; preset tail-out plays out) | — | — |
| **Mark fade flash** *(optional polish)* | 1 | `on_kernel_event(UnitDied { unit: tracked })` se `unit == tracked_target` | `predator_mark_fade` (puff su death) | `EntityCenter(EventTarget)` | `Static` |
| **Entry flash (active false→true)** | 1 | `on_kernel_event` branch che porta `predator_active=true` (sia threshold hit sia Ult force) → after the state set, `ctx.notify(...)` | `predator_lock` (red eye-flash su Dorumon + slash overlay su tracked) | `SelfCenter` | `Travel { to: EntityCenter(FromBlueprintState("predator_loop.tracked_target")), ease: EaseOut, ms: 150 }` |
| **Active aura** | 2 | `Added<Buff_PredatorActive>` su Dorumon entity *(componente tipato derivato da `predator_active=true`)* | `predator_aura_loop` | `SelfCenter` | `Static` |
| **Chain consume flash** | 1 | `on_kernel_event(UnitDied { unit: tracked })` durante `dash_metal` ChainStrike resolution | (none — coperto da `chain_arc` di `dash_metal` FSM, vedi 02 skill doc) | — | — |
| **Exit fade (active true→false)** | 1 | `on_kernel_event(TurnEnded)` branch che decrementa `expires_in` a 0, oppure `predator_active=false` da consume | `predator_aura_dissipate` | `SelfCenter` | `Static` |
| **Aura despawn** | 2 | `RemovedComponents<Buff_PredatorActive>` su Dorumon | (manager despawned) | — | — |

### Note implementative

- **Component naming (§2.2e §E):** `predator_active=true` non è un `BuffId` stringa (è un campo bool nello `PredatorLoopState`). Per essere osservabile via `Added`/`RemovedComponents`, il listener inserisce/rimuove un **componente marker dedicato** `Buff_PredatorActive` (tag-style, vuoto) sull'entity Dorumon quando `predator_active` flippa. Listener gameplay continua a leggere `self.state.predator_active` per le decisioni; il componente è solo presentation hook.
- **Mark migrating osservatore (§2.2e §D):** lo sketch in §2.2e §D `observe_predator_mark` è il template. La `Local<HashMap<Entity, Entity>>` mappa Dorumon → mark manager. Detecta diff tra `want` (current `tracked_target`) e `have` (last spawned), despawn old + spawn new. **Non** un emitter singolo retargetable: il preset `predator_mark_loop` ha loop state che deve resettare quando cambia tracked (pop animation va replayed). Despawn-respawn è la scelta corretta qui.
- **Force-entry via Ult (identity §F5 / `metal_cannon` 03):** quando `metal_cannon` Spit emette `SetBlueprintState("predator_loop.predator_active", true)`, il listener osserva quel cambio via `on_kernel_event` (o `Changed<DorumonBlueprint>` se l'engine emette change detection sul write), aggiunge `Buff_PredatorActive`, e contestualmente emette `predator_lock` notify. **Stesso preset** dell'entry da threshold — il player non distingue (e non deve). Coerente con G4 max(N,M) duration: se il buff è già attivo, ri-applicarlo refresh `expires_in` al massimo dei due.
- **`EntityCenter` failure modes (§2.2d §B):**
  - Mark observer trova `tracked_target = None` → no spawn (legal, mark just isn't present).
  - Entry flash `Travel.to` con tracked che muore tra `on_enter` e arrivo proiettile: §2.2d §H.4 snapshot-once policy — il particle atterra sul `Vec2` snapshottato, non sull'entity. ✅
- **Headless:** §2.2e §G. Listener gameplay (state machine, threshold gating, chain consume) gira identico in headless; test integration in `tests/` osservano `PredatorLoopResolved` event payload e `predator_active` state diff, non i VFX.

---

## §6 — Verdetto

`predator_loop` è il **template "blueprint state machine listener" + Full FSM passive** del roster:
- Listener mantiene state interno (`PredatorLoopState`) e pusha signal nell'FSM passivo.
- FSM (`02-02e §A.0`) ha 3 nodi (IdleScan/Armed/ChainPrimed) + 5 edge ispezionabili editor-side.
- State è interrogabile via predicate (`BlueprintState`, `HpPctBelow` con `FromBlueprintState` target_ref).
- State è mutabile via Command (`SetBlueprintState`, F5 Ult force-entry).
- Eventi del kernel (`DamageDealt`, `UnitDied`, `TurnEnded`) trasportano segnali edge-side.

Pattern **generalizzabile** ad altri Digimon che richiedono state machine interna (es. futuri form-change Renamon/Kyubimon, evoluzioni). Vocabolario `BlueprintState` + `SetBlueprintState` + `FromBlueprintState(...)` formalizza il contratto. **Full FSM mandate fit**: la complessità native di predator_loop (tracking + threshold + chain consume + ult force) si esprime in 3 nodi distinti — niente edge esplosi, niente special-casing.

**Allineamento doc-codice è action item pratico**, non gap architetturale.
