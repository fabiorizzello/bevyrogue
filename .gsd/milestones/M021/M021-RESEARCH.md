# M021 — Research & Audit: Kernel ⇄ Digimon Identities Decoupling

**Scope.** Fondazione del combat: kernel agnostico, "1 Digimon = 1 plugin", API skill potente e auto-contenuta. Due fasce:

- **Fascia B** — `trait Blueprint` + `BlueprintRegistry` + migrazione 6 blueprint Digimon + kernel digimon-free.
- **Fascia A** — `trait Ability` (Skill ∪ Passive) + `SkillCtx` + `Intent` + interprete kernel/Bevy + skill auto-contenute.

Questa research è il **single source of truth** del design pre-roadmap. Subsume i recap interni e gli audit additivi.

---

## 0. Lenti d'analisi

Tre tipi di consumer interrogano la superficie del combat, e ciascuno con tre fedeltà:

| Consumer | Cosa chiede | Mode |
|---|---|---|
| **Skill author** | "voglio dire *cosa fa* la mia skill" | write (dichiara Intent + hooks) |
| **Kernel / interprete** | "drena, applica, propaga eventi" | execute (mutazione deterministica) |
| **UI / AI / observability** | "è legale? cosa farebbe? cos'è successo?" | read (legality / dry-run / journal) |

L'astrazione deve servire tutti e tre **dalla stessa definizione di skill/blueprint**, senza che l'autore debba scrivere tre volte la stessa logica.

Fedeltà ortogonali al mode:

| Fedeltà | Chi consuma | Cosa risponde |
|---|---|---|
| **Legality** | UI grayed, AI filter | "posso lanciarla adesso?" — risorse, fase, cooldown, blueprint-state precondition |
| **Preview** (input + impact) | UI picker + tooltip, AI scoring | "che selezione chiedo + cosa succede se la lancio su X?" |
| **Execute** | Kernel commit | "lanciala, applica gli effetti" |

Stessa skill, stessa `resolve()`, mode diverso del `SkillCtx`. Preview = drain Intent in `Mode::DryRun`, kernel non applica ma aggrega in `ImpactShape`.

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

### 1.4 Shim compat — **RIMOSSI in M020/S02-T01** (commit `b321391`)

I `pub use blueprints::agumon::identity as twin_core` (e i 2 simili `predator_loop`, `holy_support`) in `src/combat/mod.rs` sono già stati rimossi e tutti i 13 call-site rerouted ai path canonici `blueprints::<name>::<Type>`. Anti-pattern già spento — vedi anche §7.6.

### 1.5 `UnitDef` + `ValidationSnapshot` conoscono i nomi delle mechanic

- `src/data/units_ron.rs:74 UnitDef` ha 2 field roster Digimon-specifici hardcoded: `twin_core: TwinCoreRosterMetadata` (riga 97) + `holy_support: HolySupportRosterMetadata` (riga 100). Aggiungere un nuovo Digimon con la sua roster-metadata richiede di estendere `UnitDef` con un altro field tipato — anti-pattern parziale. Target: `blueprint_data: HashMap<String, BlueprintPayload>` keyed dal registry.
- `src/combat/observability.rs:31 ValidationSnapshot` ha 5 sub-snapshot field inline: `twin_core`, `holy_support`, `predator_loop`, `battery_loop`, `precision_mind_game` (righe 40-44). Aggiungere un nuovo Digimon richiede aggiornare lo struct.

### 1.6 Skill DSL data-driven (da sostituire)

`src/data/skills_ron.rs:208 enum Effect` ha **14 varianti** (Damage, ToughnessHit, GainSP, UltGain, Stun, Revive, GrantFreeSkill, ApplyStatus, AdvanceTurn, DelayTurn, GrantEnergy, SelfAdvance, Heal, Cleanse). L'apply pipeline (`resolution.rs:626 apply_effects`, ~125 LOC) collassa `Effect` in `ResolvedAction` flatten con campi paralleli. Spike `spike-skill-dsl-coverage` ha verificato copertura 24/24 skill canon — la DSL **regge**, ma è data-driven con logica nascosta nel collapse (Heal/Cleanse mutex, bounce hop selection, GrantFreeSkill target derivation). Va sostituita con `trait Ability` Rust per portare la logica esplicita nel blueprint.

### 1.7 Pipeline scaffolding parzialmente in place

`src/combat/turn_system/pipeline.rs` (~67 KB) implementa già il 4-beat (`TacticalCyclePhase::Declared → PreApp → Impact → Applied`) con guard PerHop, fan-out `AllAllies`/`AllEnemies`, integration `Effect::Heal`/`Cleanse`. **Non duplicarlo** in S-A2: l'`Intent` applier riusa questo pipeline come substrato Bevy-aware, non lo sostituisce.

---

## 2. Macro-aree funzionali — API surface da coprire

Mappatura completa delle funzionalità che il combat **deve supportare** e che l'astrazione deve esporre. Letta dal codice corrente + 24 skill canon + 6 passive canon. Non propone design qui: enumera la *surface*.

### A. Ability lifecycle (skill attiva + passiva)

Una sola unità di astrazione (`trait Ability` con `AbilityKind { Active, Passive }`, vedi §5.3): 24 skill canon + 6 passive = 30 ability. Ogni ability espone:

- **Identità statica**: `id`, `owner` (blueprint), `kind`, `category` (Basic/Skill/Ult/Passive), `display`, `tags` (DamageTag, AbilityTag).
- **Legality** snapshot-pure: "lanciabile ora?" — fasi, KO, stun, blueprint-state preconditions.
- **Cost** snapshot-pure: SP / Ult / Energy / FreeSkill, **condizionale via buff sull'actor** (es. EnhancedNext → 0 SP).
- **Input shape** snapshot-pure: cosa chiedere all'utente (single enemy / N selections / AoE / self / none / Invalid{reason}).
- **Resolve**: produce `Intent` via `ctx.enqueue(...)`. Mai mutazione diretta.
- **Hooks**: dichiarazione passiva su che eventi reagire (UnitDied, OnSkillCast, OnDamageDealt, …) + filter (`ally_of_owner`, `is_enemy`).
- **Modifiers**: passive che alterano calcoli (PreClamp / Additive / Mult / PostClamp) — vedi §I.
- **Capability slots opzionali** (per pattern non-trasversali): redirect_targeting, override_damage_calc, telegraph multi-step, reaction window.

> **Gap aperto (vedi Q11)**: telegraph multi-tick, reaction windows, aborts mid-resolve, costs runtime-divergent. Vanno classificati: in-trait vs capability slot vs scope rimandato.

### B. Resource economies

- **SP pool** (`sp.rs`): shared cross-party, `RoundSpTracker` con cap non_basic +2/round.
- **Ult charge** (`ultimate.rs`): per-unit, 5 trigger type (`UltAccumulationTrigger`: OnBasicAttack/OnHitTaken/OnAllyFollowUp/OnKill/OnOffensivePartyEvent).
- **Energy** (`energy.rs`): per-unit, `RoundEnergyTracker` con cap (secondary 10, external 30) e `EnergyGainSource`.
- **Free-skill** (Frontiera Patamon/Renamon): conteggio carte gratuite (`GrantFreeSkill`).
- **Health / HP** — manipolata da damage/heal, ma il *consumo come risorsa* (sacrificio?) non esiste in canon attuale.

**Superficie API che l'astrazione deve dare alla skill**:
- query: `sp_available()`, `ult_ready()`, `energy_room(source)`, `free_skill_count()`, predicati `has_status(actor, Status::EnhancedNext)`
- enqueue: `Intent::ConsumeSp / GainSp / UltGain / GrantEnergy / GrantFreeSkill`
- Cost-as-function (non come stringa) per UI dinamica.

### C. Damage pipeline (formula = single source of truth)

Oggi in `damage.rs` + `resolution.rs`. La formula moltiplicativa è:

```
final = base × tag_mod × triangle_mod × break_mod × status_amp × attacker_mult × DR_mod
```

con `DamageBreakdown` esposto per UI floating + observability. Il kernel **deve** restare unico esecutore: la skill dichiara base + tag + on_hit chain, **non duplica la formula**.

**Superficie API target**:
- enqueue: `Intent::DealDamage { target, base, tag, on_hit, on_miss, on_kill, curve_hop }`
- query (per AI scoring / UI preview): `ctx.predict_damage(actor, target, base, tag) -> DamageBreakdown` su snapshot frozen.
- knobs ortogonali registrabili: triangle (attribute matchup), tag affinity (Fire/Ice/Lightning/Holy/…), break (toughness state), status amp (Heated+Fire, Chilled+Ice), DR clamp.
- **Cleanup**: triangle e tag-affinity tables vanno fuori dal kernel come asset (`ctx.affinities()`, `ctx.level_curves()`), non hardcoded.

### D. Status & buffs (taxonomy canon §H.1)

`StatusEffectKind` (`status_effect.rs`): Heated, Chilled, Paralyzed, Slowed, Blessed (+ riservati: Burn, Shock). `StatusBag` con `apply()` (refresh max_dur), `tick_all()`, `cleanse_n()`. `BuffKind` enum partiziona Buff vs Debuff.

**Superficie API**:
- enqueue: `Intent::ApplyStatus { target, kind, duration }`, `Intent::Cleanse { target, count? }`
- query: `ctx.has_status(unit, kind)`, `ctx.status_duration(unit, kind)`, `ctx.cleanse_candidates(unit)`
- **Auto-consume** (Q4): meta-flag `consume_on_skill_cast: bool` su `StatusEffectDef` invece di `Intent::ConsumeStatus` esplicito per skill — applier centralizzato, autore non duplica.
- **Cleanse + Heal mutex** (canon v0): unica skill ne fa l'una *o* l'altra, mai entrambe. L'astrazione deve permettere/vietare esplicitamente — non lasciarlo invariante non-detto come adesso.
- **Stack-aware status** numerici (Heated × N DoT scaling) → fuori scope M021 (D009 deferred). Tenere il punto di estensione aperto.
- **Outcome esplicito**: `StatusBag::apply` con `max(old,new)` silente. L'astrazione deve esporre `StatusApplyOutcome` (Applied/Refreshed/Resisted/AlreadyMax) come parte del `DealDamageResult`.

### E. Targeting bifronte: input shape vs impact shape

Distinzione **non negoziabile** (§5.5). Oggi `action_query.rs:282 ActionAffordance` ha solo input shape — manca l'altra metà.

| | Input shape (Mode::Picker) | Impact shape (Mode::DryRun) |
|---|---|---|
| Cosa | Quante selezioni chiedo all'utente | Quante entità colpisce davvero |
| Esempio | Bounce: 1 enemy | Bounce: primary + 2 hops |
| Quando cambia | Buff Heat-mode → 2 selezioni invece di 1 | Snapshot frozen, ma deriva dal dry-run |
| Consumer | UI picker, AI filter | Tooltip preview, AI scoring |
| Derivato da | `Ability::input_shape(snap, actor)` esplicito | Drain Intent in Mode::DryRun aggregato |

**Capability `TargetShape` esistenti** che il kernel risolve oggi (M018): Single, Blast (primary + adjacents), Row, AllEnemies, AllAllies, SelfOnly, Bounce {hops, selector, repeat}. **Restano kernel-level**: sono primitive geometriche, non identità.

**Superficie API**:
- input dichiarativo via `InputShape::*`
- builder per shape geometriche standard: `ctx.deal(primary).blast()`, `ctx.bounce(hops, selector)`, `ctx.aoe(side)`
- selectors registrabili: `BounceSelector::{LowestHpPctAlive, NextSlotAlive, AdjLowest, …}` + custom via trait/closure registrato.

### F. Turn order & phase

`CombatPhase` (`state.rs:11`): WaitingForTurn / WaitingAction / Resolving / Victory / Defeat. `ActionValue` (`av.rs`) + `TurnOrder` resource + `TurnAdvanced` event. `TacticalCyclePhase` (`kernel.rs:9`): Declared → PreApp → Impact → Applied — è il "4-beat" pipeline (`turn_system/pipeline.rs`).

**Superficie API**:
- enqueue: `Intent::AdvanceTurn / DelayTurn / SelfAdvance / Stun`
- query: `ctx.turn_preview(n)`, `ctx.next_actor()`, `ctx.av(unit)`, `ctx.is_stunned(unit)`
- 4-beat è punto di osservazione per passive/hook (es. "on_combat_beat") — già esposto come `CombatEventKind::OnCombatBeat`.

### G. Reactive bus + follow-up

`CombatEventKind` (`events.rs:26`): **~30 varianti** (lifecycle + granular). `FollowUpTrigger` (`kit.rs`): OnEnemyBreak / OnAllyLowHp / OnEnemyKill. `FollowUpTrace` (`follow_up.rs:42`) con `FollowUpOriginKind` (FollowUp vs FormIdentity) + `FollowUpSkipReason` (5 motivi).

**Superficie API**:
- **Intra-skill chain** (sincrono, dichiarativo): `on_hit`/`on_miss`/`on_kill` come campi dell'Intent, varianti chiuse piccole (Heal, GainSp, ApplyStatus, EnqueueFollowUp, BlueprintSignal). Snapshot frozen — predict pre-skill, non interleaved.
- **Cross-skill reactive**: hooks dichiarativi (`AbilityHooks::on_event(filter, handler)`); interprete cabla l'Observer Bevy. Skill non vede `Trigger<E>`.
- **Follow-up queue**: FIFO drain post-action, guard team/KO/stun, trace journal per debug.

> **Schema canonico (§5.6 + Q-hookOrder, proposta forte confermata)**: ordine quando N passive ascoltano lo stesso event = FIFO per `register_ability` order, tie-break per `AbilityId`. Gli enqueue dagli hook entrano nella **stessa** `VecDeque` del cast originale (single drain coda), applicati dopo il completamento dell'apply attuale e prima del prossimo event drain. Hook drain e action drain condividono il loop kernel — non sono due code separate.

### H. Cross-blueprint identity state

5 famiglie oggi nel kernel (anti-pattern da smontare): Twin Core (Agumon+Gabumon **shared**), Battery Loop (Tentomon), Holy Support (Patamon), Predator Loop (Dorumon), Precision Mind Game (Renamon). Ognuna ha enum `*Signal` + `*Transition` dentro `kernel.rs` (~600 LOC totali di Digimon-bleed).

**Superficie API target**:
- ogni mechanic = mini-plugin `blueprints/<owner>/` con Component/Resource proprietari.
- accesso typed dalla skill: `ctx.blueprint_state::<TwinCoreState>() -> Option<&T>` (party-absent → None gestito gracefully).
- mutazione tramite `Intent::BlueprintSignal { owner: "twin_core", signal, payload }`; il plugin owner ha un Observer/system che applica typed (downcast checked, D006).
- **shared mechanic** (`twin_core`): registrato come *terzo plugin*, Agumon e Gabumon dichiarano dipendenza al boot.
- **Type-pair compile-time-checkable**: il plugin owner espone `pub fn heat_signal(amount: i32) -> Intent` constructor — la skill chiamante non costruisce mai il payload `Box<dyn Any>` a mano.

> **Gap aperto (Q7, Q10)**: layer single vs double per `Intent::BlueprintSignal` (journal/replay coverage); validation grafo `Agumon depends twin_core` in `App::finish()`.

### I. Modifier pipeline (passive stat boost)

Pattern target (§5.12 + Q1): 4-stage **PreClamp → Additive → Multiplicative → PostClamp**. Stessa categoria → tie-break per `(priority: i8, source: AbilityId)` esplicito, non solo string cmp.

**Superficie API**:
- `Ability::modifiers(snap, unit) -> AbilityModifiers` opt-in default-empty.
- enum `ModifierStage` chiuso.
- `Modifier { stage, value, condition, source, priority }`.
- introspezione: kernel logga quale Modifier ha contribuito al breakdown — observability lato passive.

### J. Combat events / observability

`CombatEvent` come bus single-source-of-truth (CLAUDE.md). `ValidationSnapshot` (`observability.rs:31`) cattura phase/winner/sp/turn_preview/action_log_tail/units + 5 sub-snapshot Digimon-specifici (anti-pattern, da spostare a `HashMap<String, BlueprintSnapshot>`).

**Superficie API**:
- evento bus rimane `Message`/`MessageReader` (Bevy 0.18 rename) **non** Observer-only, per FIFO deterministica.
- per ogni Intent applicato, il kernel emette evento granular (OnDamageDealt, OnStatusApplied, OnHealed, OnCleansed, OnBreak, …).
- `BlueprintSnapshot` trait: `Debug + Serialize + 'static` (Q6), un valore per blueprint registrato.
- journal JSONL (`jsonl_logger.rs`) registra `BlueprintSignal` se layer-double, non se layer-single → decisione Q7.

### K. UI/AI read API

`query_action_affordance` (`action_query.rs`) ritorna `ActionAffordance` con kind/action/target/targets/resource/resource_details/implementation/toughness. Consumato da `ui/combat_panel.rs` e `bin/combat_cli.rs`. `CombatQuerySnapshot` con `UnitQuerySnapshot` completi.

**Superficie API target**:
- **`Mode::DryRunNoTarget`** (hover skill, target non scelto) → ritorna `InputShape + ImpactShape::Pending + cost`. UI grayed con tooltip.
- **`Mode::DryRun`** (target scelto) → `ImpactShape` completo (chain inclusa) + previewed damage breakdown + status outcomes.
- **`Mode::Execute`** → applica, emette eventi.
- riconciliazione con `ActionAffordance` esistente: o lo sostituisce o lo subsume — decisione Q-reconcile aperta (non blocca la roadmap, può chiudersi durante S-A7 col worker). Nota: il "duplicato di `predict_damage`" citato in design precedenti **non esiste** (verificato `rg "predict_damage" src/` → 0). Il vero duplicato è tra `apply_effects` e `query_action_affordance` (entrambi leggono `SkillDef.effects`). Single `resolve(Mode::DryRun)` → muore il duplicato.

### L. Bootstrap / data / spawn

`bootstrap.rs` (SelectionRequest → EncounterComposition → spawn) + `EncounterPreset` (3 preset hardcoded). `units_ron.rs` (UnitDef) + `skills_ron.rs` (SkillDef ↔ da ridurre a SkillTuning numbers-only) + `party_ron.rs`. `party_validation.rs` valida composizione.

**Superficie API target**:
- `units.ron` invariato (già numeri).
- `skills.ron` collassa a `Assets<SkillTuning>` per ID (HashMap), 1 struct typed per skill (es. `BabyFlameNumbers`) deserializzata fail-fast al boot.
- `RosterEntry` blueprint-keyed: `blueprint_data: HashMap<String, BlueprintPayload>` invece di field hardcoded.
- preset encounter resta data — non astrazione.

### M. Plugin registration / extension surface

Oggi `BLUEPRINTS: &[BlueprintRegistration]` hardcoded in `blueprints/mod.rs:110-135` con 6 entry — la forma di registrazione è **uniforme** (`BlueprintRegistration { owner, dispatch }`), ma il **module backing è asimmetrico**: 3 moduli idiomatici (`agumon/`, `dorumon/`, `patamon/`) + 3 file flat (`gabumon.rs`, `renamon.rs`, `tentomon.rs`). Da `kernel.rs:1067-1093` (`register_combat_kernel_runtime`) si registrano 3 Plugin + 2 system + 4 Resource. Anti-pattern: `BLUEPRINTS` è una lista statica — un nuovo Digimon richiede l'edit del file kernel.

**Superficie API target**:
- `trait CombatAppExt`: `register_blueprint`, `register_blueprint_state::<T>()`, `register_ability(Arc<dyn Ability>)`.
- `CombatPlugin` neutro (no winit/wgpu/egui) — verificato grep `rg "use bevy" src/combat/{api,blueprints}/` → 0.
- `BlueprintRegistry` + `AbilityRegistry` startup-frozen (D007 + Q10).
- validazione dipendenze cross-plugin (es. Agumon depends `TwinCoreState`) in `App::finish()` — pending check vs panic loud.

---

## 3. Trasversali — vincoli che attraversano tutte le aree

1. **Determinismo headless-first** (CLAUDE.md). Drain FIFO via `VecDeque<Intent>` resource + exclusive kernel system. No HashMap iteration in apply order. No wall-clock, no RNG senza seed.
2. **Frozen snapshot semantics** (Q8). Una sola decisione da prendere: freeze totale (snapshot ricostruito una volta a inizio resolve, drain interpreta tutto contro snap, applica al World solo al termine) **vs** interleaved (apply tra ogni Intent). Voto: freeze totale — più semplice da spiegare, dry-run identico a execute. Va enforced via tipo: `SkillCtx::snapshot` è `Arc<FrozenSnapshot>` immutabile, le query del ctx non leggono mai `&World`.
3. **Dry-run = Execute - apply** (§5.1 fedeltà variabile). Stesso codice `resolve()`, mode diverso. Garantisce che UI preview ≡ runtime reality.
4. **Reject scope** (Q9). Decidere: all-or-nothing (preview ≡ execute, atomicità), partial-apply, o solo informativo. Voto: all-or-nothing — preview UI deve essere veritiero o crolla la fiducia.
5. **Kernel agnostico** (P001). Invariante grep: `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame" src/combat/kernel.rs` → 0. Aggiungere un Digimon = 0 file toccati fuori da `blueprints/<x>/`.
6. **Bevy 0.18 specifics**: `CombatEvent` come `Message` (FIFO). Observer Bevy **solo** per il dispatch cross-skill reattivo (`AbilityHooks::on_event(...)` cablato dall'interprete in `On<E>`, ex `Trigger<E>`); l'intra-skill chain (`on_hit`/`on_miss`/`on_kill`) **non** è Observer ma drain linkage dentro l'`Intent`, risolta sequenzialmente dal kernel. La skill autore non vede mai `Trigger<E>` raw.
7. **SkillCtx come SystemParam o exclusive-system-built**: tendere a costruirlo da exclusive system (kernel system) che possiede `&mut World` + cached `QueryState`s. La skill vede solo lo stub.

---

## 4. Target architecture

### 4.1 Cosa il kernel **deve** sapere (core invariant)

- Turn order, AV gauge, TacticalCycle (windup/strike/recovery), Beat
- Strain (toughness/break), Flow (SP), Fatigue (Ult charge) — risorse universali
- Tag bus generico (CombatTag) → meccanismo, non lista chiusa
- Status taxonomy canon §H.1 (5 status) — vocabolario condiviso, non identitario
- Custom signal dispatch infrastructure (envelope `Blueprint { owner, signal, payload }`)
- **Intent applier**: damage formula, mitigation, break, status tick — single source of truth per i side-effect

### 4.2 Cosa il kernel **non deve** sapere

- Twin Core: heated stack rules, cross-resonance — è Agumon+Gabumon
- Battery Loop: charge kinds, transfer rules — è Tentomon
- Predator Loop: berserk cap, target tracking — è Dorumon
- Holy Support: grace gauge, martyr light — è Patamon
- Precision Mind Game: phase ladder — è Renamon
- Logica di **qualsiasi skill** — sta nel blueprint owner via `impl Ability`

### 4.3 Shape kernel post-M021

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

> **Relazione con `Intent::BlueprintSignal`** (Q7): la skill produce `Intent::BlueprintSignal`. L'`intent_applier` può:
> - **Layer single**: chiamare direttamente `bp.handle_signal(payload)` via registry. Niente `CombatKernelTransition::Blueprint` esiste — meno indirezione.
> - **Layer double**: produrre un `CombatKernelTransition::Blueprint` che entra nel transition stream (utile per replay/journal JSONL).
>
> Implicazione: `OnKernelTransition` oggi registra le transition. Single layer = perdiamo replay sul signal blueprint. Va deciso.

### 4.4 Cross-mechanic state (D005 shared-mechanic mini-plugin)

Twin Core cross-resonance (Agumon ↔ Gabumon): si crea un mini-plugin `blueprints/twin_core/` con `owner = "twin_core"`, ospita il `Component`/`Resource` condiviso. Agumon e Gabumon dipendono da `TwinCoreState`, non viceversa. Pattern riusabile per future shared mechanic.

Accesso runtime alla shared state da una skill: tipizzato via `ctx.blueprint_state::<TwinCoreState>()` (vedi §5.9).

---

## 5. Fascia A — Design corrente

### 5.1 Principio: Skill API potenziata, fedeltà variabile

La skill non è una DSL statica né un blob di codice opaco. È **un oggetto interrogabile** alle 3 fedeltà di §0. Stessa skill, stessa `resolve()`, mode diverso del `SkillCtx`. Preview = drain Intent in `Mode::DryRun`, kernel non applica ma aggrega in `ImpactShape`.

### 5.2 RON = solo numeri (tuning sheet)

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

### 5.3 Trait `Ability` unico (Skill ∪ Passive)

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

> **Pattern non coperti dal trait core** (Q11): telegraph multi-tick, reaction window, persistence cross-turn, aborts mid-resolution, cost runtime-divergent (Ult charge consumed solo se hit). Mia raccomandazione: capability slot per telegraph/multi-tick, in-trait per il resto (sono pattern di 1-2 skill su 24).

### 5.4 Builder pattern per costruire le skill

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

Per test puri Rust (no App): `AbilityBuilder::tuning_raw(BabyFlameNumbers { ... })` bypass-asset, costruisce skill senza loader RON.

### 5.5 Targeting bifronte: input shape vs impact shape

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

### 5.6 Chain semantics: Intent linkage + Observer

Snapshot frozen per tutto `resolve()` (Q8 — freeze totale): predict_damage e queries leggono lo stato pre-skill, non lo stato post-Intent-precedente. Per esprimere reattività "se l'attacco è andato a segno, cura":

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

**Schema Observer wiring (sotto-specificato — Q-hookOrder)**:
1. `CombatEvent` Bevy → `AbilityEvent` neutro (con filter già applicato) tradotto in `hook_router.rs`.
2. Filter `ally_of_owner` come enum chiuso `EventFilter::{AllyOfOwner, EnemyOfOwner, Self_, Any}` + custom predicate registrato. Non function ad hoc.
3. **Order** quando N passive ascoltano lo stesso event: FIFO per `register_ability` order, ulteriormente tie-break per `AbilityId` cmp.
4. Una passive emette via `ctx.enqueue_skill(...)` → questi Intent entrano nella **stessa drain coda** del cast originale (single VecDeque). Vengono applicati dopo il completamento dell'apply attuale, prima del successivo event drain.

### 5.7 `Intent` canon

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

**Semantica Reject (Q9 — all-or-nothing)**: una `resolve()` produce **o solo Intent applicabili o solo Reject**. Atomicità. Se `OnHit::Heal` si rejecta a target full HP mid-chain, l'intero apply della skill viene rolled back (dry-run mode lo restituisce come `ImpactShape::Rejected { reason }`). Garantisce preview = execute.

### 5.8 Cost: separato, condizionale via buff sull'actor

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

### 5.9 Cross-blueprint state: typed accessor

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

Implementazione: `SkillCtx` ha `HashMap<TypeId, &dyn Any>` popolato dal kernel pre-resolve copiando le `Resource` rilevanti (frozen). Type-safe lato chiamante (typo = compile error), kernel zero conoscenza, party-absent → `None` (skill gestisce gracefully).

Mutazione di shared state: via constructor pubblicato dal plugin owner:

```rust
// in blueprints/twin_core/mod.rs
pub fn heat_signal(amount: i32) -> Intent {
    Intent::BlueprintSignal {
        owner: "twin_core",
        signal: "Heat",
        payload: BlueprintPayload::new(HeatPayload { amount }),
    }
}

// in skill Agumon
ctx.enqueue(twin_core::heat_signal(1));
```

Il plugin `twin_core` ha un Observer che applica la mutation typed dentro casa sua. Kernel mai conosce il payload concreto. Tipo compile-time-checked al sito di costruzione.

### 5.10 `combat::api` facade + extension trait

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

Invariante grep-verifiable: `rg "use bevy" src/combat/blueprints/ src/combat/api/` → **zero righe**. Niente `Trigger<E>`, `Query<…>`, `Commands`, `EventWriter<…>` fuori dall'interprete (§5.11).

`CombatAppExt` come single registration surface:

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

**Registry lifecycle (Q10)**: tutti i `register_blueprint_state::<T>()` aggregano in un `pending: Vec<TypeId>`, validati in `App::finish()` via `World::resource_exists::<T>()`. Plugin order non matters: se `AgumonPlugin` registra `TwinCoreState` dependency prima di `TwinCorePlugin`, l'errore esplode al `finish()` con messaggio actionable ("Agumon depends on TwinCoreState but no plugin registered it").

### 5.11 Interprete — l'unico punto Bevy-aware

L'interprete è il bridge tra trait domain-pure e macchina Bevy/kernel. Tre responsabilità:

1. **Registrazione**: insert nel registry, wire Observer Bevy per gli `hooks()` dichiarati, verifica state types al boot.
2. **Dispatch runtime**: costruisce `SkillCtx`/`HookCtx`, invoca metodi del trait nel mode richiesto, traduce drain in apply.
3. **Astrazione primitive**: `Trigger<E>`, `Query<…>`, `Commands`, `EventWriter<…>` vivono **solo qui**.

Strutturato in 4 sub-moduli con un solo entry point pubblico:

```
src/combat/interpreter/
├── mod.rs                  → pub fn install(app: &mut App)
├── ability_dispatcher.rs   → ctx building + Mode::Execute/DryRun
├── hook_router.rs          → Observer Bevy wiring per AbilityHooks
├── modifier_aggregator.rs  → composition dei modifier passivi
└── intent_applier.rs       → drain Intent → mutation World
```

L'esterno vede solo `combat::interpreter::install(app)`. L'astrazione resta intatta a livello API.

### 5.12 Modifier pipeline (Approccio B + tie-break esplicito)

Per le passive che applicano modifier (Light Aegis +10% vs status, Twin Core +5% vs Fire, Battery Loop +20% atk Tentomon): pipeline esplicita configurabile invece di kernel hardcoded.

```rust
pub struct Modifier {
    pub stage: ModifierStage,        // PreClamp | Additive | Multiplicative | PostClamp
    pub value: f32,
    pub condition: Option<ModifierCondition>,
    pub source: AbilityId,            // per debug / introspection
    pub priority: i8,                 // tie-break esplicito, default 0
}

pub enum ModifierStage { PreClamp, Additive, Multiplicative, PostClamp }
```

Apply order: tutti i PreClamp → tutti gli Additive → tutti i Multiplicative → tutti i PostClamp. Stessa categoria: ordine `(priority desc, source asc)`. `priority` esplicito chiude il buco "rinomino skill = regressione numerica silent". Default `0` per il 95% delle skill.

Introspection: kernel può loggare quale Modifier ha contribuito (debug observability).

Vantaggio: nessun "magic number ordering" sepolto, blueprint autore vede esattamente come si compongono i suoi modifier con quelli altrui.

### 5.13 Strategia di evoluzione del trait — default-impl + capability slot

Aggiungere un metodo al trait dopo M021 sarebbe breaking (5+ blueprint da toccare). Decisione (Q2): **default-impl per il 90% dei pattern** (zero overhead, noop dichiarato), **capability slot per overrides non-trasversali**.

```rust
// Default-impl: nuova capability noop, non breaking
pub trait Ability {
    fn modifiers(&self, _snap: &Snapshot, _unit: UnitId) -> AbilityModifiers {
        AbilityModifiers::none()
    }
}

// Capability slot: pattern raro, overhead di Option<Box<dyn>> giustificato
pub struct AbilityCapabilities {
    pub redirect_targeting: Option<Box<dyn TargetRedirector>>,
    pub override_damage_calc: Option<Box<dyn DamageCalcOverride>>,
    pub telegraph: Option<Box<dyn TelegraphSchedule>>,
}
```

Aggiungere una nuova capability = nuovo `Option<_>` field, blueprint che non la usano non vedono cambi.

### 5.14 Asset injection — eager construction (M021)

Skill riceve risorse RON al construction time, possiede `Arc<SkillTuning>` per la sua vita:

```rust
// in AgumonPlugin::build
let tuning = app.world().resource::<Assets<SkillTuning>>().get(BABY_FLAME_HANDLE).unwrap();
let skill = AbilityBuilder::active(...).tuning(tuning.clone()).build();
app.register_ability(skill);
```

Skill è pure, niente lookup runtime, test triviali (`AbilityBuilder::active(...).tuning_raw(BabyFlameNumbers { ... }).build()`). Hot-reload del RON differito (M021 non richiede): se servirà, conversione meccanica a lookup lazy via `ctx`.

Per asset condivisi (affinity table, scaling curves): accessor mirati sul ctx (`ctx.affinities()`, `ctx.level_curves()`). Skill non vede `Res<...>`, vede solo il getter. Q3 risolta: accessor mirati, non blob unico (evita coupling implicito).

Proc-macro `#[derive(Ability)]` con `#[tuning]` field attribute: differita post-M021 — investimento sensato a 50+ skill, non a 24.

---

## 6. Capability matrix — cosa una skill canon può chiedere

Aggregata da `resolution.rs` + 24 skill canon (verificate dallo spike) + 6 passive. Ogni casella = una operazione che `SkillCtx`/`Intent` deve supportare:

| Capacità | Forma API | Skill esempio |
|---|---|---|
| Single-target damage | `Intent::DealDamage` | Agumon Sharp Claws |
| AoE damage | `Intent::DealDamage` × N (via `ctx.aoe()`) | Tentomon Electrical Discharge |
| Blast (primary + adj) | `ctx.deal(p).blast()` | Patamon Sparking Air Shot |
| Bounce chain | `ctx.bounce(hops, selector)` | Renamon Koyosetsu |
| Toughness hit | `Intent::ToughnessHit` (auto da DealDamage break tag) | Tutti |
| Heal % max | `Intent::Heal { amount_pct_max_hp }` | Patamon Holy Aegis |
| Cleanse N | `Intent::Cleanse { count }` | Patamon (variant) |
| Apply status | `Intent::ApplyStatus { kind, duration }` | Gabumon Blue Cyclone |
| Stun | `Intent::Stun` | Tentomon ult |
| Revive | `Intent::Revive { hp_pct }` | (riservato) |
| Gain SP (party) | `Intent::GainSp { side }` | Renamon basic |
| Ult gain (unit) | `Intent::UltGain` | Auto on basic |
| Grant Energy | `Intent::GrantEnergy` | Battery Loop |
| Consume SP | `Intent::ConsumeSp` | (cost path) |
| Grant FreeSkill | `Intent::GrantFreeSkill` | Patamon support |
| Advance/Delay turn | `Intent::AdvanceTurn / DelayTurn / SelfAdvance` | Renamon Tohakken |
| Follow-up enqueue | `Intent::FollowUp` o `OnKill::EnqueueFollowUp` | Dorumon Predator Loop |
| Blueprint signal | constructor del plugin owner (vedi §5.9) | Twin Core Heat, Battery charge |
| Reject (legality fail mid-resolve) | `Intent::Reject { reason }` | Bounce target morto |
| Listen on event | `Ability::hooks().on_event(filter, handler)` | Holy Aegis (UnitDied) |
| Persistent stat modifier | `Ability::modifiers(snap, unit)` | Twin Core +5% Fire |
| Auto-consume status | `StatusEffectDef::consume_on_skill_cast` flag | EnhancedNext |

**Pattern reali non ancora in tabella** (Q11), da decidere se in-trait o capability slot:

- Telegraph multi-tick (skill che si carica N beat poi parte) → **capability slot** raccomandato
- Reaction window (skill che apre finestra di counter altrui) → in-trait via `hooks()`
- Persistence cross-turn (Twin Core Heated stack durevole) → in-trait via `modifiers()`
- Aborts mid-resolution (target morto a metà bounce) → in-trait via `Intent::Reject` + all-or-nothing
- Cost runtime-divergent (Ult charge consumed solo se hit, non on cast) → **manca pattern**: serve `on_resolution_complete` hook o `Ability::cost_adjustment(outcome)`. Decidere in Q11.

---

## 7. Anti-pattern attuali da smontare senza paura

Il "non avere paura di smontare" si applica concretamente a:

1. **`CombatKernelTransition` con 5 (di 11) variant Digimon-specifiche** (`kernel.rs:890` — `TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame` + 6 kernel-generic: `TacticalCycle`, `Strain`, `Flow`, `Fatigue`, `Tag`, `Beat`) → conservare le 6 kernel-generic, collassare le 5 Digimon in `Blueprint(BlueprintTransition { owner, payload: Box<dyn Any> })` opaco.
2. **`ValidationSnapshot` con 5 field hardcoded** (`observability.rs:31`) → `HashMap<String, BlueprintSnapshot>` popolata dal registry.
3. **`UnitDef` con 2 field roster Digimon-specifici** (`units_ron.rs:97-100`: `twin_core: TwinCoreRosterMetadata`, `holy_support: HolySupportRosterMetadata`) → `blueprint_data: HashMap<String, BlueprintPayload>`.
4. **`enum Effect` data-driven con 14 varianti** (`skills_ron.rs:208`) + `apply_effects` ~125 LOC collapse → muore a fine S-A5. Nessun deprecated wrapper (Q5: doppio code path → bug di sync).
5. **3 blueprint flat** (`gabumon.rs`, `renamon.rs`, `tentomon.rs`) **vs 3 plugin idiomatici** (`agumon/`, `dorumon/`, `patamon/` — `dorumon/` ha anche `hooks.rs`) → tutti plugin uniformi.
6. **Shim `pub use blueprints::agumon::identity as twin_core`** → ✓ **RIMOSSO in M020/S02-T01** (commit `b321391`). Non più anti-pattern attivo, mantenuto in lista come "done".
7. **Duplicazione fantasma `predict_damage`** che design precedenti citano ma `rg` non trova: il vero duplicato è tra `apply_effects` e `query_action_affordance` (entrambi leggono `SkillDef.effects`). Single `resolve(Mode::DryRun)` → muore il duplicato.
8. **Custom signal payload validation per-blueprint** (no schema enforcement) → typed signal constructor pubblicato dal plugin owner (`pub fn heat_signal(amount: i32) -> Intent`).
9. **`SkillDef.effects` come logica camuffata da dato** → diventa `SkillTuning::numbers` numbers-only; logica si sposta in `impl Ability for X`.
10. **`StatusBag::apply` con `max(old, new)` duration** silente: la re-apply che fail-rolla **non** ritorna info al chiamante. L'astrazione deve esporre `StatusApplyOutcome` (Applied/Refreshed/Resisted/AlreadyMax) come parte del `DealDamageResult`.

---

## 8. moonshine_kind — stato

Già adottato dove serve: `Query<Instance<Unit>>` in `headless.rs:210`, `ui/combat_panel.rs:234-235`, `bin/combat_cli.rs:135`.

Residui legittimi: `Query<(Entity, &FloatingDamage)>` in `floating.rs:22` — usato solo per despawn, OK.

Residui da convertire (QQ post-M021): `av.rs:56` `pub unit_entity: Entity` → `Instance<Unit>`; `turn_system/mod.rs:380` payload entity → idem.

**Non bloccante M021.**

---

## 9. Fascia B — Slicing (Blueprint trait + kernel decoupling)

### S01 — `CombatPlugin` extract (refactor zero-logic)

**Goal:** estrarre `register_combat_kernel_runtime` in un Bevy `Plugin` neutro headless/windowed.
**File:** `src/main.rs`, `src/headless.rs`, `src/windowed.rs`, nuovo `src/combat/plugin.rs`.
**Accept:** `cargo check`, `cargo check --features windowed`, `cargo test` tutti verdi. Zero cambi di logica.
**Note:** già neutro nel `main.rs:63` (chiamata prima del branch). Verificare se vale 1 h dedicata o se piega dentro S02.
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

**After:** `ValidationSnapshot { blueprint_states: HashMap<String, BlueprintSnapshot> }` popolata dal registry. `BlueprintSnapshot: Debug + Serialize + 'static` (Q6).
**Risk:** medio (touch su ~10+ test file). **Stima:** 2.5 h.

**Totale Fascia B:** ~19 h.

---

## 10. Fascia A — Slicing

Ricalibrato dopo adozione del trait `Ability` unico + builder + default-impl evolution. 24 skill canon + 6 passive canon = 30 ability totali da migrare.

### S-A0 — `combat::api` facade (foundation)

**Goal:** modulo `src/combat/api/` con tutti i tipi domain-pure (vedi §5.10). Zero `use bevy` eccetto tipi neutri.
**File nuovi:** struttura completa di `combat/api/`.
**Accept:** `rg "use bevy" src/combat/api/` → 0. Compila standalone. Trait `Ability` object-safe (test `static_assertions::assert_obj_safe!`).
**Risk:** basso. **Stima:** 2.5 h.

### S-A1 — Interprete + registry

**Goal:** `combat::interpreter::install(app)` con 4 sub-moduli (§5.11). `AbilityRegistry` come Resource. `CombatAppExt` con `register_ability`/`register_blueprint`/`register_blueprint_state`. Observer wiring per `hooks()`. Pending validation in `App::finish()` (Q10).
**Accept:** registrazione + lookup test verdi. Observer per `AbilityEvent::UnitDied` correttamente cablato. Validation panic loud con messaggio actionable se state dipendenza mancante.
**Risk:** medio-alto (è il punto magico). **Stima:** 4 h.

### S-A2 — `Intent` canon + chain linkage + apply pipeline

**Goal:** definizione canonica con `on_hit`/`on_miss`/`on_kill` + `BlueprintSignal` + `Reject`. Split di `apply_effects` in `intent_applier` che routa per variant. All-or-nothing reject (Q9).
**Accept:** drain order FIFO, chain linkage applicata sequenzialmente, dry-run mode (no apply) testato. `ImpactShape::Rejected` esposto.
**Risk:** medio-alto. **Stima:** 3.5 h.

### S-A3 — `SkillCtx` + `Snapshot` façade + builders

**Goal:** `SkillCtx` con frozen snapshot totale (Q8) `Arc<FrozenSnapshot>`, drain Intent, `blueprint_state::<T>()`, helper builders (`ctx.deal(t).damage(n).tag(...).on_hit(...).done()`). `Mode::DryRun` vs `Mode::Execute` vs `Mode::DryRunNoTarget`.
**Accept:** test puro Rust (no App) della catena builder. `ImpactShape` derivato dal dry-run drain.
**Risk:** medio. **Stima:** 3 h.

### S-A4 — `AbilityBuilder` + tuning typed

**Goal:** builder fluente per Active/Passive con tuning deserialize typed (`struct BabyFlameNumbers` derive Deserialize). Loader RON minimale che produce `Assets<SkillTuning>` per ID. Eager injection al construction. `tuning_raw` per test.
**Accept:** una skill di prova costruita via builder ritorna `Arc<dyn Ability>`. Tuning typo nel RON = errore al boot, non runtime.
**Risk:** medio. **Stima:** 3 h.

### S-A5 — Migration 24 skill canon

**Goal:** una skill alla volta in `blueprints/<x>/abilities/<name>.rs`. Cleanup `Effect`-based collapse in `resolve_action`. **Drop completo di `enum Effect` a fine slice** (Q5).
**Accept:** suite integration verde. `enum Effect` rimosso da `skills_ron.rs` a fine slice.
**Risk:** medio (volume di lavoro). **Stima:** 5 h.

### S-A6 — Migration 6 passive canon

**Goal:** Twin Core Fire/Water (Agumon, Gabumon), Holy Aegis (Patamon), Predator Loop (Dorumon), Battery Loop (Tentomon), Precision Mind Game (Renamon). Ogni passive in `blueprints/<x>/abilities/<name>.rs` con `kind: Passive` + `hooks()`.
**Accept:** observer test verdi per ogni passive reattiva. Modifier pipeline (§5.12) cabla correttamente i passive stat boost.
**Risk:** medio-alto. **Stima:** 4 h.

### S-A7 — UI/AI consumers riscritti

**Goal:** `query_skill_preview` riscritto via `Ability::resolve(ctx, Mode::DryRun)` → `ImpactShape`. UI `combat_panel.rs` consuma il nuovo preview. `ActionAffordance` riconciliato (sostituito o subsume — decidere durante slice in base al churn). AI scoring esposto come helper esterno su `ImpactShape`.
**Accept:** `action_query.rs` riconciliato. Duplicazione `apply_effects` ↔ `query_action_affordance` morta. `Mode::DryRunNoTarget` cablato per hover-senza-target.
**Risk:** basso. **Stima:** 2.5 h.

**Totale Fascia A:** ~27.5 h.

---

## 11. Ordine slice interleaved

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

## 12. Criterio falsificabile di successo

La `M024-CONTEXT` (primo nuovo roster post-M021) deve includere una sezione "files toccati" che **non contiene** alcun path sotto `src/combat/` eccetto `blueprints/<new_digimon>/`. Se l'autore di un nuovo Digimon deve toccare `kernel.rs`, `intent.rs`, `ctx.rs`, o la facade `combat::api`, M021 ha fallito.

Invariante grep-verifiable continuativo:

```
rg "use bevy" src/combat/blueprints/      → 0 righe
rg "use bevy" src/combat/api/             → 0 righe (eccetto tipi neutri tipo Resource trait)
rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame" src/combat/kernel.rs → 0 righe
```

---

## 13. Out of scope esplicito

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
- AI scoring heuristics avanzate (oggi enemy_ai.rs è banale Ult→Skill[0]→Basic) → milestone "AI smarter" successivo.
- Stack-aware status numerici (Heated × N DoT scaling) → D009 deferred.
- Split `kernel.rs` 1393 LOC per topic — **naturalmente risolto da S04+S05+S06** che svuotano il kernel di ~600 LOC digimon-specifici.

---

## 14. Domande aperte

Tutto il resto è chiuso (DECISIONS D002–D007 + design corrente §5). Domande che **devono** essere chiuse prima della M021-ROADMAP.

### Ranking di priorità (verifica post-research)

| Cluster | Q | Status | Cosa plasma |
|---|---|---|---|
| ⭐ **Svolta design** (tradeoff reale, decisione bloccante) | **Q7** | da decidere | forma `CombatKernelTransition`, `intent_applier`, journal/forensics coverage, API del plugin owner |
| ⭐ **Svolta design** | **Q11** | parzialmente da decidere | numero esatto di metodi nel trait `Ability` (incl. `cost_adjustment`); congela l'API surface di `combat::api/ability.rs` |
| ⭐ **Svolta design** | **Q8** | proposta forte, conferma esplicita | tipo di `SkillCtx::snapshot`; semantica `on_hit::*`; garanzia `preview ≡ execute` |
| Conferma blocco | **Q1, Q2, Q3, Q4, Q5, Q6, Q9, Q10** | proposte forti allineate | un singolo "ok" o annotazioni puntuali sufficiente |
| Già risolto / uniformare | **Q-hookOrder** | risolto in §5.6 + §G | nessuna decisione, solo documentazione |
| Non bloccante | **Q-reconcile** | può chiudersi in S-A7 | non blocca la roadmap |
| Azione (non domanda) | **Q12** | da eseguire | persistere D008/D009/D010 in `.gsd/DECISIONS.md` |

**Round breve consigliato:** Q7 + Q11 + Q8 in un unico passaggio. Sono i tre che plasmano `Intent`, `Ability`, `SkillCtx` — ogni miss qui = breaking trait dopo M021.



### Q1 — Modifier pipeline (proposta forte: confermata + priority)

4-stage `PreClamp → Additive → Multiplicative → PostClamp` + tie-break esplicito `(priority: i8, source: AbilityId)`. Default priority 0. Vedi §5.12. Alternativa transformer = troppo flessibile, troppo difficile da debug.

**Proposta:** confermata. Va decisa.

### Q2 — Trait evolution strategy

Default-impl per il 90% pattern (`fn hooks(&self) -> AbilityHooks { AbilityHooks::none() }`), capability slot solo per overrides non-trasversali. Vedi §5.13.

**Proposta:** confermata.

### Q3 — Asset condivisi: accessor mirati vs blob

`ctx.affinities()`, `ctx.level_curves()` getter mirati vs `ctx.kernel_data()` blob con sotto-getter. Accessor mirati allinea con §5.10 facade (zero `Res<…>` in blueprint).

**Proposta:** accessor mirati.

### Q4 — Auto-consume status

Meta-flag `consume_on_skill_cast: bool` su `StatusEffectDef` vs `Intent::ConsumeStatus` esplicito enqueuto dalla skill. Vedi §5.8.

**Proposta:** meta-flag (applier centralizzato, skill autore non duplica).

### Q5 — Timing drop di `enum Effect`

Fine S-A5 (24 skill migrate) — passive non hanno bisogno di `Effect`. Alternativa: deprecated wrapper fino a S-A6 (crea doppio code path → bug di sync).

**Proposta:** fine S-A5, no wrapper.

### Q6 — `BlueprintSnapshot` trait shape

`Box<dyn Any>` opaco vs `trait BlueprintSnapshot: Debug + Serialize + 'static`. Serve `Serialize` per i tests + JSONL.

**Proposta:** trait con Debug + Serialize + 'static.

### Q7 — Layer single vs double per `Intent::BlueprintSignal`

- **Single**: `intent_applier` chiama direttamente `bp.handle_signal(payload)` via registry. Niente `CombatKernelTransition::Blueprint`. Più semplice, **perde replay/journal** sul signal.
- **Double**: applier produce `CombatKernelTransition::Blueprint` che entra nel transition stream. JSONL completo, una indirezione in più.

Implicazione concreta: `OnKernelTransition` (events.rs) oggi registra le transition → forensics. Single layer = forensics cieca sui signal blueprint.

**Da decidere** — propendo per **double** per coerenza con `OnKernelTransition` journal coverage; single solo se la coperture replay non è ritenuta valore.

### Q8 — Semantica snapshot frozen

Freeze totale (snapshot ricostruito una volta a inizio resolve, drain interpreta tutto contro snap, applica al World solo al termine) vs interleaved (apply tra ogni Intent).

Decisione richiesta perché:
- preview ≡ execute solo se freeze totale
- chain `on_hit::Heal` deve vedere il World pre-skill o post-DealDamage? Freeze totale dice pre-skill.

**Proposta:** freeze totale, enforced via tipo (`SkillCtx::snapshot: Arc<FrozenSnapshot>`, no `&World`).

### Q9 — Reject scope (sticky semantics)

- **All-or-nothing**: una resolve produce o solo Intent applicabili o solo Reject. Atomicità. Preview = execute.
- **Partial-apply + Reject**: apply gli Intent precedenti al Reject, marca i seguenti skipped.
- **Reject informativo**: apply tutto, Reject è metadata per UI.

**Proposta:** all-or-nothing (preview-execute fidelity).

### Q10 — Registry validation timing

`register_blueprint_state::<T>()` aggregato in `pending: Vec<TypeId>` e validato in `App::finish()` via `World::resource_exists::<T>()`. Errore loud con messaggio actionable se mancante. Alternativa: lazy create con default, hide bugs.

**Proposta:** App::finish() validation, panic loud.

### Q11 — Pattern non coperti: capability slot vs in-trait

Telegraph multi-tick, reaction window, cost runtime-divergent (Ult charge consumed solo se hit). Decidere quali entrano nel trait core e quali in capability slot.

- Telegraph multi-tick → **capability slot** `Box<dyn TelegraphSchedule>` (1-2 skill su 24, raro)
- Reaction window → **in-trait** via `hooks()` (pattern reattivo standard)
- Cost runtime-divergent → **in-trait** via nuovo `fn cost_adjustment(outcome) -> Option<AbilityCost>` default `None`

**Da decidere** la lista esatta + se cost_adjustment vale il metodo aggiuntivo.

### Q12 — Persistenza D008/D009/D010

`M021-CONTEXT.md` cita D008 (CombatPlugin separation), D009 (stack-aware status differito), D010 (trait Skill + SkillCtx). `.gsd/DECISIONS.md` si ferma a D007.

**Da fare:** persistere le tre decisioni in `.gsd/DECISIONS.md` prima della roadmap, altrimenti la roadmap punta a decisioni non esistenti.

### Q-hookOrder — Observer wiring schema (sotto-domanda di Q2/Q11)

Quando N passive ascoltano lo stesso event:
- Ordine: FIFO per `register_ability` order, tie-break `AbilityId` cmp.
- Coda enqueue da hook: stesso `VecDeque<Intent>` del cast originale.
- Schema esplicito vive in `interpreter/hook_router.rs`.

**Proposta:** confermata, ma serve documentare nella roadmap.

### Q-reconcile — ActionAffordance vs SkillPreview

`SkillPreview` (D004) sostituisce `ActionAffordance` o vi si aggiunge? Quando target non scelto: `Mode::DryRunNoTarget` ritorna `InputShape + ImpactShape::Pending + cost`, `Mode::DryRun` ritorna full preview.

**Da decidere** durante S-A7 in base al churn — chiudere la decisione prima del cut o lasciare al worker dello slice.

---

## 15. Pre-roadmap checklist

Prima di scrivere `M021-ROADMAP.md`:

1. **Chiudere Q1–Q11** (le proposte sono pronte, servono i sì/no espliciti).
2. **Persistere D008/D009/D010** in `.gsd/DECISIONS.md` (Q12).
3. **Validare la lista anti-pattern** (§7) come "smontaggi autorizzati" — il "non avere paura di smontare" del CONTEXT è il via libera, ma vale conferma esplicita.
4. **Decidere granularità slice** (M021-ROADMAP) — 15 slice intersliced è OK, oppure raggruppare per riduzione overhead di context-switch?

Le proposte forti in questa research (Q2 default-impl, Q3 accessor mirati, Q4 meta-flag, Q5 hard drop S-A5, Q6 trait con Serialize, Q8 freeze totale, Q9 all-or-nothing, Q10 finish validation) sono coerenti con tutto il resto del design. Q1, Q7, Q11 hanno tradeoff reali da pesare.

---

## 16. Verifica drift vs codice (2026-05-14)

Pass di verifica eseguito in parallelo (2 Explore agents) per validare i claim del documento contro `src/combat/`:

| Claim verificato | Esito | Note |
|---|---|---|
| `CombatPhase` 5 variants (`state.rs:11`) | ✓ PASS | Confermato |
| `TacticalCyclePhase` 4 variants (`kernel.rs:9`) | ✓ PASS | Declared → PreApp → Impact → Applied |
| `CombatEventKind` ~30 variants (`events.rs:26`) | ✓ PASS | 30 esatte, recenti aggiunte: `OnHealed`, `OnCleansed`, `EnergyGained`, `UltimateUsed` |
| `FollowUpTrigger` 3 variants (`kit.rs:9`) | ✓ PASS | OnEnemyBreak / OnAllyLowHp / OnEnemyKill |
| `RoundEnergyTracker` cap 10/30 (`energy.rs:52`) | ✓ PASS | secondary 10, external 30 |
| `Effect` 14 variants (`skills_ron.rs:208`) | ✓ PASS | conteggio confermato |
| `apply_effects` LOC (`resolution.rs:626`) | ✗ FIX | ~125 LOC (research diceva ~200) — **aggiornato §1.6, §7.4** |
| `CombatKernelTransition` Digimon-specific | ✗ FIX | 5 di 11 variant, non solo 5 totali — **aggiornato §7.1** |
| `RosterEntry` con 5 field hardcoded | ✗ FIX | la struct è `UnitDef` con 2 field (`twin_core`, `holy_support`) — **aggiornato §1.5, §7.3** |
| Shim `pub use ... as twin_core` ancora vivi | ✗ FIX | rimossi in commit `b321391` (M020/S02-T01) — **aggiornato §1.4, §7.6** |
| `BLUEPRINTS` 6 entry asimmetriche | ✗ NUANCE | registrazione uniforme, ma backing module asimmetrico — **aggiornato §M** |
| `ValidationSnapshot` 5 sub-snapshot | ✓ PASS | tutti i field hardcoded ancora presenti |
| `enum Intent` non esiste ancora | ✓ PASS | corretto: è la nuova astrazione di M021 |
| `turn_system/pipeline.rs` esiste | ✓ + aggiunta | ~67 KB, 4-beat già implementato — **aggiunto §1.7** |

**Conclusione**: 8/14 claim PASS al primo passaggio, 6/14 drift corretti chirurgicamente. Sostanza intatta — l'analisi macro-aree (§2), il design Fascia A (§5), il capability matrix (§6), e le Q (§14) rimangono validi. La correzione più importante: l'anti-pattern §7.6 (shim) è già stato spento in M020 e va trattato come "done", non come work-item di M021.

---

## 17. Reality-check finale post-decisioni (2026-05-14)

Pass eseguito dopo persistenza di D008–D015 e scrittura della roadmap. Tre agenti in parallelo: (a) Explore su codice attuale vs decisioni, (b) ricerca online su pattern Bevy 0.18 + skill engine in Rust, (c) recap esaustivo delle macro-aree funzionali combat. Risultati distillati: **3 BLOCKER strutturali**, **4 ATTRITI**, **11 scoperte inattese**, mappatura **35 aree funzionali**. Nessuna decisione richiede revoca; tre richiedono micro-prep prima di S01.

### 17.1 BLOCKER (richiedono prep strutturale prima/dentro S01-S02)

| # | Decisione | Hallazgo | Mitigazione proposta |
|---|---|---|---|
| **B1** | **D009** (AbilityHook cast-scoped) | `cast_id` non esiste nel codebase (zero match). `CombatEvent` non porta discriminator cast. ~50 callsites di emit chiamano senza scope. | **S01-T01** (extract CombatPlugin) deve atomicamente aggiungere `cast_id: CastId(NonZeroU32)` a `CombatEvent` e propagarlo da `resolution.rs`. Tracking origin: in `pipeline::step_app` quando si emette `ResolvedAction`, assegnare `cast_id = next_id()` e propagarlo nel context. Costo: 1 task aggiuntivo a S01, ~50 callsites in compile-error guide. |
| **B2** | **D010** (Ult instant cast) | Oggi `ActionIntent::Ultimate` passa dallo stesso `advance_turn_system` pipeline di Skill/Basic; non c'è separazione semantica. Renderlo istantaneo richiede o (a) duplicare la pipeline per fare 2 cast in 1 tick, o (b) inserire un by-pass nello state machine post-Ult. | **Spostato S-A3 → S-A1**: la separazione "Ult ≠ turn-consuming" diventa **precondition** del CombatPlugin extraction. Senza di essa, S-A4 (hook dispatch) non può funzionare perché Ult-as-action contamina il `cast_id` lifecycle. Aggiungere `TacticalCyclePhase::UltInstant` come variant transition + bypass turn advance. |
| **B3** | **D011** (turn-phase order 5-step) | L'ordine attuale in `advance_turn_system` (turn_system/mod.rs:469–630) è: Heated DoT tick → Stun check → Paralysis + status tick → Snapshot → Skill select. **Manca** uno step "turn-start passive trigger" dopo il tick e prima dello snapshot. KO resolution è implicita ("skip se Ko"), non esplicita. | **S-A4** (hook/listener dispatch) deve introdurre lo step esplicito "Apply turn-start hooks via `BlueprintListener::on(OnTurnStart)`" tra status tick e snapshot. Refactor di `advance_turn_system` in fasi nominate (`PreTurnTick`, `PostTickKoResolve`, `TurnStartHooks`, `PostHooksKoResolve`, `Snapshot`, `SelectAction`) — non solo riordinare codice, ma esporre le fasi come `SystemSet` Bevy. |

### 17.2 ATTRITI (manageable con prep tattica)

| # | Decisione | Hallazgo | Mitigazione |
|---|---|---|---|
| **A1** | **D008** (Intent::BlueprintSignal double-layer) | `CombatKernelTransition` enum ha 5 variant Digimon-specific hardcoded (`TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`) a `kernel.rs:890–902`. Il drop di queste è il **success criterion** di M021. | Aggiungere variant `CombatKernelTransition::Blueprint { owner: BlueprintId, payload: Box<dyn BlueprintSignalPayload> }`, deprecare le 5 variant in S-A5, rimuoverle in S-B4 (kernel digimon-free). |
| **A2** | **D013** (listener ordering) | Bevy `MessageWriter/MessageReader` è FIFO per insertion order, **non** per priority. Per ottenere (initiative DESC, slot ASC, team_id ASC) servirà un sort esplicito dei listener prima dell'invocazione. | Il dispatcher hook (S-A4) tiene `Vec<HookEntry { weight: HookWeight, hook: ... }>` e ordina prima del drain. `HookWeight = (initiative, slot, team_id)` tuple ordering canonico. |
| **A3** | **D015-Q5** (drop enum Effect) | ~60 occorrenze in 24 skill canon, più `apply_effects()` (resolution.rs) come matchbox su variant. Più heavy del previsto. | Migrazione spalmata su S-A5 → S-B3, con gate `rg "enum Effect"` = 0 a fine S-A8 (vedi roadmap). Aggiunto a §7.4 come "anti-pattern principale da abbattere". |
| **A4** | **D015-Q2** (BlueprintState: Component + Reflect) | Nessun blueprint component attuale implementa `#[derive(Reflect)]`. `TwinCoreState`, `BatteryLoopState`, `PrecisionMindGameState` sono Resource senza Reflect. | S-A2 (state extraction) aggiunge `#[derive(Reflect)]` a tutti i blueprint state component. Costo: 5 tipi × 1 riga. Bevy 0.17+ ha **auto-registration via `reflect_auto_register`** quindi il `register_type` esplicito è ridondante per scopo reflection — resta solo come gate validation. |

### 17.3 Scoperte inattese (rilevanti, non blocker)

#### Codice attuale
1. **Asimmetria blueprint plugins** — Agumon/Dorumon/Patamon sono `Plugin` (struct + `impl Plugin`); Gabumon/Renamon/Tentomon sono file flat con solo dispatcher. **S-B2 deve omogeneizzare verso il pattern Plugin uniforme prima di applicare `trait Blueprint`**.
2. **`CustomSignalPayload` RON layer** (skills_ron.rs:260) è il bridge corrente tra enum Effect e signal dispatcher — sarà deprecato da `Intent::BlueprintSignal`. Aggiungere riga di migrazione a S-A8.
3. **JSONL non incatena transition blueprint** — `jsonl_logger.rs` legge `CombatEvent` ma non `CombatKernelTransition`. La D008 (double-layer routing) **richiede** che JSONL aggiunga subscription al transition stream. Aggiunto come T-jsonl-transition a S-A4.

#### Pattern Bevy 0.18
4. **`AppExtStates` è il modello canonico** per "register typed entity atomico". `register_blueprint::<B>(app)` dovrebbe seguire la stessa shape: extension trait su `App`, generic, chainante `register_type::<B::State>() + init_resource + insert nel registry`. Vedi `bevy::state::prelude::AppExtStates`.
5. **`#[require(BlueprintState)]` > runtime registration** — issue Bevy [#16406](https://github.com/bevyengine/bevy/issues/16406), [#16645](https://github.com/bevyengine/bevy/issues/16645) confermano bug noti su required components runtime. Preferire `#[require]` static su `BlueprintMarker`.
6. **Event → Message rename in Bevy 0.17** — la nomenclatura "CombatEvent" della roadmap va verificata: in Bevy 0.18 `Event` = trigger observer, `Message` = buffered queue (vecchio `Event`). Probabilmente il nostro `CombatEvent` è semanticamente un `Message`. Non è un bug, ma onboarding debt se non documentato.
7. **`bevy-trait-query` è 10× più lento** delle query concrete (81 µs vs 8 µs). **NON usare nell'hot path damage calc**. Per blueprint lookup: `HashMap<BlueprintId, ...>` su `Resource` frozen-after-startup.

#### Pattern Rust skill engine (progetti reali)
8. **Veloren preferisce data struct + ECS system** — `Attack` è una struct (non trait), risolta da sistemi dedicati. Funziona su MMO con centinaia di skill. **Domanda implicita**: il nostro `trait Ability` è necessario per ~30 skill, o `enum Ability` + match nel kernel basta? Decisione D015-Q5 (drop enum Effect) implica trait → non ri-aprire, ma flag per post-MVP review.
9. **bracket-lib usa Component marker = Intent** — `WantsToMelee`, `WantsToMove` come Component, drenati da system. Pattern identico al nostro `Intent` ma component-based. Trade-off: ECS-native ma esplode in N component types per N intent variant. **La nostra scelta `enum Intent` resta valida**.
10. **`typetag` + `Send + Sync`** — confermato che la libreria de-facto per `Box<dyn Trait>` serializzabili ha [limitazione documentata](https://github.com/serde-rs/serde/issues/384) con `Send + Sync`. Se M0xx aggiunge "save mid-combat", `Box<dyn BlueprintSnapshot>` richiede `serde_flexitos` o tag manuali. **D008 (JSONL output-only) è safe**, ma annotare nel CODEBASE.md.

#### Architetturali
11. **`enum Intent` rischia mega-funzione** — con ~17 variant + `Box<dyn Any>` payload, l'`intent_applier` diventa un mega-match. Considerare split per family (`DamageIntent`, `StatusIntent`, `ResourceIntent`, `BlueprintIntent`) con sotto-dispatcher. **Da decidere in S-A5**.

### 17.4 Recap macro-aree combat (35 aree mappate)

Audit completo delle aree funzionali che il combat deve supportare. Surface API attuale + target post-M021 per ognuna. Mantiene allineamento sul "cosa deve essere estendibile da blueprint vs cosa resta kernel-core".

**Kernel-core (stabile, non blueprint-extensible)**:
1. Turn order & AV (`turn_order.rs`, `av.rs`, `turn_system/mod.rs`)
2. Speed & Tempo modifiers (`speed.rs`, `resistance.rs`)
3. Damage breakdown (`damage.rs`)
4. Toughness & Break (`toughness.rs`)
5. Stun & CC (`stun.rs`)
6. Skill resolution pipeline (`resolution.rs`, `turn_system/mod.rs`)
7. Target shape & Bounce (`resolution.rs`)
8. Bootstrap & encounter spawn (`bootstrap.rs`)
9. Event bus + Action log (`events.rs`, `log.rs`)
10. JSONL logger (`jsonl_logger.rs`)
11. RNG centralization (`rng.rs`)
12. Unit/types/team core (`unit.rs`, `types.rs`, `team.rs`)
13. Action query & affordance (`action_query.rs`)
14. Round flags (`round_flags.rs`)
15. Tempo resistance (`resistance.rs`)
16. Kit & skill config (`kit.rs`)
17. Enemy AI (`enemy_ai.rs`)
18. Enemy counterplay (`enemy_counterplay.rs`, `counterplay.rs`)
19. Floating numbers UI (`floating.rs`)
20. Observability snapshots (`observability.rs`)

**Blueprint-extensible (post-M021 via trait Ability / trait Blueprint / hook)**:
21. SP pool & resource (`sp.rs`)
22. Ultimate charge & accumulation (`ultimate.rs`)
23. Status effects & ticks (`status_effect.rs`)
24. Damage reduction & buffs (`buffs.rs`)
25. Follow-up reactions (`follow_up.rs`)
26. Form identity (`kit.rs`, `follow_up.rs`)
27. Energy system (`energy.rs`)
28. Custom signal dispatch (`blueprints/mod.rs`, `blueprints/*/signals.rs`)
29. Twin Core - Agumon (`kernel.rs` partial, `blueprints/agumon/signals.rs`)
30. Predator Loop - Dorumon (`blueprints/dorumon/`)
31. Holy Support - Patamon (`blueprints/patamon/`)
32. Battery Loop - Tentomon (`battery_loop.rs`, `blueprints/tentomon.rs`)
33. Precision Mind Game - Renamon (`precision_mind_game.rs`, `blueprints/renamon.rs`)
34. Kernel registry & hooks (`kernel.rs`)
35. Skill custom signal payload (`skills_ron.rs`)

**Coupling Digimon-specifico nel kernel (4 punti da abbattere)**:
- C1. **5 variant `CombatKernelTransition` hardcoded** (`kernel.rs:890–902`) → diventeranno `CombatKernelTransition::Blueprint { owner, payload }` generico
- C2. **`observability.rs` mescola snapshot blueprint-specific** (5 sub-snapshot) → optional injection via hook registry
- C3. **`blueprints/mod.rs` router centralizzato di custom signal** → per-blueprint plugin registration in `CombatKernelRegistry`
- C4. **Bootstrap inserisce Resource per-blueprint** in `register_combat_kernel_runtime()` (kernel.rs:1067–1093) → plugin-driven registration via `trait Blueprint`

### 17.5 Impatto sulla roadmap M021

| Modifica richiesta | Slice | Note |
|---|---|---|
| Aggiungere `cast_id: CastId` a `CombatEvent` + propagazione 50 callsites | **S01** | Task atomico nel CombatPlugin extraction; senza non funziona S-A4 |
| Spostare "Ult instant cast" da S-A3 a **S01** o **S-A1** | **S01/S-A1** | Precondition di tutto il resto: Ult-as-action contamina il `cast_id` lifecycle |
| Refactor `advance_turn_system` in fasi nominate (SystemSet) | **S-A4** | 5 SystemSet espliciti, con turn-start hook injection point |
| Sort esplicito listener per `(initiative, slot, team_id)` | **S-A4** | `HookWeight` tuple, drain ordinato |
| `#[derive(Reflect)]` su 5 blueprint state component | **S-A2** | 5 righe; auto-registration Bevy 0.17 fa il resto |
| Omogeneizzare Gabumon/Renamon/Tentomon a pattern Plugin | **S-B2** | Pre-requisito per applicare `trait Blueprint` uniformemente |
| JSONL subscribe a `CombatKernelTransition` stream | **S-A4** | Senza, D008 (double-layer) non chiude il loop sull'osservabilità |

### 17.6 Validazione finale per kickoff S01

- [x] Tutte le decisioni D008–D015 sono **applicabili**, con prep tattica documentata sopra
- [x] Tre blocker (cast_id, Ult instant, turn-phase order) hanno mitigazione concreta
- [x] Nessuna decisione richiede revoca o ri-discussione
- [x] Roadmap richiede 3 aggiustamenti minori (S01 espanso, S-A4 con SystemSet refactor, S-B2 con omogeneizzazione plugin) — non ridisegno
- [x] 35 macro-aree mappate, 4 punti di coupling Digimon-specifico identificati come obiettivo principale del refactor

**Verdetto**: M021 è **GO** per S01 con i tre aggiustamenti sopra. Nessun blocker richiede ulteriore discussione di design.

---

**Fonti citate**:
- [Bevy 0.17→0.18 migration guide](https://bevy.org/learn/migration-guides/0-17-to-0-18/)
- [Bevy 0.16→0.17 migration guide](https://bevy.org/learn/migration-guides/0-16-to-0-17/) (Event/Message rename)
- [AppExtStates](https://docs.rs/bevy/latest/bevy/state/prelude/trait.AppExtStates.html)
- [Bevy required components](https://docs.rs/bevy/latest/bevy/prelude/trait.Component.html)
- [bevy-trait-query benchmarks](https://docs.rs/bevy-trait-query/latest/bevy_trait_query/)
- [Veloren combat module](https://docs.veloren.net/veloren_common/combat/index.html)
- [Roguelike Tutorial in Rust (intent pattern)](https://bfnightly.bracketproductions.com/chapter_1.html)
- [typetag](https://github.com/dtolnay/typetag), [serde_flexitos](https://crates.io/crates/serde_flexitos)
- Bevy issues [#16406](https://github.com/bevyengine/bevy/issues/16406), [#16645](https://github.com/bevyengine/bevy/issues/16645) (required components bugs)
