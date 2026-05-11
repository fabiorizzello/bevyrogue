# §2.7 — Skill-as-Plugin — kernel chiuso senza catalogo di skill primitives

**Design driver:** un catalogo chiuso di `Step` / `StepTarget` / `Condition` / `damage formula` parsato da RON costringe a **allargare il kernel** per ogni nuovo pattern di skill (kill→nearby, marked AOE, multi-fase, branch su damage, ecc.). Ogni variante nuova = modifica al kernel + bump RON `version`. Una skill esotica ("colpisci, se hai colpito un alleato curalo, se hai colpito un nemico applica burn, scaling con HP%, e branche di branche") forza ad aggiungere `Step` finché il vocabolario diventa una skill scripting language.

**Decisione:** la skill è un **plugin Bevy come il blueprint del Digimon** (§2.3). Il kernel non ha catalogo di gameplay primitives (no `Step`, no `damage formula`, no `StepTarget`). Ha solo:

1. **Un trait** `SkillBehavior` con 3 metodi che la skill **deve** esporre per allacciarsi al motore.
2. **Un canale di effetti** `KernelEffect` — già esistente da M016 — che è l'**unico** modo in cui la skill muta lo stato.
3. **Un meccanismo suspend/resume** (§2.6) — che resta perché serve a QTE, animation gate, ecc.

## A — Il trait

```rust
pub trait SkillBehavior: Send + Sync + 'static {
    /// READ-ONLY. Calcola cosa la skill mostrerebbe come bersagli e stima del risultato.
    /// Chiamata su hover / selezione nella UI. Deve essere pura (nessuna mutazione di world).
    fn preview(&self, ctx: &SkillPreviewCtx) -> SkillPreview;

    /// READ-ONLY. Verifica se la skill è eseguibile *ora*. Risposta = OK o lista motivi.
    /// Il kernel usa questo per: highlight UI, AI filtering, action-queue gate.
    fn legality(&self, ctx: &SkillLegalityCtx) -> Result<(), Vec<IllegalReason>>;

    /// MUTATING via effetti. Esegue la skill. Riceve ctx pausabile (§2.6).
    /// Ritorna Done o Suspend{reason, cursor}. Emette KernelEffect, non muta direttamente.
    fn execute(&mut self, ctx: &mut SkillExecCtx) -> SkillStepOutcome;

    /// Metadata statico (per editor §2.5, contract test, action button).
    fn manifest(&self) -> &SkillManifest;

    /// READ-ONLY. Lista di filtri evento a cui la behavior vuole essere notificata.
    /// Default vuoto: la skill non reagisce a niente. Opt-in per kind B/C/D/E.
    fn subscribes_to(&self) -> &[EventFilter] { &[] }

    /// MUTATING via effetti. Dispatch reactive hook. Chiamata dal kernel quando passa un
    /// `CombatEvent` che matcha uno dei filtri. Stessa contract di `execute` — niente
    /// `&mut World`, solo `KernelEffect`/transition via `ctx`.
    fn on_event(&self, ev: &CombatEvent, ctx: &mut SkillHookCtx) {}
}
```

I due metodi opt-in (`subscribes_to` + `on_event`) sono il **trait surface dei reactive hook** dei blueprint — formalizzati in §C2 sotto. Default vuoti: una skill semplice (Pepper Breath, Bubble Blast) li ignora.

## B — Cosa resta in RON

Una skill in RON diventa **identità + parametri**, mai logica:

```ron
"agumon_pepper_breath": SkillRon(
    version: 2,
    behavior: "agumon::pepper_breath",   // ID che resolva un SkillBehavior nel registry
    display: SkillDisplay(
        name_loc: "skill.agumon.pepper_breath",
        icon: "skills/fire.png",
        animation_track: "agumon/skill_a",
    ),
    tags: ["fire", "ranged", "single"],
    // Free-form numerici/params che la behavior consuma.
    // Tipo validato a boot dal manifest della behavior.
    params: {
        "atk_mul": 1.6,
        "burn_chance_pct": 30,
        "burn_duration": 2,
        "sp_cost": 1,
        "refund_on_kill": true,
    },
),
```

I **numeri restano in RON** (regola §2.1 invariata): la behavior reads `ctx.params.f32("atk_mul")` invece di averli hardcoded. La *forma* dei parametri è libera per skill, la *logica* sta in Rust, il **catalogo** dei comportamenti possibili non esiste.

## C — I tre allacciamenti al motore

**Preview surface — chi verrà colpito + stima:**

```rust
SkillPreview {
    targets: SmallVec<[Entity; 8]>,      // bersagli illuminabili in UI
    damage_estimate: Option<DamageRange>, // (min, max) per ogni target — opzionale
    sp_delta: i8,                         // costo previsto (post refund/surcharge)
    secondary_effects: Vec<PreviewBadge>, // "burn 2t", "knock back", "self-buff"
    warnings: Vec<PreviewWarning>,        // "may miss adjacent due to wall", "consumes ult"
}
```

Esempio: la skill "kill→nearby" rilegge `last_struck_target` dalla preview state, calcola "se ammazzo, colpirei A e B" e popola `targets: [P, A, B]` + `warnings: ["A/B colpiti solo se P muore"]`. Tutto questo lo fa la behavior, non il kernel.

**Legality surface — la skill può partire?**

```rust
IllegalReason {
    NotEnoughSp { needed: u8, have: u8 },
    OnCooldown { remaining: u8 },
    InvalidTarget,
    SelfStunned,
    BlueprintGate(String),    // motivo custom del blueprint (es. "battery off")
}
```

`legality()` esegue di nuovo nel kernel anche dopo la selezione, prima di committare l'azione — è il single point of truth per "questa skill può eseguire ora". UI grigia il bottone, AI scarta la mossa, contract test verifica determinismo.

**Execute surface — la skill prende il controllo:**

```rust
impl SkillBehavior for PepperBreath {
    fn execute(&mut self, ctx: &mut SkillExecCtx) -> SkillStepOutcome {
        let target = ctx.initial_target();
        let dmg = ctx.params.f32("atk_mul") * ctx.caster().atk();
        ctx.emit(KernelEffect::Damage { target, amount: dmg, source: ctx.action_id() });
        if ctx.rng_bool_pct(ctx.params.u8("burn_chance_pct")) {
            ctx.emit(KernelEffect::ApplyStatus {
                target,
                status: "burn",
                stacks: ctx.params.u8("burn_duration"),
            });
        }
        // Branch "kill→nearby" — pure Rust, niente vocabolario chiuso:
        if ctx.last_effect_killed(target) {
            for adj in ctx.adjacent_enemies_of(target) {
                let bonus = ctx.params.f32("atk_mul") * 0.4 * ctx.caster().atk();
                ctx.emit(KernelEffect::Damage { target: adj, amount: bonus, .. });
            }
            if ctx.params.bool("refund_on_kill") {
                ctx.emit(KernelEffect::RefundSp { caster: ctx.caster_id(), amount: 1 });
            }
        }
        SkillStepOutcome::Done
    }
}
```

Il kernel **non** sa nulla di "Strike", "Branch", "StepTarget". Vede solo `KernelEffect::Damage`, `ApplyStatus`, `RefundSp` ecc. — che sono **primitives di stato**, non di gameplay. La differenza è cruciale: `Damage` è "muta -X hp", non "applica strike step"; il kernel resta data-driven, la skill logica.

## C2 — Reactive hook surface (tassonomia A–F)

Il **follow-up attack** classico (Agumon `Pyre Reignite`) non è l'unica forma di reazione: i blueprint dei 6 Digimon (vedi §08) richiedono **6 kind ortogonali** di hook. Il kernel non hardcoda nessun kind — espone solo il **dispatcher** `subscribes_to` + `on_event`. La behavior (o il blueprint, via behavior bridge) sceglie la forma.

| Kind | Forma | Trigger surface | Output via `SkillHookCtx` |
|---|---|---|---|
| **A. Follow-up attack** | enqueue skill in `FollowUpQueue` (FIFO, esistente da M015) | `CombatEvent` filter (es. `ToughnessBroken`) | `ctx.enqueue_follow_up(skill_id, target)` |
| **B. Threshold auto-trigger** | meter ≥ N → auto-emit effetto (no attacco) | `CombatKernelTransition` watch sul proprio kernel state | `ctx.emit(KernelEffect::*)` / `ctx.signal(...)` |
| **C. Cross-unit listener** | ally event → muta self/team | `CombatEvent` filtrato su `caster == ally` | transition / status / signal |
| **D. Phase transition** | event → avanza state machine privata | `CombatEvent` filter (es. `SkillResolved`) | transition only (su kernel state proprio) |
| **E. Conditional grant** | event → conferisce buff/mark | `CombatEvent` filter (es. `AllyHpLow`) | `ApplyStatus` / transition |
| **F. Gate / consumer** | mark o phase esiste → skill X legal o consumata | `legality()` query + pre-resolve consume | `IllegalReason::BlueprintGate(...)` + post-resolve transition |

**Punto chiave kernel-side:** A è già implementato come consumer specializzato del bus (`FollowUpQueue`). B/C/D/E sono **tutti lo stesso meccanismo**: `subscribes_to() → filter match → on_event()`. F vive sul lato `legality()`, non sul bus — perché è una query, non una notifica.

```rust
pub enum EventFilter {
    /// Qualunque CombatEvent dove caster è una specifica entità (es. ally).
    CasterIs(EntityRef),
    /// Tutti gli SkillCast con tag x (es. SkillCast{tag: "ice"}).
    SkillCastWithTag(&'static str),
    /// Transizioni del proprio kernel state (es. cross_resonance threshold reached).
    KernelTransition(&'static str),
    /// Generic event kind filter (es. ToughnessBroken, HpDropped).
    EventKind(CombatEventKindMask),
    /// Combinatori (And, Or, Not) per filter composition.
    And(Box<EventFilter>, Box<EventFilter>),
    Or(Box<EventFilter>, Box<EventFilter>),
}
```

`SkillHookCtx` espone lo **stesso** mutation channel di `SkillExecCtx` (solo `emit(KernelEffect)`, RNG seedato, niente `&mut World`) → la determinism story di §E vale identica.

**Dispatcher contract:**

1. Boot — il kernel raccoglie `subscribes_to()` da tutte le skill+blueprint registrate, costruisce un indice `EventKind → Vec<HookId>` (no scan O(N) per evento).
2. Run — quando un `CombatEvent` è pubblicato sul bus, il dispatcher fa lookup, invoca `on_event` solo per gli hook con filter matching, in ordine deterministico (`hook_id` ascending — niente race con N hook che reagiscono al medesimo evento).
3. Re-entry — un `on_event` può emettere un nuovo `CombatEvent` (es. spark che apre `ThermalSpark`). Il dispatcher accoda, **non** invoca ricorsivamente; resolve loop di `02-08` (effect cascade) drena come al solito.
4. Cap iterazioni — stessa policy di §E: hard cap su effetti emessi per hook tick (es. 16), diagnostic + termination se sfora.

**Blueprint vs skill ownership.** Un hook può appartenere o a una **skill specifica** (raro: vive solo se la skill è in roster) o al **blueprint del Digimon** (comune: passive cross-skill, es. Tentomon Battery threshold). I blueprint hanno la stessa surface `SkillBehavior` (§2.3 ha già wirato i Digimon come plugin): registrano hook tramite un `BlueprintBehavior` wrapper che riusa `subscribes_to`/`on_event`. Niente nuovo trait per i blueprint.

**Mapping ai 6 Digimon canon (per riferimento — pieno dettaglio in §08 §0 e §8):**

| Digimon | Hook kind richiesti |
|---|---|
| Agumon | A (Pyre Reignite), E (Heated boost da ally ice) |
| Gabumon | A (mirror Agumon), E (Chilled boost da ally fire) |
| Patamon | E (Grace su ally HP low), F (Martyr Light gate) |
| Dorumon | C (Exploit build su skill cast), F (Berserk gate, PreyLock consumer) |
| Tentomon | B (StaticCharge≥3 → team energy), C (ally cast → +1 charge) |
| Renamon | D (Mind-Game phase transitions), F (skill legality per phase) |

Tutti e 6 i kind sono coperti dal roster Rookie → il dispatcher A–F è blocking per M017, non deferibile.

## D — Suspend/resume integration

`SkillExecCtx::request_yield(YieldReason)` ritorna `Suspend { cursor }`. La skill è una state machine privata interna alla behavior — non più un cursor su un `Vec<Step>` parsato da RON. Esempi:

- QTE: `ctx.request_qte(TimedPress(500ms))` → `Suspend(QuickTimeEvent)`; al resume, `ctx.qte_result()` decide il branch in Rust.
- Animation gate: `ctx.await_anim_marker("impact")` → suspend finché l'event track raggiunge il marker.
- Blueprint coupling: la skill può chiamare il blueprint del Digimon proprietario via `ctx.signal_blueprint("battery_drain")` e attendere risposta. Già coerente con §2.3.

È la **skill** che decide quando sospendere, in pure Rust (non un parser RON che riconosce uno step `QuickTimeEvent`). Il kernel sa solo: "behavior ha ritornato `Suspend`, freeze fino a `SkillYieldResolved`".

## E — Determinismo + sicurezza (la parte che si paga per la libertà)

La libertà ha un prezzo: senza enum chiuso, il kernel non può più **controllare** cosa fa la skill. Mitigazioni:

| Rischio | Mitigazione |
|---|---|
| Skill non deterministica (usa RNG global, wall-clock, side-channel) | `SkillExecCtx` espone **solo** RNG seedato + tick logico. Niente `std::time`, niente `rand::thread_rng`. Clippy lint custom + contract test |
| Skill muta direttamente world e bypassa observability | `SkillExecCtx` non dà `&mut World`. Solo `emit(KernelEffect)`. Effects sono il single mutation channel — riusa pattern M016 |
| Skill panica o entra in loop | Cap iterazioni hard al boot (es. 64 effetti per execute step); se sfora → `KernelEffect::Diagnostic("skill_runaway")` + skill termina forzatamente |
| Skill non aggiorna preview consistentemente con execute (UI mente) | Contract test golden: per ogni skill, eseguire `preview` + `execute` in parallel con stesso seed, asserire `preview.targets ⊇ execute_emitted_targets` e `preview.damage_estimate` contiene `actual_damage` |
| Skill dimentica caso headless (QTE blocca AI test) | `manifest()` dichiara `headless_default_for_yield`; contract test verifica che ogni yield reason abbia default. Stesso pattern §2.6 |
| Skill bypassa legality (kernel chiama execute senza legality OK) | Kernel **non** chiama `execute` se `legality` non passa. Resolver lo enforce |
| Editor non sa che parametri esporre | `SkillManifest::params_schema()` enum `ParamKind { Float{min,max,default}, Int, Bool, StatusRef, ... }`. Editor leva da lì il form |
| Reactive hook crea ciclo infinito (A emette evento che ritriggera A) | Dispatcher non chiama `on_event` ricorsivamente: nuovi event accodati al bus, drenati dal cascade loop di §2.8 con cap. Hard cap per-tick (16 effetti/hook) + diagnostic |
| Ordine hook non deterministico (2 hook reagiscono allo stesso evento) | Indice dispatcher ordinato per `hook_id` ascending (assegnato a registry time, stabile). Contract test golden su replay con seed |
| Hook bypassa filter e legge tutto il bus | `SkillHookCtx` espone solo `ev: &CombatEvent` passato dal dispatcher — niente accesso al bus completo. Filter forzato a boot |

## F — Quando una skill è "semplice" — opt-in helper library

Il 70-80% delle skill è "applica X danno a Y bersaglio, opzionale status". Costringere ognuna a essere una `impl SkillBehavior` è verboso. Forniamo **una behavior pre-cotta** `DeclarativeSkill` (in `combat::skill::declarative`) che espone un mini-DSL Rust:

```rust
declarative::strike()
    .damage_mul(1.5)
    .status("burn", 2)
    .cost_sp(1)
```

`DeclarativeSkill` è **una behavior tra tante**, non il modello universale. Una skill custom **non la tocca** e scrive `impl SkillBehavior` libero in Rust.

## G — Estensione del kernel = aggiungere KernelEffect

Le primitive di mutazione (`KernelEffect`) **sono** un catalogo chiuso, ma di stato base, non di gameplay:
`Damage`, `Heal`, `ApplyStatus`, `RemoveStatus`, `RefundSp`, `ConsumeSp`, `KnockBack`, `Move`, `EmitEvent`, `Diagnostic`. Aggiungere una nuova primitiva (es. `RecruitTemporaryAlly`) è una decisione esplicita kernel-side, con review, contract test e bump major. Questo è il **vero confine** stretto.

## H — Slice impact

- **S03b** "`SkillBehavior` trait + registry + `SkillExecCtx` + reactive hook dispatcher" — contiene anche la cascade drain loop (§2.8) e l'indice `EventFilter → HookId`.
- **S03c** "skill RON v2 + behavior porting" — ~30 skill esistenti migrate, la maggioranza via `DeclarativeSkill`, 2-3 custom Rust. Hook kind A esistenti (follow-up FIFO) restano wired via blueprint → behavior bridge.
- **S03d** "Kernel suspend/resume" (§2.6).

Vedi §5 per dettaglio dipendenze. §08 §11 elenca l'impatto cross-Digimon dei 6 hook kind.
