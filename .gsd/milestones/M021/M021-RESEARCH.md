# M021 — Research & Audit: Kernel ⇄ Digimon Identities Decoupling

**Scope.** Fondazione del combat: kernel agnostico, "1 Digimon = 1 plugin", API skill potente e auto-contenuta. Due fasce:

- **Fascia B** — `trait Blueprint` + `BlueprintRegistry` + migrazione 6 blueprint Digimon + kernel digimon-free.
- **Fascia A** — `trait Ability` (Skill ∪ Passive) + `SkillCtx` + `Intent` + interprete kernel/Bevy + skill auto-contenute.

---

## 1. Evidence — stato attuale

### 1.1 `kernel.rs` contiene domain digimon-specifici

`src/combat/kernel.rs` = **1393 LOC**. Definisce, oltre al core (Strain/Flow/Fatigue/Tag/Beat/TacticalCycle), enum e struct **per ogni mechanic identitaria**:

| Mechanic | Digimon | Tipi nel kernel | Linee |
|---|---|---|---|
| Twin Core | Agumon + Gabumon | `TwinCoreSignal`, `TwinCoreTransition` | 433, 445 |
| Battery Loop | Tentomon | `BatteryLoopChargeKind`, `BatteryLoopBlockedReason`, `BatteryLoopStep`, `BatteryLoopSignal`, `BatteryLoopTransition` | 509–549 |
| Precision Mind Game | Renamon | `PrecisionMindGamePhase`, `PrecisionMindGameRejectReason`, `PrecisionMindGameStep`, `PrecisionMindGameTransition` | 667–695 |
| Predator Loop | Dorumon | `PredatorLoopCapKind`, `PredatorLoopBlockedReason`, `PredatorLoopStep`, `PredatorLoopSignal`, `PredatorLoopTransition` | 747–787 |
| Holy Support | Patamon | `HolySupportSignal`, `HolySupportStep`, `HolySupportRejectReason`, `HolySupportTransition` | 913–941 |

E sopra a tutti, l'**enum chiuso** che il kernel routa internamente (`kernel.rs:889`):

```rust
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    TwinCore(_), BatteryLoop(_), HolySupport(_), PredatorLoop(_), PrecisionMindGame(_),
}
```

→ **Aggiungere un Digimon = aprire `kernel.rs` ed editare enum + apply system.** Non è plug-and-play.

### 1.2 Registrazione plugin asimmetrica

`src/combat/blueprints/mod.rs:110-135`: `const BLUEPRINTS: &[BlueprintRegistration]` con 6 entry hardcoded. Niente `trait Blueprint` — ogni file blueprint inventa la propria forma. `dispatch` produce **già** `CombatKernelTransition::*` typed: il blueprint deve conoscere che esiste una variant kernel apposita.

### 1.3 Asimmetria struttura plugin

```
src/combat/blueprints/
├── agumon/        ← mod.rs + identity.rs + signals.rs  (plugin idiomatico)
├── patamon/       ← mod.rs + identity.rs + signals.rs  (plugin idiomatico)
├── dorumon/       ← mod.rs + identity.rs + signals.rs + hooks.rs (plugin idiomatico)
├── gabumon.rs     ← 63 LOC flat — niente Plugin Bevy
├── renamon.rs     ← 40 LOC flat — niente Plugin Bevy
└── tentomon.rs    ← 70 LOC flat — niente Plugin Bevy
```

Solo 3/6 sono Bevy `Plugin`. Battery Loop e Precision Mind Game cablati altrove come system standalone (`kernel.rs:1069-1070`).

### 1.4 Shim compat ancora vivi

`src/combat/mod.rs` espone `pub use blueprints::agumon::identity as twin_core` (e 2 simili predator_loop, holy_support). Da rimuovere quando i 3 blueprint flat migrano a plugin.

### 1.5 Roster + ValidationSnapshot conoscono i nomi delle mechanic

- `RosterEntry` ha field hardcoded `twin_core`, `holy_support`, `battery_loop`, … invece di un blueprint-keyed payload.
- `ValidationSnapshot` ha field inline (es. `battery_loop`) — aggiungere un nuovo digimon richiede aggiornare lo struct.

### 1.6 Skill DSL data-driven (da sostituire)

`src/data/skills_ron.rs:SkillDef` ha 8 campi rilevanti: `id`, `display_name`, `category`, `cost`, `targeting`, `tags`, `effects: Vec<Effect>`, `custom_signals: Vec<SkillCustomSignal>`. L'apply pipeline (`resolution.rs:626 apply_effects`, ~200 LOC) collassa `Effect` in `ResolvedAction` flatten con 15+ campi paralleli. Spike `spike-skill-dsl-coverage` ha verificato copertura 24/24 skill canon — la DSL **regge**, ma è data-driven con logica nascosta nel collapse (Heal/Cleanse mutex, bounce hop selection, GrantFreeSkill target derivation). Va sostituita con `trait Ability` Rust per portare la logica esplicita nel blueprint.

---

## 2. Target architecture

### 2.1 Cosa il kernel **deve** sapere (core invariant)

- Turn order, AV gauge, TacticalCycle (windup/strike/recovery), Beat
- Strain (toughness/break), Flow (SP), Fatigue (Ult charge) — risorse universali
- Tag bus generico (CombatTag) → meccanismo, non lista chiusa
- Status taxonomy canon §H.1 (5 status) — vocabolario condiviso, non identitario
- Custom signal dispatch infrastructure (envelope `Blueprint { owner, signal, payload }`)
- **Intent applier**: damage formula, mitigation, break, status tick — single source of truth per i side-effect

### 2.2 Cosa il kernel **non deve** sapere

- Twin Core: heated stack rules, cross-resonance — è Agumon+Gabumon
- Battery Loop: charge kinds, transfer rules — è Tentomon
- Predator Loop: berserk cap, target tracking — è Dorumon
- Holy Support: grace gauge, martyr light — è Patamon
- Precision Mind Game: phase ladder — è Renamon
- Logica di **qualsiasi skill** — sta nel blueprint owner via `impl Ability`

### 2.3 Shape kernel post-M021

`CombatKernelTransition` perde le 5 variant digimon-specifiche:

```rust
// PRIMA: 11 variant
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    TwinCore(_), BatteryLoop(_), HolySupport(_), PredatorLoop(_), PrecisionMindGame(_),
}

// DOPO: 7 variant — 6 core + 1 opaque blueprint passthrough
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    Blueprint(BlueprintTransition),
}

pub struct BlueprintTransition {
    pub owner: &'static str,
    pub payload: Box<dyn Any + Send + Sync>,  // D006
}
```

Il system `apply_combat_kernel_transitions` matcha solo le 6 core; per `Blueprint(_)` chiama il plugin owner via registry. Componenti di stato delle mechanic → `Component` Bevy registrato dal plugin, **non** field dentro `CombatState` o `Unit`.

### 2.4 Cross-mechanic state (D005 shared-mechanic mini-plugin)

Twin Core cross-resonance (Agumon ↔ Gabumon): si crea un mini-plugin `blueprints/twin_core/` con `owner = "twin_core"`, ospita il `Component`/`Resource` condiviso. Agumon e Gabumon dipendono da `TwinCoreState`, non viceversa. Pattern riusabile per future shared mechanic.

Accesso runtime alla shared state da una skill: tipizzato via `ctx.blueprint_state::<TwinCoreState>()` (vedi §5.8).

---

## 3. moonshine_kind — stato

Già adottato dove serve: `Query<Instance<Unit>>` in `headless.rs:210`, `ui/combat_panel.rs:234-235`, `bin/combat_cli.rs:135`.

Residui legittimi: `Query<(Entity, &FloatingDamage)>` in `floating.rs:22` — usato solo per despawn, OK.

Residui da convertire (QQ post-M021): `av.rs:56` `pub unit_entity: Entity` → `Instance<Unit>`; `turn_system/mod.rs:380` payload entity → idem.

**Non bloccante M021.**

---

## 4. Fascia A — Design corrente

### 4.1 Principio: Skill API potenziata, fedeltà variabile

La skill non è una DSL statica né un blob di codice opaco. È **un oggetto interrogabile a tre fedeltà**:

| Fedeltà | Chi consuma | Cosa risponde |
|---|---|---|
| **Legality** | UI grayed, AI filter | "posso lanciarla adesso?" — risorse, fase, cooldown, blueprint-state precondition |
| **Preview** (input + impact) | UI picker + tooltip, AI scoring | "che selezione chiedo + cosa succede se la lancio su X?" |
| **Execute** | Kernel commit | "lanciala, applica gli effetti" |

Stessa skill, stessa `resolve()`, mode diverso del `SkillCtx`. Preview = drain Intent in `Mode::DryRun`, kernel non applica ma aggrega in `ImpactShape`.

### 4.2 RON = solo numeri (tuning sheet)

`assets/data/skills.ron` contiene **solo dati di balance**, mai logica. Tutto il resto (legality, input shape, cost, effetti, side-effect, hooks) vive in Rust dentro `impl Ability for X`.

Schema RON minimo dopo M021:

```ron
SkillTuning(
    id: "agumon_baby_flame",
    numbers: {
        "base_dmg": 100,
        "heated_bonus_pct": 30,
        "sp_cost": 30,
    },
)
```

Conseguenza: niente più `RonSkill` generico, ogni skill = un file `.rs` dedicato nel blueprint. `src/data/skills_ron.rs` collassa al loader che produce solo `HashMap<SkillId, SkillTuning>`. Aggiungere una skill = `blueprints/<x>/abilities/<name>.rs` + entry RON + `register_ability(MySkill)`.

Fuori dal RON:
- **`cost`** — può essere condizionale (skill enhanced = 0 SP). Vive in `Ability::cost(snap, actor)`.
- **`targeting`** — può essere condizionale (eleggibili variabili per buff/fase). Vive in `Ability::input_shape(snap, actor)`.
- **`effects` / `custom_signals`** — è logica, vive in `Ability::resolve()`.
- **`display_name` / `tags` / `category`** — vivono come `const` Rust nella skill o `SkillDisplay::tooltip_key` per i18n separata.

Tuning typing: `numbers` deserializzato in un `struct` typed per-skill (`struct BabyFlameNumbers { base_dmg: i32, heated_bonus_pct: i32, sp_cost: i32 }` con `derive(Deserialize)`), così i typo del RON sono fail-fast al boot.

### 4.3 Trait `Ability` unico (Skill ∪ Passive)

Skill attive e passive Digimon condividono ~70% di superficie (id, owner, display, hooks, cost contestuale). Un solo trait `Ability` con `kind: AbilityKind { Active, Passive }`, un solo `AbilityRegistry`, un solo `register_ability`. La distinzione attiva/passiva è dato runtime, non type system; copre naturalmente reactive skills come Patamon Holy Aegis (passiva con trigger su `UnitDied`).

Forma del trait (object-safe, supporta `Arc<dyn Ability>`):

```rust
// src/combat/api/ability.rs
pub trait Ability: Send + Sync + 'static {
    // Identity (cheap, statico, no snapshot)
    fn id(&self) -> AbilityId;
    fn owner(&self) -> BlueprintId;
    fn kind(&self) -> AbilityKind;
    fn category(&self) -> AbilityCategory;
    fn display(&self) -> &AbilityDisplay;
    fn tags(&self) -> AbilityTagSet;

    // Legality (cheap, no drain)
    fn legality(&self, snap: &Snapshot, actor: UnitId) -> Legality;

    // Cost (cheap, può leggere snapshot per condizionale via buff sull'actor)
    fn cost(&self, snap: &Snapshot, actor: UnitId) -> AbilityCost;

    // Input shape (Faccia A — picker UX; può variare a runtime via snapshot)
    fn input_shape(&self, snap: &Snapshot, actor: UnitId) -> InputShape;
    // InputShape::SingleEnemy { eligible } | NSelections | AoE | Self | None | Invalid { reason }

    // Resolve — unica produzione di Intent, mode-aware
    fn resolve(&self, ctx: &mut SkillCtx, target: TargetSelection);

    // Hooks cross-event (opt-in via default)
    fn hooks(&self) -> AbilityHooks { AbilityHooks::none() }
    fn on_event(&self, _ev: &CombatEvent, _ctx: &mut HookCtx) {}

    // Modifier passivi continuativi (opt-in)
    fn modifiers(&self, _snap: &Snapshot, _unit: UnitId) -> AbilityModifiers {
        AbilityModifiers::none()
    }
}
```

Object safety preservata: no `Self` in posizioni problematiche, no generic methods, no associated types non-bound.

### 4.4 Builder pattern per costruire le skill

Invece di scrivere `impl Ability` a mano per ogni skill, l'autore compone:

```rust
AbilityBuilder::active("agumon_baby_flame")
    .owner("agumon")
    .category(AbilityCategory::Skill)
    .tags(&[DamageTag::Fire])
    .tuning::<BabyFlameNumbers>(BABY_FLAME_HANDLE)
    .cost(|snap, actor, t| if snap.has_status(actor, Heated) { AbilityCost::free() } else { AbilityCost::sp(t.sp_cost) })
    .input(InputShape::SingleEnemy { side: Enemy, life: Alive })
    .legality(legality::self_alive_not_stunned)
    .resolve(|ctx, target, t| {
        ctx.deal(target.primary()).damage(t.base_dmg).tag(Fire).done();
    })
    .build()
```

`build()` ritorna `Arc<dyn Ability>` che implementa il trait fat dietro le quinte. Aggiungere un nuovo capability al trait = nuovo metodo opzionale del builder, niente breaking. Per skill `.passive()` invece di `.active()`, e `.on_event(...)` per reattività.

### 4.5 Targeting bifronte: input shape vs impact shape

Due cose distinte, mai mescolate:

| | Input shape | Impact shape |
|---|---|---|
| Cosa | Cosa chiedo al giocatore prima del lancio | Cosa colpirò davvero |
| Esempio (Bounce) | 1 selezione enemy | 3 entità (primary + 2 bounce) |
| Esempio (Smart Volley) | 3 selezioni | 3 entità |
| Esempio (Field Strike) | 0 selezioni (AoE) | N + conditional |
| Dove vive | `Ability::input_shape(snap, actor)` — dichiarato dal trait | **Derivato dal dry-run** di `resolve()`, aggregato dall'interprete in `ImpactShape` |
| Cambia a runtime | Sì (buff Heat-mode raddoppia i target richiesti) | Sì (snapshot frozen, ma dipende da actor/target/buffs) |
| Visto da UI | Picker selezione | Tooltip preview |
| Visto da AI | Filter "è applicabile?" | Scoring "quanto vale?" |

Una skill può colpire **più entità di quante l'input ne richieda** (bounce, splash, AoE su target singolo). Questa è feature, non bug — `impact_shape` la espone uniformemente.

`InputShape::Invalid { reason }` quando il set eleggibili è vuoto runtime (es. nessun alleato con HP < 50%): UI mostra grayed con tooltip, AI filtra.

### 4.6 Chain semantics: Intent linkage + Observer

Snapshot frozen per tutto `resolve()` (Semantic A): predict_damage e queries leggono lo stato pre-skill, non lo stato post-Intent-precedente. Per esprimere reattività "se l'attacco è andato a segno, cura":

**Intra-skill** — linkage dichiarativa nell'Intent stesso:

```rust
ctx.enqueue(Intent::DealDamage {
    target,
    base: 120,
    tag: Physical,
    on_hit: Some(OnHit::Heal { who: actor, ratio: 0.5 }),
    on_kill: Some(OnKill::EnqueueFollowUp(AGUMON_SHARP_CLAWS)),
    on_miss: None,
});
```

`OnHit/OnMiss/OnKill` sono enum chiuse piccole (Heal, GainSp, ApplyStatus, EnqueueFollowUp, BlueprintSignal). Il kernel applica la chain quando risolve l'Intent. La skill scrive **una sola riga dichiarativa**, niente conditional imperativo. Dry-run può ispezionare la chain → preview accurato.

**Cross-skill** — Observer kernel-side, esposto via `AbilityHooks`:

```rust
.on_event(AbilityEvent::UnitDied { filter: ally_of_owner }, |ev, ctx| {
    ctx.enqueue_skill(ev.unit_owner, HOLY_AEGIS, TargetSelection::single(ev.unit));
})
```

L'autore dichiara su quali eventi reagisce; l'interprete cabla l'Observer Bevy interno. La skill non vede mai `Trigger<E>` raw.

### 4.7 `Intent` canon

```rust
// src/combat/api/intent.rs
pub enum Intent {
    // Damage & resources
    DealDamage   { target: UnitId, base: i32, tag: DamageTag, on_hit: Option<OnHit>,
                   on_miss: Option<OnMiss>, on_kill: Option<OnKill>, curve_hop: Option<u8> },
    ToughnessHit { target: UnitId, amount: i32 },
    Heal         { target: UnitId, amount_pct_max_hp: u32 },
    Revive       { target: UnitId, hp_pct: u32 },
    GainSp       { side: Team, amount: i32 },
    UltGain      { unit: UnitId, amount: i32 },
    GrantEnergy  { unit: UnitId, amount: i32 },
    ConsumeSp    { unit: UnitId, amount: i32 },

    // Status & cleanse
    ApplyStatus  { target: UnitId, kind: StatusEffectKind, duration: u32 },
    Cleanse      { target: UnitId, count: Option<u8> },

    // Turn order
    AdvanceTurn  { unit: UnitId, pct: u32 },
    DelayTurn    { unit: UnitId, pct: u32 },
    SelfAdvance  { pct: u32 },
    Stun         { target: UnitId },

    // Cards economy
    GrantFreeSkill { unit: UnitId, count: u8 },

    // Reactive (intra-skill follow-up)
    FollowUp     { trigger: FollowUpTrigger, source: UnitId, skill: AbilityId },

    // Escape hatch — typed nel blueprint, opaco al kernel
    BlueprintSignal { owner: &'static str, signal: &'static str, payload: BlueprintPayload },

    // Control flow — non muta stato di gioco
    Reject       { reason: LegalityReasonCode, detail: Option<String> },
}
```

`UnitId` (POD) e non `Instance<Unit>` — skill testabili senza Bevy. `Instance<Unit>` resta convenience al boundary kernel↔ECS.

Drain order = **FIFO** (Vec). Determinismo banale. Le skill canon non hanno cross-order coupling perché ogni skill ha al massimo 1 damage + 1 status.

### 4.8 Cost: separato, condizionale via buff sull'actor

Cost vive in `Ability::cost(snap, actor)` — funzione pura+cheap, snapshot read-only, no enqueue. UI/AI la chiamano ad ogni hover/refresh; pre-skill picker mostra cost dinamico ("30 SP" → "0 SP (enhanced)").

```rust
fn cost(&self, snap: &Snapshot, actor: UnitId) -> AbilityCost {
    if snap.has_status(actor, Status::EnhancedNext) {
        AbilityCost::free()
    } else {
        AbilityCost::sp(self.tuning.sp_cost)
    }
}
```

Buff `EnhancedNext` si autoconsuma: meta-flag su `Status` (`consume_on_skill_cast: true`), una sola sorgente di verità per "quando si spegne", l'autore della skill non se ne preoccupa.

### 4.9 Cross-blueprint state: typed accessor

Pattern uniforme per leggere lo state di un mini-plugin shared (es. `TwinCoreState`):

```rust
// AgumonPlugin::build registra la dipendenza
app.register_blueprint_state::<TwinCoreState>();

// in skill Agumon
fn resolve(&self, ctx: &mut SkillCtx, target: TargetSelection) {
    if let Some(twin) = ctx.blueprint_state::<TwinCoreState>() {
        if twin.cross_resonance >= 2 {
            ctx.deal(target.primary()).damage(self.tuning.base_dmg + bonus).done();
            return;
        }
    }
    ctx.deal(target.primary()).damage(self.tuning.base_dmg).done();
}
```

Implementazione: `SkillCtx` ha `HashMap<TypeId, &dyn Any>` popolato dal kernel pre-resolve copiando le `Resource` rilevanti. Type-safe lato chiamante (typo = compile error), kernel zero conoscenza, party-absent → `None` (skill gestisce gracefully).

Mutazione di shared state: via `Intent::BlueprintSignal { owner: "twin_core", signal: "Heat", payload }`. Il plugin `twin_core` ha un Observer che applica la mutation typed dentro casa sua. Kernel mai conosce il payload concreto.

### 4.10 `combat::api` facade + extension trait

Slice S-A0 estrae il modulo `combat::api` che ospita **tutti i tipi domain-pure** visti dai blueprint:

```
src/combat/api/
├── mod.rs              (re-exports)
├── ability.rs          (trait Ability, AbilityKind, AbilityCategory)
├── builder.rs          (AbilityBuilder)
├── ctx.rs              (SkillCtx, HookCtx, Snapshot façade)
├── intent.rs           (Intent + OnHit/OnMiss/OnKill enums)
├── shape.rs            (InputShape, ImpactShape, TargetSelection)
├── cost.rs             (AbilityCost, AbilityModifiers)
├── legality.rs         (Legality, LegalityReasonCode)
├── display.rs          (AbilityDisplay, AbilityTagSet)
└── hooks.rs            (AbilityHooks, AbilityEvent filters)
```

Invariante grep-verifiable: `rg "use bevy" src/combat/blueprints/ src/combat/api/` → **zero righe**. Niente `Trigger<E>`, `Query<…>`, `Commands`, `EventWriter<…>` fuori dall'interprete (§4.11).

`CombatAppExt` come single registration surface per il futuro autore:

```rust
pub trait CombatAppExt {
    fn register_blueprint<B: Blueprint>(&mut self, bp: B) -> &mut Self;
    fn register_blueprint_state<T: BlueprintState>(&mut self) -> &mut Self;
    fn register_ability<A: Into<Arc<dyn Ability>>>(&mut self, ability: A) -> &mut Self;
}

// Esempio uso
impl Plugin for AgumonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(AgumonBlueprint)
           .register_blueprint_state::<TwinCoreState>()
           .register_ability(abilities::sharp_claws())
           .register_ability(abilities::baby_flame())
           .register_ability(abilities::baby_burner())
           .register_ability(abilities::twin_core_fire_passive());
    }
}
```

### 4.11 Interprete — l'unico punto Bevy-aware

L'interprete è il bridge tra trait domain-pure e macchina Bevy/kernel. Tre responsabilità:

1. **Registrazione**: insert nel registry, wire Observer Bevy per gli `hooks()` dichiarati, verifica state types al boot.
2. **Dispatch runtime**: costruisce `SkillCtx`/`HookCtx`, invoca metodi del trait nel mode richiesto, traduce drain in apply.
3. **Astrazione primitive**: `Trigger<E>`, `Query<…>`, `Commands`, `EventWriter<…>` vivono **solo qui**.

Strutturato in 4 sub-moduli con un solo entry point pubblico per non gonfiare un singolo file:

```
src/combat/interpreter/
├── mod.rs                  → pub fn install(app: &mut App)
├── ability_dispatcher.rs   → ctx building + Mode::Execute/DryRun
├── hook_router.rs          → Observer Bevy wiring per AbilityHooks
├── modifier_aggregator.rs  → composition dei modifier passivi
└── intent_applier.rs       → drain Intent → mutation World
```

L'esterno vede solo `combat::interpreter::install(app)`. L'astrazione resta intatta a livello API.

### 4.12 Modifier pipeline (proposta — Approccio B)

Per le passive che applicano modifier (Light Aegis +10% vs status, Twin Core +5% vs Fire, Battery Loop +20% atk Tentomon): pipeline esplicita configurabile invece di kernel hardcoded.

```rust
pub struct Modifier {
    pub stage: ModifierStage,        // PreClamp | Additive | Multiplicative | PostClamp
    pub value: f32,
    pub condition: Option<ModifierCondition>,
    pub source: AbilityId,            // per debug / introspection
}

pub enum ModifierStage { PreClamp, Additive, Multiplicative, PostClamp }
```

Apply order: tutti i PreClamp → tutti gli Additive → tutti i Multiplicative → tutti i PostClamp. Stessa categoria: ordine deterministico per `source` (AbilityId stable). Introspection: kernel può loggare quale Modifier ha contribuito (debug observability).

Vantaggio: nessun "magic number ordering" sepolto, blueprint autore vede esattamente come si compongono i suoi modifier con quelli altrui.

### 4.13 Strategia di evoluzione del trait — capability-based

Aggiungere un metodo al trait dopo M021 sarebbe breaking (5+ blueprint da toccare). Pattern proposto: **capability slots** opzionali nel builder, niente modifica al trait core.

```rust
pub struct AbilityCapabilities {
    pub redirect_targeting: Option<Box<dyn TargetRedirector>>,
    pub override_damage_calc: Option<Box<dyn DamageCalcOverride>>,
    pub custom_animation_hint: Option<AnimationHint>,
    // … nuove capability aggiunte qui col tempo
}

// Builder usage
AbilityBuilder::active("...")
    .with_capability(redirect_targeting(custom_redirector))
    .build()
```

Aggiungere una nuova capability = nuovo campo `Option<_>`, blueprint che non la usano non vedono cambi. Default = `None`, comportamento standard. Zero breaking.

### 4.14 Asset injection — eager construction (M021)

Skill riceve risorse RON al construction time, possiede `Arc<SkillTuning>` per la sua vita:

```rust
// in AgumonPlugin::build
let tuning = app.world().resource::<Assets<SkillTuning>>().get(BABY_FLAME_HANDLE).unwrap();
let skill = AbilityBuilder::active(...).tuning(tuning.clone()).build();
app.register_ability(skill);
```

Skill è pure, niente lookup runtime, test triviali (`AbilityBuilder::active(...).tuning(test_tuning()).build()`). Hot-reload del RON differito (M021 non richiede): se servirà, conversione meccanica a lookup lazy via `ctx`.

Per asset condivisi (affinity table, scaling curves): accessor mirati sul ctx (`ctx.affinities()`, `ctx.level_curves()`). Skill non vede `Res<...>`, vede solo il getter. Proc-macro `#[derive(Ability)]` con `#[tuning]` field attribute: differita post-M021 — investimento sensato a 50+ skill, non a 24.

---

## 5. Fascia B — Slicing (Blueprint trait + kernel decoupling)

### S01 — `CombatPlugin` extract (refactor zero-logic)

**Goal:** estrarre `register_combat_kernel_runtime` in un Bevy `Plugin` neutro headless/windowed.
**File:** `src/main.rs`, `src/headless.rs`, `src/windowed.rs`, nuovo `src/combat/plugin.rs`.
**Accept:** `cargo check`, `cargo check --features windowed`, `cargo test` tutti verdi. Zero cambi di logica.
**Risk:** basso. **Stima:** 1 h.

### S02 — `trait Blueprint` + opaque `BlueprintTransition` + registry runtime

**Goal:** introdurre l'API target (D005, D006, D007). Nessuna migrazione ancora.
**File nuovi:** `src/combat/blueprints/api.rs` (trait + registry startup-frozen).
**Accept:** registry vuoto si registra come Bevy `Resource`. Test register+lookup. Suite verde.
**Risk:** basso. **Stima:** 1.5 h.

### S03 — Agumon migrato al trait + self-registration

**Goal:** prima migrazione completa via `Box<dyn Any>` con helper `into_transition`/`from_transition` (D006).
**Accept:** `AgumonBlueprint` impl `Blueprint`. Agumon non emette più `CombatKernelTransition::TwinCore(_)` direttamente — emette `Blueprint(_)` con payload opaco. Tests twin core cross-resonance verdi.
**Risk:** medio. **Stima:** 3 h.

### S04 — Gabumon migrato (Twin Core paired)

**Goal:** chiude la coppia Twin Core. Twin Core diventa shared mechanic mini-plugin (owner = "twin_core") (D005).
**Accept:** `TwinCoreSignal`/`TwinCoreTransition` non esistono più in `kernel.rs` — vivono in `blueprints/twin_core/`.
**Risk:** medio-alto. **Stima:** 3 h.

### S05 — Dorumon + Tentomon migrati

**Goal:** Predator Loop e Battery Loop migrati. Single-owner mechanic.
**Accept:** kernel ha perso `PredatorLoopTransition`, `BatteryLoopTransition`, e tutti gli enum collegati. Suite verde.
**Risk:** medio. **Stima:** 3 h.

### S06 — Patamon + Renamon migrati. Rimozione shim. `CombatKernelTransition` digimon-free.

**Accept:** `grep "(TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame)" src/combat/kernel.rs` → 0 match. `CombatKernelTransition` ha 7 variant. `pub use … as twin_core` shim cancellati.
**Risk:** medio. **Stima:** 3 h.

### S07 — `RosterEntry` blueprint-keyed payload

**Before:** `RosterEntry(name: "agumon", twin_core: Some(...), holy_support: None, ...)`.
**After:** `RosterEntry(name: "agumon", blueprint_data: { "twin_core": (...) })`.
**Accept:** aggiungere un Digimon non richiede toccare `units_ron.rs` né `RosterEntry` schema.
**Risk:** medio. **Stima:** 2 h.

### S08 — `ValidationSnapshot` nominato dal registry

**After:** `ValidationSnapshot { blueprint_states: HashMap<String, BlueprintSnapshot> }` popolata dal registry.
**Risk:** medio (touch su ~10+ test file). **Stima:** 2.5 h.

**Totale Fascia B:** ~19 h.

---

## 6. Fascia A — Slicing

Ricalibrato dopo adozione del trait `Ability` unico + builder + capability-based evolution. 24 skill canon + 6 passive canon = 30 ability totali da migrare.

### S-A0 — `combat::api` facade (foundation)

**Goal:** modulo `src/combat/api/` con tutti i tipi domain-pure (vedi §4.10). Zero `use bevy` eccetto tipi neutri.
**File nuovi:** struttura completa di `combat/api/`.
**Accept:** `rg "use bevy" src/combat/api/` → 0. Compila standalone. Trait `Ability` object-safe (test `static_assertions::assert_obj_safe!`).
**Risk:** basso. **Stima:** 2.5 h.

### S-A1 — Interprete + registry

**Goal:** `combat::interpreter::install(app)` con 4 sub-moduli (§4.11). `AbilityRegistry` come Resource. `CombatAppExt` con `register_ability`/`register_blueprint`/`register_blueprint_state`. Observer wiring per `hooks()`.
**Accept:** registrazione + lookup test verdi. Observer per `AbilityEvent::UnitDied` correttamente cablato.
**Risk:** medio-alto (è il punto magico). **Stima:** 4 h.

### S-A2 — `Intent` canon + chain linkage + apply pipeline

**Goal:** definizione canonica con `on_hit`/`on_miss`/`on_kill` + `BlueprintSignal` + `Reject`. Split di `apply_effects` in `intent_applier` che routa per variant.
**Accept:** drain order FIFO, chain linkage applicata sequenzialmente, dry-run mode (no apply) testato.
**Risk:** medio-alto. **Stima:** 3.5 h.

### S-A3 — `SkillCtx` + `Snapshot` façade + builders

**Goal:** `SkillCtx` con frozen snapshot, drain Intent, `blueprint_state::<T>()`, helper builders (`ctx.deal(t).damage(n).tag(...).on_hit(...).done()`). `Mode::DryRun` vs `Mode::Execute`.
**Accept:** test puro Rust (no App) della catena builder. `ImpactShape` derivato dal dry-run drain.
**Risk:** medio. **Stima:** 3 h.

### S-A4 — `AbilityBuilder` + tuning typed

**Goal:** builder fluente per Active/Passive con tuning deserialize typed (`struct BabyFlameNumbers` derive Deserialize). Loader RON minimale che produce `Assets<SkillTuning>` per ID. Eager injection al construction.
**Accept:** una skill di prova costruita via builder ritorna `Arc<dyn Ability>`. Tuning typo nel RON = errore al boot, non runtime.
**Risk:** medio. **Stima:** 3 h.

### S-A5 — Migration 24 skill canon

**Goal:** una skill alla volta in `blueprints/<x>/abilities/<name>.rs`. Cleanup `Effect`-based collapse in `resolve_action`.
**Accept:** suite integration verde. `enum Effect` rimosso da `skills_ron.rs` a fine slice.
**Risk:** medio (volume di lavoro). **Stima:** 5 h.

### S-A6 — Migration 6 passive canon

**Goal:** Twin Core Fire/Water (Agumon, Gabumon), Holy Aegis (Patamon), Predator Loop (Dorumon), Battery Loop (Tentomon), Precision Mind Game (Renamon). Ogni passive in `blueprints/<x>/abilities/<name>.rs` con `kind: Passive` + `hooks()`.
**Accept:** observer test verdi per ogni passive reattiva. Modifier pipeline (§4.12) cabla correttamente i passive stat boost.
**Risk:** medio-alto. **Stima:** 4 h.

### S-A7 — UI/AI consumers riscritti

**Goal:** `query_skill_preview` riscritto via `Ability::resolve(ctx, Mode::DryRun)` → `ImpactShape`. UI `combat_panel.rs` smette di duplicare `predict_damage` locale. AI scoring esposto come helper esterno su `ImpactShape`.
**Accept:** `action_query.rs` riconciliato. `predict_damage` locale rimosso da `combat_panel.rs`.
**Risk:** basso. **Stima:** 2.5 h.

**Totale Fascia A:** ~27.5 h.

---

## 7. Ordine slice interleaved

```
B1  (CombatPlugin extract)               — abilita registry injection
S-A0 (combat::api facade)                — fondamenta dei tipi
B2  (trait Blueprint + registry)         — D005/D006/D007
S-A1 (interprete + registry)             — entry point pubblico
S-A2 (Intent canon + chain + applier)
S-A3 (SkillCtx + builders)
S-A4 (AbilityBuilder + tuning typed)
S-A5 (migrate 24 skill canon)            — rimuove enum Effect
B3  (Agumon @ Blueprint trait, usa SkillCtx per twin-core hook)
B4  (Gabumon paired)
S-A6 (migrate 6 passive canon)           — usa modifier pipeline + hooks
B5  (Dorumon + Tentomon migrati)
B6  (Patamon + Renamon, rimozione shim, kernel digimon-free)
S-A7 (UI/AI consumers)
B7  (RosterEntry blueprint-keyed)
B8  (ValidationSnapshot from registry)
```

**Totale M021:** ~19 h (B) + ~27.5 h (A) ≈ **46.5 h** ≈ 6 giornate piene.

---

## 8. Criterio falsificabile di successo

La `M024-CONTEXT` (primo nuovo roster post-M021) deve includere una sezione "files toccati" che **non contiene** alcun path sotto `src/combat/` eccetto `blueprints/<new_digimon>/`. Se l'autore di un nuovo Digimon deve toccare `kernel.rs`, `intent.rs`, `ctx.rs`, o la facade `combat::api`, M021 ha fallito.

Invariante grep-verifiable continuativo:

```
rg "use bevy" src/combat/blueprints/      → 0 righe
rg "use bevy" src/combat/api/             → 0 righe (eccetto tipi neutri tipo Resource trait)
rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame" src/combat/kernel.rs → 0 righe
```

---

## 9. Out of scope esplicito

- DR pipeline (`BuffKind::DR` clamp 0.5) → M019 (chiuso).
- Reactive bus extension (StatusApplied, UltimateUsed) → M020 (chiuso).
- AdvanceTurn/DelayTurn split + cap → M018 (chiuso).
- TargetShape resolver (Blast/AoE/Bounce) → M018 (chiuso).
- Asset pipeline loader (`clip.ron`, `animation_fsm.ron`) → M022.
- AnimGraph runtime + sprite render → M023.
- `Entity → Instance<Unit>` cleanup in `av.rs`/`turn_system/mod.rs` payload → QQ post-M021.
- `default_headless_script` esternalizzazione → QQ post-M021.
- Hot-reload del RON skill tuning → post-M021 (eager injection è sufficiente).
- Proc-macro `#[derive(Ability)]` → post-M021 (sensato a 50+ skill).
- Split `kernel.rs` 1393 LOC per topic — **naturalmente risolto da S04+S05+S06** che svuotano il kernel di ~600 LOC digimon-specifici.

---

## 10. Domande ancora aperte

Tutto il resto è chiuso (DECISIONS D002–D007 + design corrente §4). Restano da confermare:

| ID | Domanda |
|---|---|
| **Q1** | Modifier pipeline esplicita con 4 stage (§4.12) — regge, o vuoi un'alternativa (es. transformer puri component-by-component)? |
| **Q2** | Capability-based evolution (§4.13) — confermi, o preferisci default-impl sul trait (più semplice ma con rischio "default che nasconde bug")? |
| **Q3** | Asset condivisi (affinity table, level curves, ecc.): accessor mirati sul ctx (`ctx.affinities()`, `ctx.level_curves()`) o blob unico `ctx.kernel_data()` con sotto-getter? |
| **Q4** | Auto-consume di status come `EnhancedNext` (cost-by-buff §4.8): meta-flag su `Status` (`consume_on_skill_cast: true`) o `Intent::ConsumeStatus` esplicito enqueuto dalla skill? Mia preferenza: meta-flag. |
| **Q5** | Migration timing di `enum Effect`: rimosso definitivamente a fine S-A5 (slice 24 skill) o tenuto come deprecated wrapper fino a S-A6 (passive)? |

Chiuse queste 5, il design è pronto per la `M021-ROADMAP.md`.
