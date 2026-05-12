# Patamon — Passive: `holy_aegis` (Full FSM + listener, sub-variant A: Aura-loop)

> **Goal**: passive **non-reattiva**, sempre-attiva finché Patamon vive. Aura DR team-wide, applicata/rimossa ai death/spawn events del team.
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Sub-variant **A — Aura-loop** standard (`Inactive ↔ Aura ↔ Cleared`). Ch2 mandatory (aura state-bound), Ch1 optional (start/stop flash). `02-02e §E.1` codifica.
>
> **Gap §2.2b condivisi:** dual-role pattern (agumon/04). Qui nuovi.

## §1 — Intent

- **Effect:** −10% damage taken per **tutti gli alleati vivi** finché Patamon è vivo.
- **Trigger:** aura state-bound — FSM `Inactive ↔ Aura ↔ Cleared` (vedi §1.5). Edge solo `KernelEvent` su lifecycle (CombatStarted/UnitDied), niente edge gating su damage events.
- **Self-included:** sì (vedi identity §7).
- **Atlas:** anim layer `idle` durante tutto (no clip dedicata); frame range per nodo sono partizioni del loop idle per editor-inspectability.

## §1.5 — FSM topology (Full FSM mandate)

Sub-variant **A — Aura-loop** standard. Listener osserva `CombatStarted` (o workaround `TurnAdvanced { turn: 0 }` finché A4 deferred — vedi §6 A4) e `UnitDied { unit: self }`. Pusha signal nell'FSM per lifecycle transitions.

```ron
// Pseudocode FSM (target: src/combat/blueprints/patamon.rs::holy_aegis_fsm)
PassiveFsm {
    initial: Inactive,
    nodes: [
        Node {
            id: Inactive,
            clip: ("idle", 0..3),
            on_enter: [],                                 // pre-combat or Patamon dead
        },
        Node {
            id: Aura,
            clip: ("idle", 4..11),                        // long hold, character at rest with aura active
            on_enter: [
                // Applica aura buff a tutti gli alleati vivi (incluso self)
                ApplyBuff { id:"holy_aegis", target_ref: AllAlliesAlive,
                            mul_param: Some(Snapshot("aura_dr_value")), dur: Permanent },
                SpawnParticle { preset:"holy_aegis_dawn",
                                origin: SelfCenter, motion: Static },        // Ch1 optional start flash
            ],
        },
        Node {
            id: Cleared,
            clip: ("idle", 0..3),
            on_enter: [
                // Rimuovi aura buff da tutti gli alleati
                EmitCleanse { target_ref: AllAlliesAlive,
                              tag_filter: ById("holy_aegis") },
                SpawnParticle { preset:"holy_aegis_dusk",
                                origin: SelfCenter, motion: Static },        // Ch1 optional stop flash
            ],
        },
    ],
    edges: [
        // Inactive → Aura: combat start (Patamon vivo)
        Edge { from: Inactive, to: Aura,
               on: KernelEvent(CombatStarted | TurnAdvanced),
               predicate: And(UnitAlive(Self_),
                              BlueprintState { state_key:"holy_aegis.initialized",
                                               expected: Bool(false) }) },
        // Aura → Cleared: Patamon muore → aura collassa
        Edge { from: Aura, to: Cleared,
               on: KernelEvent(UnitDied),
               predicate: IsSelf(EventUnit) },
        // Cleared è terminale (no edge back). Patamon morto = aura assente per resto combat.
        // Revive deferred (vedi §6 A2): se mai aggiunto, edge Cleared → Aura su UnitRevived.
    ],
}
```

**Workaround A4 (`CombatStarted` deferred):** l'edge `Inactive → Aura` accetta **OR** tra `CombatStarted` e `TurnAdvanced` con guard `BlueprintState("holy_aegis.initialized") == false` per idempotency. Il listener su `TurnAdvanced { turn: 0 }` (primo turn) pusha signal sintetico nel pending_signals queue; FSM consuma + sets `initialized=true` come parte di `on_enter(Aura)` (via `SetBlueprintState` Command implicito o flag interno). Quando `CombatStarted` viene formalizzato canon-side (A4 closure), il guard `initialized` resta come safety net contro double-init.

**Edge predicate semantica:**
- `AllAlliesAlive`: nuovo target_ref per `ApplyBuff`/`EmitCleanse` (estensione `TargetShape::AoE { side: Ally }`). Iterazione team-wide live al `on_enter(Aura)`. Buff con `dur: Permanent` (`02-08 §H.2` Aura-only) sopravvive turni successivi.
- `UnitAlive(Self_)` + `initialized==false`: idempotency guard, evita re-spawn della aura su event ridondanti.
- `IsSelf(EventUnit)` su `UnitDied`: strict caster check, niente trigger su altri allied deaths.

**Stack additivo cross-unit:** `holy_aegis` (10%) + `fur_cloak` (20% Gabumon) = 30% effettivo, sotto cap 50% (`02-08 §H.3` cross-unit additive clamp 0.5). Damage pipeline somma tutti i buff `kind:DR` attivi su entity.

**Channel mapping (`02-02e §A.1` Aura-loop):**
- **Ch1 optional:** `holy_aegis_dawn` su `on_enter(Aura)` (start flash), `holy_aegis_dusk` su `on_enter(Cleared)` (stop flash). Non mandatory per sub-variant A, ma presenti per legibility transition.
- **Ch2 mandatory:** presentation observer su `Added<Buff_HolyAegisActive>` / `RemovedComponents<...>` (per ogni alleato) → `holy_aegis_halo_loop` aura emitter parented all'ally entity. Buff component tipato `Buff_HolyAegisActive` inserito dal kernel al `ApplyBuff` resolution (`02-02e §E`).

**Headless determinism:** FSM tickabile headless. `ApplyBuff(target_ref: AllAlliesAlive, dur: Permanent)` itera team alive e inserisce `Buff_HolyAegisActive` su ogni entity; damage pipeline legge il buff per il DR mult. `EmitCleanse` su `Cleared` rimuove. Test integration esercitano: spawn Patamon + 2 alleati → `tick_passive_fsm` → osservano `Buff_HolyAegisActive` presenza su tutti tre → injecta `UnitDied{unit:patamon}` → osservano cleanse.

## §2 — Blueprint contract

```rust
impl BlueprintListener for PatamonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            // Patamon morto → rimuove l'aura da tutti gli alleati
            CombatEvent::UnitDied { unit } if ctx.is_self(unit) => {
                for ally in ctx.team_alive() {
                    ctx.remove_buff(ally, "holy_aegis");
                }
            }
            // Alleato spawnato/revive (futuro) o combat start → applica aura se Patamon vivo
            CombatEvent::CombatStarted | CombatEvent::UnitSpawned { .. }
                if ctx.is_self_alive() => {
                for ally in ctx.team_alive() {
                    ctx.add_buff(ally, "holy_aegis", value: 0.10, dur: Permanent);
                }
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
combat start
  └─ CombatStarted event
     └─ Patamon listener applica holy_aegis (Permanent) a tutti gli alleati vivi (incluso self)

durante combat
  └─ tutti i damage incoming agli alleati: kernel legge buff "holy_aegis" → × 0.90

Patamon muore
  └─ UnitDied(self)
     └─ rimuove holy_aegis da tutti gli alleati
     └─ damage incoming torna 100%
```

## §4 — Trigger filter precision

- **Apply trigger:** `CombatStarted`, `UnitSpawned`, futuro `UnitRevived`.
- **Remove trigger:** `UnitDied { unit: Patamon }`.
- **Filter:** strict `caster == self`. Niente "altri Patamon" (canon: solo 1 per team).

## §5 — Power tuning

- **Value:** 10% DR (allineato a identity §4).
- **Stack rules:** additivo con `fur_cloak` (Gabumon DR 20%) → −30% se entrambe attive, **non moltiplicativo** (identity §4 lo specifica esplicitamente).
- **Cap totale DR:** 50% suggerito (evita "muri").

## §6 — Open questions

1. **A1 — Buff "Permanent" semantica.** ✅ **Chiuso (round-3, 2026-05-12):** `BuffDur::Permanent` ammesso come variante **solo per `kind: Aura`** (buff team-wide state-bound, no tick-down) — formalizzato in `02-08 §H.2` (BuffDur taxonomy). Cleanup esplicito via `remove_buff` su death event. Cleanse Patamon (`patapata_hover`) **non rimuove** holy_aegis (è buff alleato `kind:Aura`, cleanse filtra `kind:Debuff`).
2. **A2 — Patamon revive (futuro fuori scope).** Se in futuro arriva revive, listener deve riapplicare aura via hook `UnitRevived`. **Deferred M017+.**
3. **A3 — Stack additivo con `fur_cloak`.** ✅ **Chiuso (round-3, 2026-05-12):** formalizzato in `02-08 §H.3` (stacking rules): **cross-unit additivo con clamp 0.5** (es. `holy_aegis` 10% + `fur_cloak` 20% = 30% effettivo, sotto cap). Intra-unit invece replace-max (vedi gabumon/03 §5.3 ult `dr_self` vs `fur_cloak`). La damage pipeline somma valori DR di buff `kind:DR` attivi su una entity (cross-unit additive), clamp finale 0.5.
4. **A4 — `CombatStarted` kernel event.** **Open (deferred):** event bus formalization non ancora chiusa in `events.rs`. Workaround corrente: listener inizializza aura su prima `TurnAdvanced` (turn 0) check, idempotent. Action item permanente: aggiungere `CombatStarted` come kernel event utile a ogni init-time listener (anche revive futuro). **Non blocca M017** — workaround `TurnAdvanced` regge.

## §7 — Verdetto

`holy_aegis` è il **template Aura-loop pure** del roster (`02-02e §A.1` sub-variant A):
- FSM 3 nodi (Inactive/Aura/Cleared) + 2 edge lifecycle-driven. Ch2 mandatory (aura state-bound), Ch1 optional flash (dawn/dusk transition).
- Workaround A4 (`CombatStarted` deferred): edge OR `CombatStarted | TurnAdvanced` + `initialized` guard idempotent. Quando A4 chiude doc-side, l'edge guard resta come safety net.
- `ApplyBuff(target_ref: AllAlliesAlive, dur: Permanent)` su `on_enter(Aura)` — pattern team-wide aura applicabile a future passive (es. team-wide buff condizionale).

Status post round-3:
- A1 ✅ chiuso (`BuffDur::Permanent` per `kind:Aura`, `02-08 §H.2`).
- A3 ✅ chiuso (DR stacking cross-unit additivo clamp 0.5, `02-08 §H.3`).
- A4 deferred (event bus formalization) — workaround edge OR regge.
- A2 deferred M017+ (revive out-of-scope) — se aggiunto, edge `Cleared → Aura` on `UnitRevived`.
- **Full FSM mandate** ✅ chiuso (`02-02e §A.0`).

**Nessun gap architetturale duro** rimasto — A4 è estensione minor del kernel event set, workaround regge fino a formalizzazione.
