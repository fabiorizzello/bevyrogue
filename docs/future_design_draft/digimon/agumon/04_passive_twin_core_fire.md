# Agumon — Passive: `twin_core_fire` (Full FSM + listener, sub-variant C: State-watch)

> **Goal**: validare il **dual-role blueprint** §2.2b §B. Il blueprint Agumon è **executor** durante una sua skill (file 01–03) e **listener+FSM** sempre, ascoltando `CombatEvent` per pilotare l'FSM passivo e applicare effetti. I due ruoli **non si parlano internamente**.
>
> **Full FSM mandate (`02-02e §A.0`):** la passive ha **FSM 3+ nodi + edge + clip frame range + VFX su almeno un canale**, tickabile headless. Sub-variant **C — State-watch**: l'edge gating legge predicate sul partner status (Chilled by Gabumon) per transizionare `Dormant → Armed`. §A.1 trigger semantics.

## §1 — Intent

Twin Core fire-side (canon §8): **+damage condizionale** se Gabumon (Twin Core partner) è in team e ha applicato `Chilled` su un nemico nello stesso round.

- **Direzione:** fire-side aumenta quando l'ice-side ha "armato" il bersaglio.
- **Reciprocità:** Gabumon ha specularmente `twin_core_ice` che ascolta `StatusApplied(Heated)` da Agumon (vedi `gabumon/04_passive_fur_cloak.md` Path B).
- **Scope:** **Full FSM listener-driven** (vedi §1.5). Anim layer rimane `idle` (no clip dedicata visibilmente diversa) — i frame range per nodo sono partizioni dello stesso loop idle per editor-inspectability.

> **⚠️ Team-conditional (esplicito).** La passive è **completamente inerte** se Gabumon non è in party. Nessun trigger possibile (nessuno emette `StatusApplied(Chilled, caster=gabumon)`), l'FSM resta in `Dormant` per tutto il combat, nessun buff applicato, nessun VFX. È accettato come HSR-style team synergy (cfr. eidolons/team-comp bonus): pianifica party comp di conseguenza. Niente fallback "vale comunque +X% Fire", per non rompere la specificità del nome ("Twin Core" implica due core, due unità).

## §1.5 — FSM topology (Full FSM mandate)

Sub-variant **C — State-watch** (`02-02e §A.1`). Listener osserva `CombatEvent::StatusApplied`, valuta predicate (caster Gabumon + status Chilled) e pusha signal in `pending_signals`; FSM consuma signal su `KernelEvent(...)` edge.

```ron
// Pseudocode FSM (target: src/combat/blueprints/agumon.rs::twin_core_fire_fsm)
PassiveFsm {
    initial: Dormant,
    nodes: [
        Node {
            id: Dormant,
            clip: ("idle", 0..3),
            on_enter: [],                                 // pure rest state
        },
        Node {
            id: Armed,
            clip: ("idle", 4..7),
            on_enter: [
                ApplyBuff { id:"twin_core_fire_active", target_ref: Self_,
                            mul_param: Some(Snapshot("fire_boost_mul")), dur: UntilRoundEnd },
                SpawnParticle { preset:"twin_core_ignite", origin: SelfCenter, motion: Static },
                SpawnParticle { preset:"twin_core_link_pulse",
                                origin: EntityCenter(Caster),
                                motion: Travel { to: SelfCenter, ease: EaseOut, ms: 250 } },
            ],
        },
        Node {
            id: Boosted,
            clip: ("idle", 8..11),
            on_enter: [
                SpawnParticle { preset:"twin_core_amplify",
                                origin: EntityCenter(EventTarget), motion: Static },
            ],
        },
    ],
    edges: [
        // Dormant → Armed: Gabumon applica Chilled
        Edge { from: Dormant, to: Armed,
               on: KernelEvent(StatusApplied { caster_is: "gabumon", status: Chilled }) },
        // Armed → Boosted: Agumon emette Fire damage (un frame transient per VFX overlay)
        Edge { from: Armed,    to: Boosted,
               on: KernelEvent(DamageDealt { caster_is_self: true, tag: Fire }) },
        Edge { from: Boosted,  to: Armed,    on: TimeInNode(1) },
        // Armed → Dormant: fine round (buff drop via RoundEnd cleanup)
        Edge { from: Armed,    to: Dormant,
               on: KernelEvent(RoundEnded),
               on_exit: [SpawnParticle { preset:"twin_core_dissipate",
                                         origin: SelfCenter, motion: Static }] },
    ],
}
```

**Edge predicate semantica:**
- `caster_is:"gabumon"` su Dormant→Armed filtra il `caster.identity_id` del `StatusApplied`. Niente trigger su Tentomon Paralyzed o nemici con status fittizio.
- `caster_is_self:true && tag:Fire` su Armed→Boosted filtra outgoing damage solo per Fire emesso da Agumon stesso.
- Round-scope buff = `dur: UntilRoundEnd` su `ApplyBuff` (§9 G-Buff); `RoundEnded` edge fa cleanup.

**Channel mapping (`02-02e §A.1`):**
- **Ch1 (trigger-proc):** tutti gli `SpawnParticle` `on_enter` di Armed/Boosted + `on_exit` di Armed sono Ch1 visual (one-shot pop, link beam, amplify overlay, dissipate).
- **Ch2 (persistent-presence):** presentation observer su `Added<Buff_TwinCoreFireActive>` / `RemovedComponents<...>` (template `02-02e §D`) gestisce `twin_core_fire_loop` aura per la durata di `Armed`. State-watch sub-variant standard (entrambi i canali mandatory per A.1).

**Headless determinism:** FSM tickabile via `tick_passive_fsm(ctx)` (`02-02e §A.1`) — `ApplyBuff` gameplay-side gira identico, `SpawnParticle` no-op. Test integration esercitano edge Dormant↔Armed e leggono `Buff_TwinCoreFireActive` presence + `DamageDealt` multiplier downstream.

## §2 — Blueprint contract

```rust
// Pseudocode (non binding finché M017 non definisce trait)
impl BlueprintListener for AgumonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            CombatEvent::StatusApplied { target, status: Chilled, caster, .. } => {
                if ctx.is_partner(caster, "gabumon") && ctx.same_round() {
                    // Mark TwinCoreActive { until: end_of_round } su self
                    ctx.add_self_buff(BuffId("twin_core_fire_active"), 1);
                }
            }
            CombatEvent::DamageDealt { caster: Agumon, target, tag: Fire, .. }
                if ctx.has_self_buff("twin_core_fire_active") => {
                // Boost +X% sul damage emesso (riapplica modifier)
                // OPPURE: il boost è già pre-applicato dal pre-damage hook
                // Vedi §6.2 per il design exact
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
turno T (Gabumon attiva)
  Gabumon FSM (gabumon_shot) → EmitStatus(Chilled, target=enemy1)
    └─ CombatEvent::StatusApplied { caster: gabumon, target: enemy1, status: Chilled }
       └─ Agumon listener cattura
          └─ aggiunge self-buff "twin_core_fire_active" (round-scoped)

turno T+1 (Agumon attiva)
  Agumon FSM (baby_flame/baby_burner/sharp_claws) → EmitDamage(...)
    └─ pre-damage hook letto: has_self_buff("twin_core_fire_active")? sì
       └─ damage scaled × twin_core_multiplier (es. ×1.15)
       └─ CombatEvent::DamageDealt(boosted amount)

fine round → buff scaduto
```

## §4 — Trigger filter precision

Listener match deve evitare false positive:

- **Caster check:** `caster == Gabumon` (alleato in team). Non Tentomon che applica Paralyzed; non un nemico con status fittizio. Filter su `caster.identity_id == "gabumon"`.
- **Status check:** `status == Chilled`. Filter strict.
- **Round scope:** `ctx.same_round()` = il buff dura fino a end_of_round, non N turni. **Implementazione:** buff `expires_on: RoundEnd`.
- **Cap:** una sola attivazione per round o accumulabile? **Proposta:** binary flag (active / not active), no stack. Più semplice, evita spirali abuse.

## §5 — Power tuning placeholder

- **Multiplier:** `×1.15` (+15% damage Fire emesso da Agumon, fino a fine round, dopo che Gabumon ha applicato Chilled).
- **Alternatve:** flat +damage, crit chance bonus, ToughnessHit bonus. **Decisione canon §8:** "+damage" → multiplier su Fire damage. Confermato.
- **Numero finale è game-design**, non FSM-stress.

## §6 — Stress test findings

### ✅ Cosa funziona

- Dual-role pattern (listener vs executor) è **chiaramente separato**: il listener osserva `CombatEvent` e pusha signal nell'FSM passivo; la skill-FSM (in esecuzione su 01/02/03) emette Commands → KernelEffect → `CombatEvent`. Il bus è il mediatore. **Pulito.**
- Round-scope buff è facilmente esprimibile come `Buff { expires_on: RoundEnd }`. Sistema esistente nel kernel (status durations) lo supporta concettualmente.
- **Full FSM mandate fit (`02-02e §A.0`):** State-watch sub-variant si mappa cleanly su 3 nodi (Dormant/Armed/Boosted) + 4 edge. Editor-inspectable parity con skill FSM. Niente special-casing nel validator (`02-02b §L` esteso a passive FSM include il check "almeno un VFX `on_enter` su nodo raggiungibile" — soddisfatto da `twin_core_ignite` su Armed e `twin_core_amplify` su Boosted).

### ⚠️ Contraddizioni / gap

1. **Pre-damage hook vs post-damage event.** Il listener Agumon vuole **modificare** il damage di un proprio `EmitDamage` se ha il buff attivo. Ma `CombatEvent::DamageDealt` è **post-fact**: il damage è già applicato. Per modificare, serve **pre-damage hook** o re-design:
   - **A.** Il buff è letto **dalla resolution kernel** (non dal listener) come "Agumon ha buff TwinCoreActive → applica multiplier al damage in resolution pipeline". Listener applica il **buff component**, kernel lo legge in resolution. ✅ Modello pulito.
   - **B.** Listener emette una `CombatEvent::DamageModifierRequest` prima di `DamageDealt` finale. Bus-mediated, ma introduce nuovo evento.
   - **Decisione consigliata:** A. Listener = buff applier; kernel = buff reader nella damage pipeline (§2.8 cascade).
2. **Listener ordering quando più passive ascoltano lo stesso evento.** Se Gabumon applica Chilled e: (a) Agumon listener arma TwinCore, (b) Kyubimon listener arma form-identity Chilled, (c) Tentomon listener fa qualcos'altro. **Ordine deterministico?** Proposta: ordine di registrazione = ordine di team slot. Deterministic, configurabile. ✓ Allineato a Bevy SystemSet.
3. **Same-round disambiguation.** `ctx.same_round()` richiede che il combat tracker abbia `current_round_id: u32`. C'è già? Verificare in `src/combat/state.rs` se esiste round counter. Se no, **action item:** aggiungere `RoundId` al `CombatState` per supportare round-scoped buff. **Likely esiste**, ma confermare.
4. **Twin Core già implementato (legacy).** I dati in `skills.ron` hanno `custom_signals: [(owner: "agumon", signal: "apply_heated", payload: Amount(amount: 3))]`. Questo è il **vecchio meccanismo** signal-based. La transizione al nuovo modello §2.2b prevede:
   - Listener Agumon ascolta `StatusApplied(Chilled)` direttamente, **niente custom_signals**.
   - I `custom_signals` legacy possono restare come **compat shim** finché tutti i blueprint sono migrati (confermato dal memory note: "keep the legacy Digimon-specific variants as compatibility shims").
5. **Twin Core bidirezionale = passive accoppiata.** Il file 04 Agumon copre fire-side. Quando si farà la cartella Gabumon, il file 04 di Gabumon coprirà ice-side speculare. Per evitare duplicazione/drift: definire **Twin Core come pattern unico** in `02-02b_animation_fsm.md` o in un file shared `digimon/_shared/twin_core.md`? **Decisione consigliata:** shared doc, ogni Digimon ha solo il side specifico. **Differire**: lo decidiamo quando arriviamo a Gabumon.

### 🟡 Aperte (non blocker)

- Listener overhead: 6 blueprint × N eventi per turno = quanti listener match? Trascurabile (vocabolario eventi piccolo, filter rapido).
- Round-end cleanup: serve sistema che droppa buffs `expires_on: RoundEnd`. Probabilmente esistente.

## §6b — VFX (Channel 1 + Channel 2, §2.2e)

> No clipmontage, no `SpawnParticle` Command. Twin Core fire-side is listener-only, so VFX vivono nei due canali §2.2e: **Channel 1** (`ListenerCtx::notify`) per i transition flash, **Channel 2** (presentation observer su `Added<Buff_TwinCoreFireActive>` / `RemovedComponents<...>`) per l'aura persistente. Naming preset porta il flavor (dual-element fire+ice), `VfxLocus` non ha anchor di body part.

### Mapping (per FX)

| Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|
| **Arm flash** | 1 | `on_kernel_event(StatusApplied { caster: gabumon, status: Chilled })` → after `add_self_buff(BuffId("twin_core_fire_active"), UntilRoundEnd)` | `twin_core_ignite` (one-shot dual-flame pop) | `SelfCenter` | `Static` |
| **Partner link** *(optional)* | 1 | stesso trigger sopra, secondo `ctx.notify` | `twin_core_link_pulse` (beam) | `EntityCenter(Caster)` *(Gabumon)* | `Travel { to: SelfCenter, ease: EaseOut, ms: 250 }` |
| **Active aura** | 2 | `Added<Buff_TwinCoreFireActive>` on Agumon entity | `twin_core_fire_loop` | `SelfCenter` (re-resolved each tick) | `Static` |
| **Boosted hit overlay** | 1 | `on_kernel_event(DamageDealt { caster: self, tag: Fire })` + `ctx.has_self_buff("twin_core_fire_active")` | `twin_core_amplify` | `EntityCenter(EventTarget)` | `Static` |
| **Dissipate** | 1 | `on_kernel_event(RoundEnded)` + buff drop branch in cleanup | `twin_core_dissipate` (soft poof) | `SelfCenter` | `Static` |
| **Aura despawn** | 2 | `RemovedComponents<Buff_TwinCoreFireActive>` on Agumon entity | (no preset — manager `VfxEmitter` removed; preset's tail-out frames play out and self-despawn) | — | — |

### Note implementative

- **Buff component naming (§2.2e §E):** `BuffId("twin_core_fire_active")` deve avere componente tipato `Buff_TwinCoreFireActive` per essere osservabile via `Added`/`RemovedComponents`. Registrare in `BuffComponentRegistry`.
- **Boosted hit lettura buff:** la query usa `ctx.has_self_buff("twin_core_fire_active")` (string-based) — quella stringa è autoritativa per gameplay, il componente tipato è solo per la presentation. Niente doppia source-of-truth.
- **Partner link `EntityCenter(Caster)`:** valido **solo** in Channel 1 (§2.2e §C tabella). Il `Caster` è `Gabumon` perché viene dal kernel event `StatusApplied { caster, .. }`. Se Gabumon è morto/fuori party a momento di emissione: spawn dropped silenziosamente (§2.2d §B `EntityRef` failure modes).
- **Boosted hit pinning:** il `twin_core_amplify` è pinnato all'evento `DamageDealt`, non a un nodo FSM. Coerente con §2.2b §M nota finale ("particle reattivo a kernel event in listener attivo del blueprint, non in FSM").
- **Reciprocità Gabumon:** speculare. `BuffId("twin_core_ice_active")` → `Buff_TwinCoreIceActive` → preset `twin_core_ice_loop` (palette blu/cyan). Mapping listener → notify identico sostituendo `Heated` per `Chilled`. Doc Gabumon `04_passive_fur_cloak.md` (o file `04b_passive_twin_core_ice.md` se separato) erediterà questa tabella swap-colorata.

### Headless

Tutto §2.2e §G: notify drop in headless, observer non compila. Gameplay (buff applier + multiplier cascade §G9) gira identico. Test integration in `tests/` non vedono i VFX, vedono il multiplier 1.15× sul `DamageDealt` payload.

---

## §7 — Verdetto

Passive **Full FSM + listener** (sub-variant C — State-watch, `02-02e §A.0/§A.1`):
- **Vince** sull'isolazione dual-role: il listener pusha signal, l'FSM passivo consuma su edge, la skill-FSM resta separata. Bus è il mediatore.
- **Vince** sulla legibility editor-side: 3 nodi (Dormant/Armed/Boosted) + 4 edge ispezionabili con lo stesso tooling delle skill FSM.
- **Espone** 2 gap reali: (G9) pre-damage hook richiede design buff-applier vs kernel-reader (chiuso in §8 G9); (G10) RoundId nel `CombatState` (action item codebase, non architetturale).
- **Bene**: nessun nuovo verbo Command. Il listener vive in pure Rust; l'FSM passivo vive in RON come le skill FSM.

---

## §8 — Aggregato — gap stress test Agumon (cross-file)

Sintesi raccolta da 01/02/03/04. **Stato post round-2:** decisioni canon scritte nei rispettivi file §8.

| # | Gap | File | Severità | Decisione (round-2) |
|---|---|---|---|---|
| G1 | `SkillDef.params` mancante | 01, 02 | **Alta** | ✅ `params: HashMap<String, ParamValue>` esteso schema RON. Vedi 01/§8. |
| G2 | `EmitDamage` no `tough_break` | 02 | Media | ✅ Scelta A: campo opzionale `tough_break_param: Option<ParamRef>`. Vedi 02/§8. |
| G3 | `EmitStatus` no `stacks_param` | 02 | Media | ✅ Aggiunto `stacks_param: Option<ParamRef>` (None=1). Vedi 02/§8. |
| G4 | Order Commands multipli `on_enter` | 02 | Media | ✅ Ordine RON = ordine emission, deterministico. Vedi 01/§8. |
| G5 | Param source kind: Snapshot vs EventPayload | 03 | **Alta** | ✅ `ParamRef::Snapshot \| EventPayload \| BlueprintState`. `multiplier_chain: Vec<ParamRef>` su EmitDamage. Vedi 03/§8. |
| G6 | Multi-target damage | 03 | Media | ✅ Scelta A: blueprint emette N Command, shape resolver blueprint-side. `TargetRef` enum nel Command. Vedi 03/§8. |
| G7 | Frame budget mismatch atlas vs FSM | 03 | Media | ✅ Recovery in 2 varianti via edge priority, padding sull'ultimo nodo della path più corta. Vedi 03/§8. |
| G8 | QTE window > node frames | 03 | Media | ✅ Suspend pausato frame counter; Resume da frame snapshot pre-suspend; risultato in `blueprint_state["last_qte_result"]`. Vedi 03/§8. |
| G9 | Pre-damage hook | 04 | **Alta** | ✅ Pattern buff-applier + kernel-reader cascade. Vedi §8.G9 sotto. |
| G10 | RoundId nel `CombatState` | 04 | Bassa | 🟡 **Action item separato:** verificare in `src/combat/state.rs`, aggiungere se mancante. |
| G11 | Ult charge trigger `OnBasicAttack` vs `OnAnyAttack` | 02 | Bassa | ✅ Rename → `OnAnyAttack` (basic + heavy charge). Vedi 01/§8 e 02/§8. |
| G12 | Modifier-firma `OnKill→Detonate` default | 03 | Bassa | ✅ Sempre attivo, parte FSM base; skill_tree può solo aggiungere overlay. Vedi 03/§8. |

### G9 — Pre-damage hook vs post-event modifier **[ALTA]** ✅

**Decisione canon:** pattern **buff-applier + kernel-reader cascade**.

```
Listener (passive)         Damage Pipeline (kernel, src/combat/damage.rs)
─────────────────          ───────────────────────────────────────────
Twin Core fire-side:        compute_damage(caster, target, base_amount, tag) {
  on StatusApplied(            let mut amt = base_amount;
    status=Chilled,
    caster=Gabumon            // §2.8 cascade ordering (canon):
  ) ──▶ apply_buff(            //   1) base damage (mul_param × ATK / DEF)
    Self_,                     amt = apply_caster_buffs(caster, amt, tag);
    "twin_core_fire_active",   //      ↑ legge buff "twin_core_fire_active" → ×1.15
    expires_on: RoundEnd       amt = apply_target_debuffs(target, amt, tag);
  )                            amt = apply_dr(target, amt, tag);     // G-DR (§9)
                               emit_event(DamageDealt{ caster, target, amount: amt, tag });
                             }
```

**Componenti del pattern:**

1. **Listener** (passive blueprint, pure Rust): ascolta `CombatEvent`, applica **Buff component** sull'entità appropriata (self/ally). Nessuna modifica diretta del damage.
2. **Buff component** (gameplay state): `Buff { id: BuffId, mul: Multiplier, dur: BuffDuration }` con `BuffDuration ∈ { Turns(u8), Permanent, UntilRoundEnd }`. `BuffId("twin_core_fire_active")` per Agumon, simmetrico ice-side per Gabumon.
3. **Damage pipeline** (`src/combat/damage.rs`): durante `compute_damage`, legge tutti i `Buff` del caster e applica multiplier prima di emettere `DamageDealt`.
4. **Round-end cleanup**: sistema esistente o nuovo che droppa buff con `expires_on: RoundEnd` su `TurnEnded` last-of-round / `RoundEnded` event.

**Cascade ordering canon §2.8:**

```
1. base_damage = mul_param × caster.ATK / target.DEF
2. apply caster buffs (positive mul: Twin Core, Power Up, ecc.)
3. apply target debuffs (vulnerability, Heated dmg boost, ecc.)
4. apply DR (target armor, Fur Cloak, ecc.) — additivo cap 50%
5. emit `IncomingDamage` event (pre-damage hook, vedi §9)
6. emit `DamageDealt` event (post-fact, kernel ha già applicato)
```

**Listener ordering deterministico:**

Ordine di registrazione = ordine team slot (party slot 0..5 → enemy slot 0..4). Allineato a Bevy SystemSet ordering. Doc §2.2b §K.

**Niente nuovo evento.** `CombatEvent::DamageModifierRequest` (opzione B) **rigettato**: introduce roundtrip e ordering ambiguity. Cascade-in-pipeline è single-pass, deterministico.

---

## §9 — Decisioni cross-roster (round-2 nuovi gap)

Decisioni dai gap emersi durante i brief Dorumon/Gabumon/Patamon/Renamon/Tentomon. Riferimento `_CONTINUE.md` righe 12-49. Tutte canon prima di scrivere i blueprint Rust in M017.

### G-Sel — Selectors / `TargetShape` esteso ✅

```rust
pub enum TargetShape {
    Primary,
    Self_,
    AdjLeft,
    AdjRight,
    Blast(TargetRef),                                  // primary + 2 adj
    AdjLowestHp { side: Side },                        // alleato con HP più bassa adj
    LowestHpPctAlive { side: Side },                   // lowest HP% nel target side
    RandomEnemyAlive { seed: SeedSource },             // SeedSource ∈ {TurnRng, CombatRng}
    AoE { side: Side, exclude_dead: bool },            // tutti i target del side
    SingleAlly { slot: Option<u8> },                   // None = chooser via UI / AI
    Bounce { hits: u8, selector: Box<TargetShape> },   // chain N hits, re-resolve ogni hop
}
```

**Regole:**
- `TargetShape` vive nel blueprint (§5/§6 commit-time resolver).
- Blueprint chiama `resolve_shape(shape, ctx) -> Vec<TargetRef>` e emette N Command (uno per `TargetRef`). Vedi G6.
- `RandomEnemyAlive` usa `TurnRng` di default (deterministico, seedato dal turn counter). `CombatRng` solo per random fuori-turno (rare).
- `Bounce` re-risolve il `selector` interno ad ogni hop — supporta "chain bounce a target diverso ogni volta". Hit count cap = `hits`.
- Nessun shape inventato lazy: estensioni passano per design review.

### G-Verbs — Vocabolario `Command` esteso ✅

```rust
pub enum Command {
    // — gameplay (eseguite headless) —
    EmitDamage(EmitDamageArgs),       // G2/G5/G6
    EmitStatus(EmitStatusArgs),       // G3
    EmitHeal(EmitHealArgs),           // ← NEW
    EmitCleanse(EmitCleanseArgs),     // ← NEW
    EmitSpGrant(EmitSpGrantArgs),     // ← NEW
    ApplyBuff(ApplyBuffArgs),         // ← NEW (unificato self/ally)
    AdvanceTurn(AdvanceTurnArgs),     // ← NEW
    DelayTurn(DelayTurnArgs),         // ← NEW
    BlockReaction(BlockReactionArgs), // ← NEW
    SetBlueprintState(SetBlueprintStateArgs), // ← NEW (per FSM custom)
    StartQTE(StartQTEArgs),

    // — presentation (no-op headless) —
    Shake(ShakeArgs),
    SpawnParticle(SpawnParticleArgs),
    PlaySound(PlaySoundArgs),
}
```

**Firme principali:**

```rust
pub struct EmitHealArgs {
    pub multiplier_chain: Vec<ParamRef>,
    pub target_ref:       TargetRef,
}

pub struct EmitCleanseArgs {
    pub target_ref:    TargetRef,
    pub tag_filter:    CleanseFilter,    // All | Negative | Positive | ById(StatusId)
}

pub struct EmitSpGrantArgs {
    pub amount_param:  ParamRef,         // Snapshot generalmente
    pub target_ref:    TargetRef,        // Self_ o SingleAlly
}

pub struct ApplyBuffArgs {
    pub id:            BuffId,
    pub target_ref:    TargetRef,        // Self_ / SingleAlly / AdjLeft / ...
    pub mul_param:     Option<ParamRef>, // multiplier opzionale
    pub dur:           BuffDuration,     // Turns(n) | Permanent | UntilRoundEnd
}

pub struct AdvanceTurnArgs {
    pub target_ref:    TargetRef,
    pub amount:        i8,               // pos = anticipa, neg = posticipa (alias DelayTurn)
}

pub struct BlockReactionArgs {
    pub kind:          ReactionKind,     // FollowUp | Counter | All
    pub target_ref:    TargetRef,
    pub dur:           BuffDuration,
}

pub struct SetBlueprintStateArgs {
    pub state_key:     String,           // es. "twin_core_fire_active", "battery_charge"
    pub value:         ParamValue,       // Int/Float/Bool/Str
}
```

**Rationale:**
- `ApplyBuff` unifica le varianti frammentate viste nei brief Dorumon (Predator Loop), Gabumon (Fur Cloak), Patamon (Holy Aegis), Renamon (Kitsune Grace). Niente più `SelfBuff` / `AllyBuff` separati.
- `AdvanceTurn(amount=-N)` = `DelayTurn`; tenere alias solo se serve leggibilità in RON.
- `SetBlueprintState` è il **canale ufficiale** per FSM custom (Twin Core, Predator Loop, Battery Loop). Lo stato vive sul blueprint listener; le Command lo leggono via `ParamRef::BlueprintState(key)`.

### G-Pred — Predicate esteso ✅

```rust
pub enum Predicate {
    TimeInNode(u8),
    KernelEvent(KernelEventFilter),
    BlueprintState { state_key: String, expected: ParamValue },   // ← NEW
    UnitAlive(TargetRef),
    HpPctBelow { target_ref: TargetRef, pct: u8 },
}
```

**`BlueprintState`** consente edge condizionali su FSM custom (es. Predator Loop edge: `BlueprintState { state_key: "predator_charge", expected: Int(3) }` → trigger discharge node). Read-only — niente side-effect dall'edge eval.

### G-Param — `ParamRef` esteso con `BlueprintState` ✅

```rust
pub enum ParamRef {
    Snapshot(String),
    EventPayload(String),
    BlueprintState(String),     // ← NEW: legge blueprint_state[key]
}
```

`BlueprintState(key)` permette ai Command di leggere lo stato custom impostato da `SetBlueprintState` o dal listener (es. `EmitDamage` con `multiplier_chain: [Snapshot("base_mul"), BlueprintState("predator_charge_mul")]`).

### G-Events — Event bus esteso ✅

```rust
pub enum CombatEvent {
    // — esistenti —
    DamageDealt { caster, target, amount, tag, skill_kind },
    StatusApplied { caster, target, status, stacks, dur },
    SpEarned { actor, amount },
    UltimateCharged { actor, amount },
    UnitDied { unit, status_remaining },
    Broken { unit },
    PredatorLoopResolved { /* ... */ },

    // — NEW round-2 —
    CombatStarted { teams: TeamComposition },
    UltimateUsed { caster, skill_id, target },
    IncomingDamage { caster, target, base_amount, tag },  // pre-damage hook, BEFORE DR/cascade emit DamageDealt
    TurnEnded { actor, was_last_in_round: bool },
    RoundEnded { round_id: u32 },                          // implicato da `expires_on: RoundEnd`
}
```

**Note:**
- `IncomingDamage` è il pre-damage hook ufficiale per shield/DR/reaction listener. Cascade ordering §2.8 step 5 (vedi G9). Listener possono solo *osservare*; modifiche al damage vanno via Buff component, non via mutating event handler.
- `UltimateUsed` triggera ultimate-related listener (es. team buff "on any ally ult").
- `TurnEnded { was_last_in_round }` evita la necessità di un evento `RoundEnded` separato in molti casi; `RoundEnded` esiste comunque per i listener `expires_on: RoundEnd` cleanup system.

### G-DR — Damage Reduction stacking ✅

**Canon §2.8 cascade step 4:** DR stacking **additivo**, cap totale **50%**.

```rust
// src/combat/damage.rs (signature target)
fn apply_dr(target: Entity, amount: f32, tag: DamageTag, world: &World) -> f32 {
    let total_dr = world.iter_dr_sources(target, tag)        // base armor + Fur Cloak + ...
        .map(|s| s.pct as f32 / 100.0)
        .sum::<f32>()
        .min(0.50);                                          // cap 50%
    amount * (1.0 - total_dr)
}
```

**Regole:**
- Stacking additivo: 20% armor + 15% Fur Cloak + 30% temp buff = 65%, capped a 50%.
- Cap **per damage instance**, non per-tag (Fire/Ice/Phys condividono lo stesso cap totale sul target).
- Visibile nel `IncomingDamage` event payload come `dr_applied_pct` (debug/log only).

### G-Tag — `Electric` tag aggiunto ✅

```rust
pub enum DamageTag {
    Physical, Fire, Ice, Holy, Dark, Electric,  // ← NEW: Tentomon kit
    // (futuro: Wind, Water, Earth, Steel, Plant — espansione roster)
}
```

### G-Status — `Paralyzed` ✅

**Action item:** verificare in `src/combat/status_effect.rs` se `StatusId::Paralyzed` esiste. Se no, aggiungere:

```rust
pub enum StatusId {
    Heated, Chilled, Bleed, Poisoned, /* ... */
    Paralyzed,        // ← NEW (Tentomon Super Shocker / Battery Loop discharge)
}
```

**Semantica Paralyzed:**
- Durata: `Turns(n)`, default n=1.
- On turn start (durante `Paralyzed`): roll `paralysis_skip_chance` (default 25%). Se success → skip turn (no action), emit `CombatEvent::TurnSkipped { actor, reason: Paralyzed }`.
- Tick: -1 turn at `TurnEnded` (own turn) come tutti gli status.
- Stacking: max 3 stack, chance scala +10% per stack (25/35/45%).

### G-Buff — Durata `Permanent` ✅

```rust
pub enum BuffDuration {
    Turns(u8),
    UntilRoundEnd,
    Permanent,          // ← NEW: dura tutto il combat fino al dispel/cleanse
}
```

**Regole:** `Permanent` buff non vengono droppati dal round-end cleanup; vivono finché un `EmitCleanse` o `UnitDied` li rimuove. Pensati per i Twin Core ("partner alive + in team" → permanent buff durante quel combat).

---

**Stato post round-2:** **0 gap rossi aperti.** I 12 gap originali Agumon più 13 cross-roster nuovi sono tutti chiusi a livello di design. Pronto per:
1. Riprendere brief Digimon (Dorumon/Gabumon/Patamon/Renamon/Tentomon) con redirect a queste decisioni canon, OPPURE
2. Iniziare scrittura schema RON/Rust target (`src/data/skills_ron.rs` + `src/combat/blueprints/*`) per M017.

**Action item residui (verificare codebase prima di M017):**
- G10: `RoundId` in `src/combat/state.rs` — verificare/aggiungere.
- G-Status: `Paralyzed` in `src/combat/status_effect.rs` — verificare/aggiungere.
- G-Tag: `Electric` in damage tag enum — aggiungere.
