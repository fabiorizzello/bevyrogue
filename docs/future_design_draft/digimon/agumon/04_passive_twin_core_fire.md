# Agumon — Passive: `twin_core_fire` (listener-only, no FSM)

> **Goal**: validare il **dual-role blueprint** §2.2b §B. Il blueprint Agumon è **executor** durante una sua skill (file 01–03) e **listener** sempre, ascoltando `CombatEvent` per applicare effetti passivi. I due ruoli **non si parlano internamente**.

## §1 — Intent

Twin Core fire-side (canon §8): **+damage condizionale** se Gabumon (Twin Core partner) è in team e ha applicato `Chilled` su un nemico nello stesso round.

- **Direzione:** fire-side aumenta quando l'ice-side ha "armato" il bersaglio
- **Reciprocità:** Gabumon ha specularmente `twin_core_ice` che ascolta `StatusApplied(Heated)` da Agumon (file separato, futura sessione Gabumon)
- **Scope:** è una **passive listener-only**. No FSM, no animation extra.

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
  Gabumon FSM (bubble_blast) → EmitStatus(Chilled, target=enemy1)
    └─ CombatEvent::StatusApplied { caster: gabumon, target: enemy1, status: Chilled }
       └─ Agumon listener cattura
          └─ aggiunge self-buff "twin_core_fire_active" (round-scoped)

turno T+1 (Agumon attiva)
  Agumon FSM (pepper_breath/nova_blast/claw_strike) → EmitDamage(...)
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

- Dual-role pattern (listener vs executor) è **chiaramente separato**: il listener non sa che esiste una FSM in esecuzione; legge solo `CombatEvent`. La FSM non sa che esiste un listener; emette Commands → KernelEffect → `CombatEvent`. Il bus è il mediatore. **Pulito.**
- Round-scope buff è facilmente esprimibile come `Buff { expires_on: RoundEnd }`. Sistema esistente nel kernel (status durations) lo supporta concettualmente.

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

## §7 — Verdetto

Passive listener-only:
- **Vince** sull'isolazione dual-role (FSM e listener disaccoppiati via bus).
- **Espone** 2 gap reali: (1) pre-damage hook richiede design buff-applier vs kernel-reader; (3) RoundId nel `CombatState`.
- **Bene**: nessun nuovo verbo Command. Il listener vive in pure Rust, non in RON.

---

## §8 — Aggregato — gap stress test Agumon (cross-file)

Sintesi raccolta da 01/02/03/04. Da rivedere prima di M017.

| # | Gap | File | Severità | Azione |
|---|---|---|---|---|
| G1 | `SkillDef.params: HashMap<String, Value>` mancante | 01, 02 | **Alta** | Estendere schema RON `skills.ron` |
| G2 | `EmitDamage` non supporta `tough_break` | 02 | **Media** | Estendere campo verbo |
| G3 | `EmitStatus` non supporta `stacks_param` | 02 | **Media** | Estendere campo verbo |
| G4 | Order semantics di Commands multipli su `on_enter` | 02 | Media | Doc §2.2b: ordine RON = deterministico |
| G5 | Param source kind: Snapshot vs EventPayload | 03 | **Alta** | Estendere param model con 2 cluster |
| G6 | Multi-target damage: 3 emit vs targeting nel verbo | 03 | Media | Scelta A (3 emits, blueprint resolve at-commit) |
| G7 | Frame budget mismatch atlas vs FSM nel ramo opzionale | 03 | Media | Variant nodi Recovery via edge priority |
| G8 | QTE window > node frames; suspend resume contract | 03 | Media | Doc §H esempio esplicito |
| G9 | Pre-damage hook vs post-event modifier | 04 | **Alta** | Decisione: buff-applier + kernel-reader cascade §2.8 |
| G10 | RoundId nel `CombatState` per round-scoped buff | 04 | Bassa | Verificare esistente, aggiungere se mancante |
| G11 | Ult charge accumulation trigger ambiguo (`OnBasicAttack` vs `OnAnyAttack`) | 02 | Bassa | Game-design decision, non FSM |
| G12 | Modifier-firma `OnKill→Detonate` come default o unlock? | 03 | Bassa | Canon §8 implica default; conferma esplicita |

**Top 3 da risolvere prima di scrivere altri 5 Digimon:**
- **G1** (params plumbing) — senza, le Commands sono inline-literal e rompono data/logic separation
- **G5** (param source kind) — vincola tutto il design event-reactive
- **G9** (damage modifier pattern) — definisce come vivono i passive

**Proposta operativa:** prima di Gabumon, dedicare 1 file `_findings_round1.md` (o un addendum a §2.2b) che chiude G1/G5/G9 a livello di design. Poi continuare i Digimon su base solida.
