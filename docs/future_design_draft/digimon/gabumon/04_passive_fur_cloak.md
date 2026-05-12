# Gabumon — Passive: `fur_cloak` + `twin_core_ice` (Full FSM + listener, dual-path)

> **Goal**: passive **self-targeting** triggered on outgoing `StatusApplied`. Mirror funzionale ma asse opposto al `twin_core_fire` di Agumon (che è **outgoing damage scaling** vs `fur_cloak` = **incoming damage mitigation**). Insieme al Twin Core ice-side definisce il dual-path Gabumon.
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Due `PassiveFsm` paralleli nel blueprint Gabumon, listener entry-point comune:
> - **Path A — `fur_cloak`**: sub-variant **B — Reactive-proc** (block-react su outgoing Chilled, transient hold).
> - **Path B — `twin_core_ice`**: sub-variant **C — State-watch** (specchio Agumon, partner Heated predicate).
>
> **Gap §2.2b condivisi:** dual-role (vedi agumon/04 §6), pre-damage vs post-event (G9), RoundId (G10). Qui solo nuovi.

## §1 — Intent

- **Direzione:** self-mitigation reattiva. Quando Gabumon applica Chilled (qualsiasi target), arma DR self.
- **Effect:** `ApplyBuff { id:"fur_cloak_dr", target:Self_, mul_param:Some(Snapshot("dr_value")), dur:Turns(1) }` — −20% damage taken, dura 1 turno (fino al prossimo damage incoming risolto, o tier end). Alias `ApplySelfBuff` chiuso in `02-02b §C2`.
- **Scope:** **Full FSM dual-path** (vedi §1.5). Anim layer rimane `idle` — i frame range per nodo sono partizioni dello stesso loop idle per editor-inspectability.

## §1.5 — FSM topology (Full FSM mandate, dual-path)

Due FSM indipendenti nel blueprint Gabumon (`02-02e §A.1` boundary note: una passive può ospitare FSM multipli quando i path sono semanticamente disgiunti).

### Path A — `fur_cloak_fsm` (Reactive-proc)

```ron
// Pseudocode FSM (target: src/combat/blueprints/gabumon.rs::fur_cloak_fsm)
PassiveFsm {
    initial: Dormant,
    nodes: [
        Node {
            id: Dormant,
            clip: ("idle", 0..3),
            on_enter: [],
        },
        Node {
            id: Armed,
            clip: ("idle", 4..7),
            on_enter: [
                ApplyBuff { id:"fur_cloak_dr", target_ref: Self_,
                            mul_param: Some(Snapshot("dr_value")), dur: Turns(1) },
                SpawnParticle { preset:"fur_cloak_arm", origin: SelfCenter, motion: Static },
            ],
        },
        Node {
            id: Absorbed,
            clip: ("idle", 8..11),
            on_enter: [
                SpawnParticle { preset:"fur_cloak_absorb", origin: SelfCenter, motion: Static },
            ],
        },
    ],
    edges: [
        // Dormant → Armed: Gabumon applica Chilled (qualsiasi target, basic/skill/ult)
        Edge { from: Dormant, to: Armed,
               on: KernelEvent(StatusApplied { caster_is_self: true, status: Chilled }) },
        // Armed → Absorbed: incoming damage assorbito (consume DR)
        Edge { from: Armed, to: Absorbed,
               on: KernelEvent(DamageDealt { target_is_self: true }) },
        // Absorbed → Dormant: transient consume, torna a rest
        Edge { from: Absorbed, to: Dormant, on: TimeInNode(1) },
        // Armed → Dormant: nessun damage nel turno (buff dur:1 scade su TurnEnded)
        Edge { from: Armed, to: Dormant,
               on: KernelEvent(TurnEnded),
               predicate: BlueprintState { state_key:"fur_cloak_dr.expires_in",
                                           expected: Int(0) } },
    ],
}
```

### Path B — `twin_core_ice_fsm` (State-watch — specchio Agumon)

```ron
// Pseudocode FSM (target: src/combat/blueprints/gabumon.rs::twin_core_ice_fsm)
PassiveFsm {
    initial: Dormant,
    nodes: [
        Node {
            id: Dormant,
            clip: ("idle", 0..3),
            on_enter: [],
        },
        Node {
            id: Armed,
            clip: ("idle", 4..7),
            on_enter: [
                ApplyBuff { id:"twin_core_ice_active", target_ref: Self_,
                            mul_param: Some(Snapshot("ice_boost_mul")), dur: UntilRoundEnd },
                SpawnParticle { preset:"twin_core_ice_ignite", origin: SelfCenter, motion: Static },
                SpawnParticle { preset:"twin_core_ice_link",
                                origin: SelfCenter,
                                motion: Travel { to: EntityCenter(Caster), ease: EaseOut, ms: 200 } },
            ],
        },
        Node {
            id: Boosted,
            clip: ("idle", 8..11),
            on_enter: [
                SpawnParticle { preset:"twin_core_ice_amplify",
                                origin: EntityCenter(EventTarget), motion: Static },
            ],
        },
    ],
    edges: [
        // Dormant → Armed: Agumon applica Heated
        Edge { from: Dormant, to: Armed,
               on: KernelEvent(StatusApplied { caster_is: "agumon", status: Heated }) },
        // Armed → Boosted: Gabumon emette Ice damage (overlay transient)
        Edge { from: Armed, to: Boosted,
               on: KernelEvent(DamageDealt { caster_is_self: true, tag: Ice }) },
        Edge { from: Boosted, to: Armed, on: TimeInNode(1) },
        // Armed → Dormant: fine round
        Edge { from: Armed, to: Dormant,
               on: KernelEvent(RoundEnded),
               on_exit: [SpawnParticle { preset:"twin_core_ice_dissipate",
                                         origin: SelfCenter, motion: Static }] },
    ],
}
```

**Channel mapping (`02-02e §A.1`):**
- **Path A** — Ch1 mandatory (`fur_cloak_arm` + `fur_cloak_absorb` su `on_enter`). Ch2 optional ma presente: `Added/Removed<Buff_FurCloakDR>` → `fur_cloak_loop` aura (armed state hold). Sub-variant B con Ch2 opzionale codificato in `02-02e §E.1`.
- **Path B** — Ch1 + Ch2 entrambi mandatory (sub-variant C standard). Ch1: ignite/link/amplify/dissipate. Ch2: `Added/Removed<Buff_TwinCoreIceActive>` → `twin_core_ice_loop` aura.

**Edge predicate semantica:**
- `caster_is_self:true && status:Chilled` su Path A: filter outgoing status, strict. Basic/skill/ult tutti triggherano (semplicità >> game-feel optimization, decisione operativa preservata).
- `caster_is:"agumon" && status:Heated` su Path B: specchio Agumon. `identity_id` filter.
- `Turns(1)` per `fur_cloak_dr` vs `UntilRoundEnd` per `twin_core_ice_active`: durate intenzionalmente distinte. Path A è block-react (1-turn window), Path B è round-scope synergy.

**Headless determinism:** entrambi gli FSM tickabili headless. `ApplyBuff` gameplay-side identico (`Buff_FurCloakDR` / `Buff_TwinCoreIceActive` tag-pure marker, valori nel `Buffs` stringy map letti dalla damage pipeline). `SpawnParticle` no-op.

## §2 — Blueprint contract

```rust
impl BlueprintListener for GabumonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            CombatEvent::StatusApplied { caster, status: Chilled, .. }
                if ctx.is_self(caster) && self.fur_cloak_arming_allowed() => {
                ctx.add_self_buff(BuffId("fur_cloak_dr"), value: 0.20, dur: Turns(1));
            }
            _ => {}
        }
    }
}

// Twin Core ice-side (file separato logico, stesso blueprint)
impl BlueprintListener {
    // on StatusApplied(Heated, caster:Agumon) → arma TwinCoreIceActive su self
    // Vedi agumon/04 specchio
}
```

## §3 — Activation flow

```
turno T: Gabumon casta gabumon_shot → EmitStatus(Chilled, primary)
  └─ CombatEvent::StatusApplied { caster:Gabumon, target:primary, status:Chilled }
     └─ fur_cloak listener cattura → add_self_buff(fur_cloak_dr, dur:1)

turno T+1 (nemico): nemico colpisce Gabumon
  └─ damage pipeline legge has_buff(fur_cloak_dr) → multiplier (1 − 0.20) = 0.80
  └─ DamageDealt(Gabumon, scaled)
  └─ buff decrementa / expira a end_of_owner_next_turn
```

## §4 — Trigger filter precision

- **Caster check:** `caster == self` (Gabumon stesso). Non Tentomon che applica Paralyzed, non altri.
- **Status check:** `status == Chilled`. Strict.
- **Basic vs Skill vs Ult:** basic applica Chilled. Identity §1 propone **"DR-self on apply"** sempre, ma 04 dell'01-basic_claw_attack domanda se gate solo su skill/ult. **Decisione operativa:** listener triggera **sempre** (semplicità >> ottimizzazione game-feel). Bilanciare via durata DR (1 turno = 1 colpo subito mitigato), non via gate trigger.
- **Cap:** single instance attiva, refresh durata.

## §5 — Power tuning placeholder

- **Value:** −20% damage taken (allineato a identity §4).
- **Durata:** 1 turno (= fino al prossimo end-of-own-turn).
- **Interazione con Ult `blue_cyclone` DR 30%:** vedi gap S2 (gabumon/03 §5.3). Replace-max consigliato.

## §5b — Presentation (Ch1 + Ch2, `02-02e §A.1` dual-path B+C)

Due `PassiveFsm` paralleli (§1.5): `fur_cloak_fsm` (Reactive-proc) + `twin_core_ice_fsm` (State-watch). Anim resta su `idle` (FSM nodes partizionano frame range per editor-inspectability, non clip distinte). Presentation split sui due path: FSM `SpawnParticle` Commands su `on_enter` (transition flash) + Ch1 `notify` listener-side per gli accent event-bound + Ch2 component observer (`Added/Removed<Buff_*>`) per le aura persistenti.

### Path A — `fur_cloak` (self DR su outgoing Chilled)

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| A1 | Arm flash | 1 | `ctx.notify` da listener su `StatusApplied{status:Chilled, caster:self}` | `fur_cloak_arm` (azure shimmer pop) | `SelfCenter` | `Static` |
| A2 | Active aura | 2 | `Added<Buff_FurCloakDR>` su Gabumon | `fur_cloak_loop` (subtle azure body shimmer) | `SelfCenter` | `Static` |
| A3 | Damage absorbed flash | 1 | `ctx.notify` da listener su `DamageDealt{target:self}` AND `has_buff("fur_cloak_dr")` | `fur_cloak_absorb` (azure spark burst) | `SelfCenter` | `Static` |
| A4 | Aura despawn | 2 | `RemovedComponents<Buff_FurCloakDR>` | — (emitter manager despawned) | — | — |

### Path B — `twin_core_ice` (cross-buff su `StatusApplied{Heated, caster:Agumon}`)

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| B1 | Arm flash | 1 | `ctx.notify` da listener su `StatusApplied{status:Heated, caster:agumon}` | `twin_core_ice_ignite` (dual frost-pop) | `SelfCenter` | `Static` |
| B2 | Partner link *(opzionale)* | 1 | stesso trigger di B1 | `twin_core_ice_link` (frost arc verso Agumon) | `SelfCenter` | `Travel { to: EntityCenter(Caster), ease: EaseOut, ms: 200 }` |
| B3 | Active aura | 2 | `Added<Buff_TwinCoreIceActive>` su Gabumon | `twin_core_ice_loop` (subtle frost halo) | `SelfCenter` | `Static` |
| B4 | Boosted hit overlay | 1 | `ctx.notify` da listener su `DamageDealt{caster:self, tag:Ice}` AND buff present | `twin_core_ice_amplify` (frost burst overlay) | `EntityCenter(EventTarget)` | `Static` |
| B5 | Dissipate flash | 1 | `ctx.notify` su `RoundEnded` (buff drop) | `twin_core_ice_dissipate` | `SelfCenter` | `Static` |
| B6 | Aura despawn | 2 | `RemovedComponents<Buff_TwinCoreIceActive>` | — | — | — |

### Note implementative

- **Tag components convenzionali (§2.2e §E):**
  - `Buff_FurCloakDR` — **tag-pure** (presence/absence). Il valore numerico (mult 0.20) resta nella `Buffs` stringy map; l'observer Channel 2 legge solo presence. Vedi gap N5.
  - `Buff_TwinCoreIceActive` — tag-pure (boost ×1.15 gestito via lookup buff stringy nella damage pipeline; allineato a `00_identity.md §6 D3`, simmetrico con Agumon fire-side).
- **`EntityRef::Caster` / `EntityRef::EventTarget`** sono ammessi solo in Channel 1 (listener context, §2.2e §C). Non possono essere usati da observer Channel 2 — che non ha event scope.
- **A3 (`fur_cloak_absorb`)** è guard'd dal listener su `has_buff("fur_cloak_dr")` — il preset si attiva solo quando il DR effettivamente assorbe.
- **B5 dissipate flash** vive su Channel 1 e va emesso **prima** del Removed event che chiude B6 (ordering interno listener vs ECS observer su RoundEnded). Vedi gap N5.5.
- **Headless gating:** entrambi i path sono presentation-only. In `headless` mode (`#[cfg(not(feature = "windowed"))]`) i `ctx.notify` sono no-op e gli observer Channel 2 non sono registrati; il gameplay (buff applicato, multiplier letto in damage pipeline) resta intatto.

## §6 — Stress test findings (nuovi)

### ✅ Cosa funziona

- Stesso pattern dual-role di agumon/04: listener vive in Rust, niente RON. **Riusabile** come template per passive dual-path FSM (sub-variant B + C) del roster.

### ⚠️ Gap nuovi

1. **N1 — Buff "self-targeting".** ✅ **Chiuso (round-3, 2026-05-12):** alias di `ApplyBuff { target: Self_, kind: DR }` (vedi `02-02b §C2` + `02-08 §H`). Il listener path lo invoca direttamente via `ctx.add_self_buff(...)` (non via FSM Commands), ma il kernel effect è uniforme: stesso `kind`, stesso stacking (intra-unit replace-max). Nessun verbo separato.
2. **N2 — Buff timing race con FSM owner.** Il listener applica buff **mentre Gabumon è ancora in `Burst` FSM**. Il buff è "attivo" già durante il resto della FSM corrente? **Decisione:** sì (state read live), ma il primo damage incoming è del nemico al turno successivo → effetto pratico = mitigation nel turno T+1.
3. **N3 — Twin Core ice-side (specchio Agumon).** Stesso blueprint contiene 2 listener path: `fur_cloak` (self) + `twin_core_ice` (cross). **Action item:** decidere se file separato `04b_passive_twin_core_ice.md` o tutto qui. **Decisione operativa:** tutto qui. Il blueprint Gabumon ha 2 reactive arm:
   - `fur_cloak`: on outgoing Chilled → self DR
   - `twin_core_ice`: on `StatusApplied(Heated, caster:Agumon)` → BuffSelf("twin_core_ice_active", round-scoped) che boosta Ice damage `×1.15` (+15%) next outgoing (canon `00_identity.md §6 D3`, simmetrico con Agumon Twin Core).
4. **N4 — `value_param` schema buff.** Vedi gap S2 (gabumon/03). Buff `value` (mult) vs status `stacks` (count) sono entità distinte nel vocabolario. Cleanse Patamon non li tocca (è solo debuff filter).
5. **N5 — Value-carrying typed-buff component.** ✅ **Chiuso (round-3, 2026-05-12):** decisione A (tag-pure marker) **formalizzata** in `02-02e §E.1` (channel layout) come convenzione canon del roster. `Buff_FurCloakDR` è zero-sized tag (presence-only); il valore numerico vive nella `Buffs` stringy map letta dalla damage pipeline. Observer Channel 2 osserva solo `Added/Removed` per gestire emitter lifecycle. Numeric binding ("VFX intensity proporzionale al mult") resta deferred a `§2.2e §I`.
6. **N5.5 — Ordering Channel 1 vs Channel 2 su buff drop.** ✅ **Chiuso (round-3, 2026-05-12):** ordering **non-normativo**, overlap di ≤1 frame tollerato — codificato in `02-02e §F` (Channel arbitration rules). Sia "flash prima del Removed" sia "flash dopo il Removed" sono accettabili: i due path vivono su channel disgiunti e il player non distingue ≤1f di drift.
7. **N6 — Observer multi-effect (anim + particle + audio).** *Deferred*: codice corrente (idle puro, niente clip dedicata su Added) non richiede observer Channel 2 di emettere `AnimClipPlay`. Gap resta aperto in `02-02e §I` per il primo caso reale (non Gabumon).

## §7 — Verdetto

`fur_cloak` + `twin_core_ice` definiscono il **template buff-applier dual-path Full FSM** del roster:
- Due FSM paralleli nello stesso blueprint: Reactive-proc (block-react) + State-watch (partner synergy). `02-02e §A.1` boundary note ("una passive può mixare sub-variant") applica intra-blueprint.
- Self-buff (fur_cloak) e cross-buff (twin_core) emettono Commands FSM-side via `ApplyBuff` unificato.
- Cleanse-immune by design (buff, non debuff).
- Edge predicate filtrano caster/status precisamente (`caster_is_self`, `caster_is:"agumon"`).
- **Presentation completa:** Ch1 trigger-proc su FSM `on_enter`, Ch2 persistent-presence via observer su `Buff_*` typed-component. Anim layer idle puro — frame range per nodo sono partizioni dello stesso loop per editor-inspectability.

**Gap nuovi esposti (status post round-3):** N1 ✅ chiuso (alias `ApplyBuff { target: Self_, kind: DR }`, `02-02b §C2`). N5 ✅ chiuso (tag-pure marker formalizzato, `02-02e §E.1`). N5.5 ✅ chiuso (ordering non-normativo, `02-02e §F`). N6 deferred a `02-02e §I` (primo caso reale). S2 ✅ chiuso (alias `ApplyBuff`, vedi gabumon/03 §5). **Full FSM mandate (`02-02e §A.0`)** ✅ chiuso: 3+3 nodi totali across Path A/B + 7 edge complessivi + clip frame range definiti + VFX su entrambi i canali.
