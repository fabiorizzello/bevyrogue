# M021 — Roster v0 fit-check: 6 rookie nel nuovo design Ability/SkillCtx/Intent

> **Scope.** Pseudocodice Rust illustrativo di tutte le **18 active skill + 6 passive** del roster v0 (Agumon, Gabumon, Dorumon, Tentomon, Patamon, Renamon), scritto contro l'API target M021 (`trait Ability`, `AbilityBuilder`, `SkillCtx`, `Intent`, `BlueprintRegistry`).
>
> **Obiettivo.** Esercizio di fit: il design copre tutte le funzionalità canon? Cosa polluta il kernel? Identificare gap dell'API surface **prima** di S01.
>
> **Non è codice da compilare.** Le firme di tipi/builder/enum sono quelle proposte in `M021-RESEARCH.md` §5; drift, semplificazioni e omissioni sono volute per leggibilità. I gap rilevati sono raccolti in §8 e propagati a `M021-RESEARCH.md` §18.

---

## 0. Convenzioni condivise

- ID di skill/blueprint sono `&'static str` (Q1 default M021 — typed wrapper post-MVP).
- `StatusEffectKind` canon v0: **`Heated, Chilled, Slowed, Paralyzed, Blessed`** (Blessed è nuovo per Renamon, vedi §G4).
- `DamageTag` canon v0: **`Fire, Ice, Electric, Holy, Dark, Physical`**.
- `BuffKind` (tag-pure, presence-only — separato da `StatusEffectKind`): `DefenseUp { pct, dur }` per Gabumon `fur_cloak` e `blue_cyclone`. Vedi §G5.
- Ogni passive è registrata come `AbilityKind::Passive`; nessuna seconda registry (P001).
- Tuning struct = una per skill, `#[derive(Deserialize, Clone)]`, eager-loaded al boot dal RON tramite `Assets<SkillTuning>` (§5.14).
- `AbilityCost::basic()` = costo standard basic (0 SP, +1 SP gen, +25 Ult charge); `AbilityCost::sp(n)` = skill spender; `AbilityCost::ult_full()` = drena ult bar.
- Nessuna `Plugin` per Digimon importa `bevy_egui` / `winit` / `wgpu` (invariante grep §M).

### Shape dell'API consumata (pin)

```rust
// src/combat/api/ability.rs (sintesi rilevante)
pub trait Ability: Send + Sync + 'static {
    fn id(&self) -> AbilityId;
    fn owner(&self) -> BlueprintId;
    fn kind(&self) -> AbilityKind;          // Active | Passive
    fn category(&self) -> AbilityCategory;  // Basic | Skill | Ult | Passive | FollowUp
    fn display(&self) -> &AbilityDisplay;
    fn tags(&self) -> AbilityTagSet;

    fn legality(&self, snap: &Snapshot, actor: UnitId) -> Legality;
    fn cost(&self, snap: &Snapshot, actor: UnitId) -> AbilityCost;
    fn input_shape(&self, snap: &Snapshot, actor: UnitId) -> InputShape;

    fn resolve(&self, ctx: &mut SkillCtx, target: TargetSelection);

    fn hooks(&self) -> AbilityHooks { AbilityHooks::none() }
    fn on_event(&self, _ev: &CombatEvent, _ctx: &mut HookCtx) {}
    fn modifiers(&self, _snap: &Snapshot, _unit: UnitId) -> AbilityModifiers {
        AbilityModifiers::none()
    }
}
```

```rust
// src/combat/api/hook.rs (D023 — chain hook come fn Rust, niente enum dichiarativi)
//
// Pattern unificato con i passive BlueprintListener (D009): ogni hook è una closure
// `(event, ctx) → ctx.enqueue(Intent...)`. Il kernel emette HitEvent/KillEvent/MissEvent/
// FinalHopEvent per il deal corrente e li route alle closure registrate sull'Ability.
// Niente `enum OnHit`, niente `enum OnKill`, niente `enum OnFinalHop`.
pub type ChainHook<E> = Arc<dyn Fn(&E, &mut SkillCtx) + Send + Sync>;

pub struct ChainHooks {
    pub on_hit:        Option<ChainHook<HitEvent>>,
    pub on_kill:       Option<ChainHook<KillEvent>>,
    pub on_miss:       Option<ChainHook<MissEvent>>,
    pub on_final_hop:  Option<ChainHook<FinalHopEvent>>,  // chiude G6 in forma hook
}

pub struct HitEvent       { pub caster: UnitId, pub target: UnitId, pub damage: u32, pub hop_idx: u8, pub tag: DamageTag, pub cast_id: CastId }
pub struct KillEvent      { pub caster: UnitId, pub target: UnitId, pub overkill: u32, pub cast_id: CastId }
pub struct MissEvent      { pub caster: UnitId, pub target: UnitId, pub reason: MissReason, pub cast_id: CastId }
pub struct FinalHopEvent  { pub caster: UnitId, pub last_target: UnitId, pub hops_done: u8, pub cast_id: CastId }

pub enum StatusApplyMode { Stack, Refresh, MaxOf }  // §G1

pub enum TargetResolver {
    Fixed(UnitId),
    LowestHpPctAlive { scope: TeamScope, exclude: Vec<UnitId> },
    AdjLowest { primary: UnitId },
    NextSlotAlive { from: UnitId, dir: Dir },
    // ...
}
```

**Dry-run preview (D024).** `SkillCtx` espone `mode: SkillCtxMode { Live, DryRun }`. In `Live`, `ctx.enqueue(Intent)` finisce nella coda kernel real-cast. In `DryRun`, finisce in un buffer locale ritornato a `query_skill_preview` (D004). Le **stesse** hook fn girano in entrambi i mode: zero divergenza preview ↔ esecuzione anche per skill con branching (Agumon detonate condizionale, Dorumon chain Predator-only, Tentomon Bounce final-hop).

---

## 1. Mini-plugin condiviso: `twin_core` (Agumon ↔ Gabumon)

Pattern D005: shared mechanic come **terzo** mini-plugin, owner `"twin_core"`. Agumon e Gabumon dichiarano dipendenza state, **non** sono owner di Twin Core.

```rust
// src/combat/blueprints/twin_core/mod.rs
pub struct TwinCorePlugin;

#[derive(Resource, Default, Debug, Serialize)]
pub struct TwinCoreState {
    /// True iff both Agumon and Gabumon are alive and in the active party.
    pub paired: bool,
}

pub struct TwinCoreBlueprint;
impl Blueprint for TwinCoreBlueprint {
    type State = TwinCoreState;
    const ID: BlueprintId = "twin_core";
    fn build(app: &mut App) {
        app.register_blueprint_state::<TwinCoreState>();
        app.add_systems(Update, refresh_paired_state);  // exclusive system kernel-side
        // Niente Ability registrate: twin_core è solo state + listener interno.
        // Le 2 passive Twin Core Fire/Ice vivono dentro Agumon/Gabumon owner.
    }
}

// Constructor pubblico — niente Box<dyn Any> a mano nelle skill chiamanti.
// Non usato dal roster v0 (Twin Core è puro state passivo, niente signal write).
// Tenuto come surface aperta per future shared mechanic.
```

**Nota.** Twin Core non emette `Intent::BlueprintSignal` nel roster v0 — è puro state read-only consultato dalle passive di Agumon/Gabumon via `ctx.blueprint_state::<TwinCoreState>()`. Il signal pattern resta dimostrato dal Predator Loop (§4).

---

## 2. Agumon — Fire burst, Heated stacker

### 2.1 Plugin entry

```rust
// src/combat/blueprints/agumon/mod.rs
pub struct AgumonPlugin;

impl Plugin for AgumonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(AgumonBlueprint)
           .register_blueprint_state_dependency::<TwinCoreState>()  // validato in App::finish()
           .register_ability(abilities::sharp_claws())
           .register_ability(abilities::baby_flame())
           .register_ability(abilities::baby_burner())
           .register_ability(abilities::twin_core_fire());
    }
}

pub struct AgumonBlueprint;
impl Blueprint for AgumonBlueprint {
    type State = ();  // Niente state Agumon-specific (Twin Core è esterno)
    const ID: BlueprintId = "agumon";
    fn build(_app: &mut App) {}
}
```

### 2.2 Tuning

```rust
// src/combat/blueprints/agumon/tuning.rs
#[derive(Deserialize, Clone)] pub struct SharpClawsNumbers { pub base_dmg: i32, pub heated_apply: u8 }
#[derive(Deserialize, Clone)] pub struct BabyFlameNumbers {
    pub base_dmg: i32, pub sp_cost: i8, pub heated_apply: u8, pub toughness_hit: i32,
}
#[derive(Deserialize, Clone)] pub struct BabyBurnerNumbers {
    pub primary_dmg: i32, pub splash_pct: u32, pub detonate_per_stack: i32,
}
#[derive(Deserialize, Clone)] pub struct TwinCoreFireNumbers { pub fire_amp_pct: u32 } // 15
```

### 2.3 `sharp_claws` — Basic

```rust
pub fn sharp_claws() -> Arc<dyn Ability> {
    AbilityBuilder::active("agumon_sharp_claws")
        .owner("agumon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Fire])
        .tuning::<SharpClawsNumbers>(SHARP_CLAWS_HANDLE)
        .input(InputShape::single_enemy_alive())
        .legality(legality::self_alive_not_stunned)
        .cost(|_s, _a, _t| AbilityCost::basic())
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Fire)
                .on_hit(move |ev: &HitEvent, ctx| {
                    // Hook fn (D023): kernel emette HitEvent, hook enqueue Intent.
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.target,
                        kind: StatusEffectKind::Heated,
                        stacks_or_dur: t.heated_apply,
                        mode: StatusApplyMode::Stack,  // +1 per call, cap a 6 lato Status def
                    });
                })
                .done();
        })
        .build()
}
```

### 2.4 `baby_flame` — Skill

```rust
pub fn baby_flame() -> Arc<dyn Ability> {
    AbilityBuilder::active("agumon_baby_flame")
        .owner("agumon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Fire])
        .tuning::<BabyFlameNumbers>(BABY_FLAME_HANDLE)
        .input(InputShape::single_enemy_alive())
        .legality(legality::self_alive_not_stunned)
        .cost(|snap, actor, t| {
            // EnhancedNext buff (canon §5.8) → 0 SP. Auto-consume del status gestito dal kernel.
            if snap.has_status(actor, StatusEffectKind::Blessed)
                && snap.status_meta(StatusEffectKind::Blessed).consume_on_skill_cast {
                AbilityCost::free()
            } else {
                AbilityCost::sp(t.sp_cost)
            }
        })
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Fire)
                .toughness_hit(t.toughness_hit)
                .on_hit(move |ev: &HitEvent, ctx| {
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.target,
                        kind: StatusEffectKind::Heated,
                        stacks_or_dur: t.heated_apply,
                        mode: StatusApplyMode::Stack,
                    });
                })
                .done();
        })
        .build()
}
```

### 2.5 `baby_burner` — Ult (single primary + splash adj + on_kill→Detonate Heated)

```rust
pub fn baby_burner() -> Arc<dyn Ability> {
    AbilityBuilder::active("agumon_baby_burner")
        .owner("agumon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Fire])
        .tuning::<BabyBurnerNumbers>(BABY_BURNER_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_s, _a, _t| AbilityCost::ult_full())
        .resolve(|ctx, target, t| {
            let primary = target.primary();

            // Snapshot frozen: leggo Heated stacks pre-skill (sopravvive al kill primary).
            let stacks = ctx.status_stacks(primary, StatusEffectKind::Heated);
            let per_stack_dmg = t.detonate_per_stack;

            // Primary hit con hook on_kill → BlueprintSignal detonate.
            // La closure cattura `stacks` e `per_stack_dmg` dallo snapshot pre-skill.
            ctx.deal(primary)
                .damage(t.primary_dmg)
                .tag(DamageTag::Fire)
                .on_kill(move |ev: &KillEvent, ctx| {
                    ctx.enqueue(Intent::BlueprintSignal {
                        owner: BlueprintId::AGUMON,
                        signal: "detonate_heated",
                        payload: BlueprintPayload::new(DetonatePayload {
                            primary: ev.target,
                            stacks,
                            per_stack_dmg,
                        }),
                    });
                })
                .done();

            // Splash adj (50% del primary_dmg) — non chain hook, è un fan-out indipendente.
            for adj in ctx.adjacents(primary) {
                ctx.deal(adj)
                    .damage(t.primary_dmg * t.splash_pct as i32 / 100)
                    .tag(DamageTag::Fire)
                    .done();
            }
        })
        .build()
}

// Handler del signal — registrato dal AgumonPlugin (interno al blueprint, kernel-opaque).
// Resta funzione Rust pura: legge payload, enqueue Intent. Niente enum dichiarativi.
fn handle_detonate(payload: &DetonatePayload, ctx: &mut HookCtx) {
    for adj in ctx.adjacents(payload.primary) {
        ctx.enqueue(Intent::DealDamage {
            target: adj,
            base: payload.stacks as i32 * payload.per_stack_dmg,
            tag: DamageTag::Fire,
            chain_hooks: ChainHooks::none(),  // detonate non innesca ulteriori hook
            curve_hop: None,
        });
    }
}
```

### 2.6 `twin_core_fire` — Passive (Modifier conditional)

```rust
pub fn twin_core_fire() -> Arc<dyn Ability> {
    AbilityBuilder::passive("agumon_twin_core_fire")
        .owner("agumon")
        .category(AbilityCategory::Passive)
        .tuning::<TwinCoreFireNumbers>(TWIN_CORE_FIRE_HANDLE)
        .modifiers(|snap, unit, t| {
            // Triggera solo se il portatore è Agumon E paired con Gabumon vivo.
            if !snap.is_owner(unit, BlueprintId::AGUMON) {
                return AbilityModifiers::none();
            }
            let paired = snap
                .blueprint_state::<TwinCoreState>()
                .map_or(false, |s| s.paired);
            if !paired { return AbilityModifiers::none(); }

            // ×1.15 damage Fire quando il target ha Chilled (applicato da Gabumon).
            AbilityModifiers::single(Modifier {
                stage: ModifierStage::Multiplicative,
                value: 1.0 + (t.fire_amp_pct as f32 / 100.0),
                condition: Some(ModifierCondition::All(vec![
                    ModifierCondition::SourceUnit(unit),
                    ModifierCondition::SourceTag(DamageTag::Fire),
                    ModifierCondition::TargetHasStatus(StatusEffectKind::Chilled),
                ])),
                source: AbilityId::AGUMON_TWIN_CORE_FIRE,
                priority: 0,
            })
        })
        .build()
}
```

---

## 3. Gabumon — Ice bulwark, Chilled stacker + DR self

### 3.1 Plugin entry

```rust
// src/combat/blueprints/gabumon/mod.rs (era flat .rs, da promuovere a dir — §1.5 RESEARCH)
pub struct GabumonPlugin;
impl Plugin for GabumonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(GabumonBlueprint)
           .register_blueprint_state_dependency::<TwinCoreState>()
           .register_ability(abilities::claw_attack())
           .register_ability(abilities::gabumon_shot())
           .register_ability(abilities::blue_cyclone())
           .register_ability(abilities::fur_cloak())
           .register_ability(abilities::twin_core_ice());
        // 5 ability: 3 active + 2 passive (fur_cloak + twin_core_ice)
    }
}
```

### 3.2 `claw_attack` — Basic (Ice)

```rust
pub fn claw_attack() -> Arc<dyn Ability> {
    AbilityBuilder::active("gabumon_claw_attack")
        .owner("gabumon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Ice])
        .tuning::<ClawAttackNumbers>(CLAW_ATTACK_HANDLE)
        .input(InputShape::single_enemy_alive())
        .legality(legality::self_alive_not_stunned)
        .cost(|_, _, _| AbilityCost::basic())
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Ice)
                .on_hit(move |ev: &HitEvent, ctx| {
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.target,
                        kind: StatusEffectKind::Chilled,
                        stacks_or_dur: t.chilled_apply,
                        mode: StatusApplyMode::Stack,
                    });
                })
                .done();
        })
        .build()
}
```

### 3.3 `gabumon_shot` — Skill (apply +2 Chilled + echo +1 Chilled adj lowest)

```rust
pub fn gabumon_shot() -> Arc<dyn Ability> {
    AbilityBuilder::active("gabumon_shot")
        .owner("gabumon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Ice])
        .tuning::<GabumonShotNumbers>(GABUMON_SHOT_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, t| AbilityCost::sp(t.sp_cost))
        .resolve(|ctx, target, t| {
            let primary = target.primary();

            // Hit primario: damage + Chilled +2 + toughness hit
            ctx.deal(primary)
                .damage(t.base_dmg)
                .tag(DamageTag::Ice)
                .toughness_hit(t.toughness_hit)
                .on_hit(move |ev: &HitEvent, ctx| {
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.target,
                        kind: StatusEffectKind::Chilled,
                        stacks_or_dur: t.chilled_apply_primary,
                        mode: StatusApplyMode::Stack,
                    });
                })
                .done();

            // Echo reattivo: +1 Chilled sull'adj lowest-HP%, tie-break slot_index asc.
            // Snapshot frozen: targeting valutato pre-skill.
            if let Some(adj) = ctx.adj_lowest_hp_pct(primary) {
                ctx.enqueue(Intent::ApplyStatus {
                    target: adj,
                    kind: StatusEffectKind::Chilled,
                    duration: t.chilled_apply_echo as u32,
                    mode: StatusApplyMode::Stack,  // ← gap §G1
                });
            }
        })
        .build()
}
```

### 3.4 `blue_cyclone` — Ult (massive Ice + Slowed + DR self 30%)

```rust
pub fn blue_cyclone() -> Arc<dyn Ability> {
    AbilityBuilder::active("gabumon_blue_cyclone")
        .owner("gabumon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Ice])
        .tuning::<BlueCycloneNumbers>(BLUE_CYCLONE_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::ult_full())
        .resolve(|ctx, target, t| {
            let actor = ctx.actor();
            let primary = target.primary();

            ctx.deal(primary)
                .damage(t.primary_dmg)
                .tag(DamageTag::Ice)
                .on_hit(move |ev: &HitEvent, ctx| {
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.target,
                        kind: StatusEffectKind::Slowed,
                        stacks_or_dur: t.slowed_dur,
                        mode: StatusApplyMode::Refresh,
                    });
                })
                .done();

            // Self DR 30% 1 turno — Buff tag-pure (non Status, vedi §G5).
            ctx.enqueue(Intent::ApplyBuff {
                target: actor,
                kind: BuffKind::DefenseUp { pct: t.self_dr_pct, dur: t.self_dr_dur },
                stack_mode: BuffStackMode::MaxReplace,  // tie-break a `fur_cloak` 20%
            });
        })
        .build()
}
```

### 3.5 `fur_cloak` — Passive (listener: OnStatusApplied Chilled by self → DR self 20% 1 turn)

```rust
pub fn fur_cloak() -> Arc<dyn Ability> {
    AbilityBuilder::passive("gabumon_fur_cloak")
        .owner("gabumon")
        .category(AbilityCategory::Passive)
        .tuning::<FurCloakNumbers>(FUR_CLOAK_HANDLE)
        .hooks(AbilityHooks::on_event(
            AbilityEvent::StatusApplied { kind: StatusEffectKind::Chilled },
            EventFilter::SourceIsOwner,  // solo se Chilled è stato applicato da Gabumon stesso
        ))
        .on_event(|ev, ctx, t| {
            // ev.source = unit Gabumon che ha appena applicato Chilled
            ctx.enqueue(Intent::ApplyBuff {
                target: ev.source,
                kind: BuffKind::DefenseUp { pct: t.dr_pct, dur: t.dr_dur },
                stack_mode: BuffStackMode::MaxReplace,
            });
        })
        .build()
}
```

### 3.6 `twin_core_ice` — Passive (Modifier: Ice ×1.15 se target Heated)

```rust
pub fn twin_core_ice() -> Arc<dyn Ability> {
    AbilityBuilder::passive("gabumon_twin_core_ice")
        .owner("gabumon")
        .category(AbilityCategory::Passive)
        .tuning::<TwinCoreIceNumbers>(TWIN_CORE_ICE_HANDLE)
        .modifiers(|snap, unit, t| {
            if !snap.is_owner(unit, BlueprintId::GABUMON) { return AbilityModifiers::none(); }
            let paired = snap.blueprint_state::<TwinCoreState>().map_or(false, |s| s.paired);
            if !paired { return AbilityModifiers::none(); }

            AbilityModifiers::single(Modifier {
                stage: ModifierStage::Multiplicative,
                value: 1.0 + (t.ice_amp_pct as f32 / 100.0),
                condition: Some(ModifierCondition::All(vec![
                    ModifierCondition::SourceUnit(unit),
                    ModifierCondition::SourceTag(DamageTag::Ice),
                    ModifierCondition::TargetHasStatus(StatusEffectKind::Heated),
                ])),
                source: AbilityId::GABUMON_TWIN_CORE_ICE,
                priority: 0,
            })
        })
        .build()
}
```

---

## 4. Dorumon — Single-target executor Dark, Predator state

### 4.1 Plugin entry (con state proprio)

```rust
// src/combat/blueprints/dorumon/mod.rs (già dir idiomatica)
pub struct DorumonPlugin;
impl Plugin for DorumonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(DorumonBlueprint)
           .register_ability(abilities::bite())
           .register_ability(abilities::dash_metal())
           .register_ability(abilities::metal_cannon())
           .register_ability(abilities::predator_loop());
    }
}

#[derive(Resource, Default, Debug, Serialize)]
pub struct PredatorLoopState {
    pub active: bool,
    pub tracked: Option<UnitId>,
    pub turns_remaining: u8,
}

pub struct DorumonBlueprint;
impl Blueprint for DorumonBlueprint {
    type State = PredatorLoopState;   // owner = "dorumon" possiede lo state
    const ID: BlueprintId = "dorumon";
    fn build(app: &mut App) {
        // Signal handler: route Enter/Exit/Tick a system tipato.
        app.add_observer(on_predator_signal);
    }
}
```

### 4.2 `bite` — Basic

```rust
pub fn bite() -> Arc<dyn Ability> {
    AbilityBuilder::active("dorumon_bite")
        .owner("dorumon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Dark])
        .tuning::<BiteNumbers>(BITE_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::basic())
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Dark)
                .done();
        })
        .build()
}
```

### 4.3 `dash_metal` — Skill (threshold scaling + on_kill→Chain in Predator state)

```rust
pub fn dash_metal() -> Arc<dyn Ability> {
    AbilityBuilder::active("dorumon_dash_metal")
        .owner("dorumon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Dark])
        .tuning::<DashMetalNumbers>(DASH_METAL_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, t| AbilityCost::sp(t.sp_cost))
        .resolve(|ctx, target, t| {
            let primary = target.primary();
            let hp_pct = ctx.hp_pct(primary);

            // Scaling threshold: ×2 damage se primary HP <50%.
            let base = if hp_pct < 50 { t.base_dmg * 2 } else { t.base_dmg };

            // Hook on_kill condizionato a Predator state. Decisione presa dal corpo
            // della closure, non dichiarata via enum — vantaggio D023: branching
            // libero, kernel invariante. Chain cap a 1 (skill virtuale dash_metal_chain
            // non porta proprio on_kill).
            let predator_active = ctx
                .blueprint_state::<PredatorLoopState>()
                .map_or(false, |s| s.active);

            let mut deal = ctx.deal(primary).damage(base).tag(DamageTag::Dark);

            if predator_active {
                deal = deal.on_kill(move |ev: &KillEvent, ctx| {
                    // LowestHpPctAlive escludendo il primary appena ucciso.
                    let next_target = ctx
                        .lowest_hp_pct_alive(SideFilter::Enemy)
                        .filter(|u| *u != ev.target);
                    if let Some(t) = next_target {
                        ctx.enqueue(Intent::EnqueueFollowUp {
                            skill: AbilityId::DORUMON_DASH_METAL_CHAIN,
                            target: TargetResolver::Fixed(t),
                        });
                    }
                });
            }

            deal.done();
        })
        .build()
}

// Skill virtuale: il "chain" è una copia di dash_metal con `chain_armed = false`
// per evitare ricorsione (max 1 chain — §5 Dorumon).
// Registrata come ability separata con stesso owner.
pub fn dash_metal_chain() -> Arc<dyn Ability> {
    AbilityBuilder::active("dorumon_dash_metal_chain")
        .owner("dorumon")
        .category(AbilityCategory::FollowUp)
        .tags(&[DamageTag::Dark, AbilityTag::ChainProc])
        .legality(|_, _| Legality::Allowed)   // legality kernel-skipped per FollowUp
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg * 2)  // assume HP <50% per chain target
                .tag(DamageTag::Dark)
                // niente on_kill: chain cap a 1
                .done();
        })
        .build()
}
```

### 4.4 `metal_cannon` — Ult (massive single + forza Predator state)

```rust
pub fn metal_cannon() -> Arc<dyn Ability> {
    AbilityBuilder::active("dorumon_metal_cannon")
        .owner("dorumon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Dark])
        .tuning::<MetalCannonNumbers>(METAL_CANNON_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::ult_full())
        .resolve(|ctx, target, t| {
            let primary = target.primary();
            let hp_pct = ctx.hp_pct(primary);

            // Bonus +50% se primary <30%.
            let base = if hp_pct < 30 { t.base_dmg * 3 / 2 } else { t.base_dmg };

            ctx.deal(primary)
                .damage(base)
                .tag(DamageTag::Dark)
                .on_hit(move |ev: &HitEvent, ctx| {
                    ctx.enqueue(Intent::BlueprintSignal {
                        owner: BlueprintId::DORUMON,
                        signal: "predator_force_enter",
                        payload: BlueprintPayload::new(PredatorEnterPayload {
                            tracked: ev.target,
                            forced: true,
                        }),
                    });
                })
                .done();
        })
        .build()
}
```

### 4.5 `predator_loop` — Passive (listener: enemy HP falls below threshold → enter state)

```rust
pub fn predator_loop() -> Arc<dyn Ability> {
    AbilityBuilder::passive("dorumon_predator_loop")
        .owner("dorumon")
        .category(AbilityCategory::Passive)
        .tuning::<PredatorLoopNumbers>(PREDATOR_LOOP_HANDLE)
        .hooks(AbilityHooks::multi(&[
            AbilityHook::on_event(
                AbilityEvent::HpChanged { side: SideFilter::Enemy },
                EventFilter::Any,
            ),
            AbilityHook::on_event(
                AbilityEvent::UnitDied { side: SideFilter::Enemy },
                EventFilter::Any,
            ),
        ]))
        .on_event(|ev, ctx, t| {
            match ev {
                AbilityEvent::HpChanged { unit, new_pct, .. } if *new_pct < t.hp_threshold => {
                    ctx.enqueue(Intent::BlueprintSignal {
                        owner: BlueprintId::DORUMON,
                        signal: "predator_enter",
                        payload: BlueprintPayload::new(PredatorEnterPayload {
                            tracked: *unit, forced: false,
                        }),
                    });
                }
                AbilityEvent::UnitDied { unit, .. } => {
                    // Exit se tracked == unit (gestito dall'observer interno
                    // di DorumonPlugin che possiede PredatorLoopState).
                    ctx.enqueue(Intent::BlueprintSignal {
                        owner: BlueprintId::DORUMON,
                        signal: "predator_check_exit",
                        payload: BlueprintPayload::new(PredatorExitPayload { killed: *unit }),
                    });
                }
                _ => {}
            }
        })
        .build()
}
```

---

## 5. Tentomon — Battery + tank-lite, Paralyzed spreader

### 5.1 Plugin entry

```rust
// src/combat/blueprints/tentomon/mod.rs (era flat .rs, da promuovere a dir)
pub struct TentomonPlugin;
impl Plugin for TentomonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(TentomonBlueprint)
           .register_ability(abilities::hard_claw())
           .register_ability(abilities::petit_thunder())
           .register_ability(abilities::electrical_discharge())
           .register_ability(abilities::battery_loop());
    }
}
```

### 5.2 `hard_claw` — Basic (+2 SP gen, override del +1 canon)

```rust
pub fn hard_claw() -> Arc<dyn Ability> {
    AbilityBuilder::active("tentomon_hard_claw")
        .owner("tentomon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Electric])
        .tuning::<HardClawNumbers>(HARD_CLAW_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::basic())  // +1 SP gen canon
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Electric)
                .done();
            // Il +1 extra (totale +2) lo emette `battery_loop` passive
            // sull'evento OnBasicAttack — vedi §5.5.
        })
        .build()
}
```

### 5.3 `petit_thunder` — Skill (Bounce(3) + on_final_hop→Paralyzed + DR self)

```rust
pub fn petit_thunder() -> Arc<dyn Ability> {
    AbilityBuilder::active("tentomon_petit_thunder")
        .owner("tentomon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Electric])
        .tuning::<PetitThunderNumbers>(PETIT_THUNDER_HANDLE)
        .input(InputShape::single_enemy_alive())  // 1 selezione, bounce derivato
        .cost(|_, _, t| AbilityCost::sp(t.sp_cost))
        .resolve(|ctx, target, t| {
            let actor = ctx.actor();

            // Bounce(3, LowestHpAlive) — kernel risolve la chain hop-per-hop con
            // snapshot rifresca per hop (vedi memory: TargetableSnapshot per-hop).
            ctx.bounce(target.primary(), t.hops)
                .selector(BounceSelector::LowestHpAlive { exclude_self: true })
                .damage_curve(DamageCurve::PerHop(vec![t.dmg_hop1, t.dmg_hop2, t.dmg_hop3]))
                .tag(DamageTag::Electric)
                .on_final_hop(move |ev: &FinalHopEvent, ctx| {
                    // Hook fn (D023): closure su FinalHopEvent — kernel chiama solo
                    // sull'ultimo hop, niente discriminazione data-driven. Chiude G6.
                    ctx.enqueue(Intent::ApplyStatus {
                        target: ev.last_target,
                        kind: StatusEffectKind::Paralyzed,
                        stacks_or_dur: t.paralyzed_dur,
                        mode: StatusApplyMode::Refresh,
                    });
                })
                .done();

            // DR self 25% 1 turno (tank-lite hook).
            ctx.enqueue(Intent::ApplyBuff {
                target: actor,
                kind: BuffKind::DefenseUp { pct: t.self_dr_pct, dur: t.self_dr_dur },
                stack_mode: BuffStackMode::MaxReplace,
            });
        })
        .build()
}
```

### 5.4 `electrical_discharge` — Ult (AoE + Paralyzed random + +1 SP team)

```rust
pub fn electrical_discharge() -> Arc<dyn Ability> {
    AbilityBuilder::active("tentomon_electrical_discharge")
        .owner("tentomon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Electric])
        .tuning::<ElectricalDischargeNumbers>(ELECTRICAL_DISCHARGE_HANDLE)
        .input(InputShape::AoE { side: Side::Enemy })
        .cost(|_, _, _| AbilityCost::ult_full())
        .resolve(|ctx, _target, t| {
            // Damage AoE su tutti i nemici vivi.
            for enemy in ctx.alive_enemies() {
                ctx.deal(enemy)
                    .damage(t.aoe_dmg)
                    .tag(DamageTag::Electric)
                    .done();
            }

            // Paralyzed "su 1 random" — gap deterministico §G7.
            // Proposta: lowest-HP-pct con tie-break slot_index asc (no RNG).
            if let Some(victim) = ctx.lowest_hp_pct_alive(SideFilter::Enemy) {
                ctx.enqueue(Intent::ApplyStatus {
                    target: victim,
                    kind: StatusEffectKind::Paralyzed,
                    duration: t.paralyzed_dur as u32,
                    mode: StatusApplyMode::Refresh,
                });
            }

            // +1 SP team (battery moment).
            ctx.enqueue(Intent::GainSp { side: ctx.actor_team(), amount: 1 });
        })
        .build()
}
```

### 5.5 `battery_loop` — Passive (dual-path: SP side-channel + Block reaction)

```rust
pub fn battery_loop() -> Arc<dyn Ability> {
    AbilityBuilder::passive("tentomon_battery_loop")
        .owner("tentomon")
        .category(AbilityCategory::Passive)
        .tuning::<BatteryLoopNumbers>(BATTERY_LOOP_HANDLE)
        .hooks(AbilityHooks::multi(&[
            // Path A — SP side-channel: +1 SP extra quando un alleato fa basic.
            AbilityHook::on_event(
                AbilityEvent::BasicAttackResolved { side: SideFilter::Ally },
                EventFilter::AllyOfOwner,
            ),
            // Path B — Block reaction: incoming damage su Tentomon con SP team ≥3.
            AbilityHook::on_event(
                AbilityEvent::IncomingDamage { target_owner: BlueprintId::TENTOMON },
                EventFilter::Self_,
            ),
        ]))
        .on_event(|ev, ctx, t| match ev {
            AbilityEvent::BasicAttackResolved { actor, .. } => {
                // +1 SP extra (sommato al +1 canon di base) → totale +2 SP per basic dell'alleato.
                ctx.enqueue(Intent::GainSp { side: ctx.team_of(*actor), amount: 1 });
            }
            AbilityEvent::IncomingDamage { target, sp_team, .. } if *sp_team >= t.block_sp_gate => {
                // DR 50% pre-formula via Modifier ad-hoc proc-only — gap §G8.
                ctx.enqueue(Intent::ApplyBuff {
                    target: *target,
                    kind: BuffKind::DefenseUp {
                        pct: t.block_proc_dr_pct,  // 50
                        dur: 0,  // ← 0 = one-shot proc, applicato solo al next IncomingDamage
                    },
                    stack_mode: BuffStackMode::ProcOnce,
                });
            }
            _ => {}
        })
        .build()
}
```

---

## 6. Patamon — Support-healer Holy, AoE hybrid ult

### 6.1 Plugin entry

```rust
// src/combat/blueprints/patamon/mod.rs (già dir idiomatica)
pub struct PatamonPlugin;
impl Plugin for PatamonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(PatamonBlueprint)
           .register_ability(abilities::tai_atari())
           .register_ability(abilities::patapata_hover())
           .register_ability(abilities::sparking_air_shot())
           .register_ability(abilities::holy_aegis());
    }
}
```

### 6.2 `tai_atari` — Basic (single Holy)

```rust
pub fn tai_atari() -> Arc<dyn Ability> {
    AbilityBuilder::active("patamon_tai_atari")
        .owner("patamon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Holy])
        .tuning::<TaiAtariNumbers>(TAI_ATARI_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::basic())
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary())
                .damage(t.base_dmg)
                .tag(DamageTag::Holy)
                .done();
        })
        .build()
}
```

### 6.3 `patapata_hover` — Skill (heal % max + cleanse 1 debuff oldest-first)

```rust
pub fn patapata_hover() -> Arc<dyn Ability> {
    AbilityBuilder::active("patamon_patapata_hover")
        .owner("patamon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Holy, AbilityTag::Heal, AbilityTag::Cleanse])
        .tuning::<PatapataHoverNumbers>(PATAPATA_HOVER_HANDLE)
        .input(InputShape::single_ally_alive())
        .cost(|_, _, t| AbilityCost::sp(t.sp_cost))
        .resolve(|ctx, target, t| {
            let ally = target.primary();
            ctx.enqueue(Intent::Heal { target: ally, amount_pct_max_hp: t.heal_pct });

            // Cleanse 1 debuff: oldest-first (FIFO), tie-break ID asc — selettore kernel-canon.
            ctx.enqueue(Intent::Cleanse {
                target: ally,
                count: Some(1),
                filter: CleanseFilter::DebuffsOnly,  // Blessed (buff) escluso
            });
        })
        .build()
}
```

### 6.4 `sparking_air_shot` — Ult (AoE damage enemies + AoE heal+cleanse allies)

```rust
pub fn sparking_air_shot() -> Arc<dyn Ability> {
    AbilityBuilder::active("patamon_sparking_air_shot")
        .owner("patamon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Holy, AbilityTag::Heal, AbilityTag::Cleanse])
        .tuning::<SparkingAirShotNumbers>(SPARKING_AIR_SHOT_HANDLE)
        .input(InputShape::AoE { side: Side::Both })
        .cost(|_, _, _| AbilityCost::ult_full())
        .resolve(|ctx, _target, t| {
            for enemy in ctx.alive_enemies() {
                ctx.deal(enemy)
                    .damage(t.aoe_dmg)
                    .tag(DamageTag::Holy)
                    .done();
            }
            for ally in ctx.alive_allies() {
                ctx.enqueue(Intent::Heal { target: ally, amount_pct_max_hp: t.heal_pct });
                ctx.enqueue(Intent::Cleanse {
                    target: ally, count: Some(1), filter: CleanseFilter::DebuffsOnly,
                });
            }
        })
        .build()
}
```

### 6.5 `holy_aegis` — Passive (Modifier: -10% damage taken team, self-inclusive)

```rust
pub fn holy_aegis() -> Arc<dyn Ability> {
    AbilityBuilder::passive("patamon_holy_aegis")
        .owner("patamon")
        .category(AbilityCategory::Passive)
        .tuning::<HolyAegisNumbers>(HOLY_AEGIS_HANDLE)
        .modifiers(|snap, unit, t| {
            // Aegis attivo solo se Patamon è vivo (any in party).
            let patamon_alive = snap
                .iter_alive_owners()
                .any(|o| o == BlueprintId::PATAMON);
            if !patamon_alive { return AbilityModifiers::none(); }

            // Si applica a tutti gli alleati (Patamon incluso — §7 Q2).
            // Stage Additive per stack-additivo (non moltiplica) con `fur_cloak` Gabumon.
            AbilityModifiers::single(Modifier {
                stage: ModifierStage::Additive,
                value: -(t.dr_pct as f32 / 100.0),  // -0.10 = -10%
                condition: Some(ModifierCondition::All(vec![
                    ModifierCondition::TargetIsAlly(BlueprintId::PATAMON),
                    ModifierCondition::Pipeline(PipelineStep::IncomingDamage),
                ])),
                source: AbilityId::PATAMON_HOLY_AEGIS,
                priority: 0,
            })
        })
        .build()
}
```

---

## 7. Renamon — AoE Holy + Time manip

### 7.1 Plugin entry

```rust
// src/combat/blueprints/renamon/mod.rs (era flat .rs, da promuovere a dir)
pub struct RenamonPlugin;
impl Plugin for RenamonPlugin {
    fn build(&self, app: &mut App) {
        app.register_blueprint(RenamonBlueprint)
           .register_ability(abilities::kokaishu())
           .register_ability(abilities::koyosetsu())
           .register_ability(abilities::tohakken())
           .register_ability(abilities::kitsune_grace());
        // `Blessed` taxonomy/effect è registrata via global status meta — vedi §G4.
    }
}
```

### 7.2 `kokaishu` — Basic (single Holy, +1 SP gen std)

```rust
pub fn kokaishu() -> Arc<dyn Ability> {
    AbilityBuilder::active("renamon_kokaishu")
        .owner("renamon")
        .category(AbilityCategory::Basic)
        .tags(&[DamageTag::Holy])
        .tuning::<KokaishuNumbers>(KOKAISHU_HANDLE)
        .input(InputShape::single_enemy_alive())
        .cost(|_, _, _| AbilityCost::basic())
        .resolve(|ctx, target, t| {
            ctx.deal(target.primary()).damage(t.base_dmg).tag(DamageTag::Holy).done();
        })
        .build()
}
```

### 7.3 `koyosetsu` — Skill (AoE Holy + SelfAdvance 25%)

```rust
pub fn koyosetsu() -> Arc<dyn Ability> {
    AbilityBuilder::active("renamon_koyosetsu")
        .owner("renamon")
        .category(AbilityCategory::Skill)
        .tags(&[DamageTag::Holy])
        .tuning::<KoyosetsuNumbers>(KOYOSETSU_HANDLE)
        .input(InputShape::AoE { side: Side::Enemy })
        .cost(|_, _, t| AbilityCost::sp(t.sp_cost))
        .resolve(|ctx, _target, t| {
            for enemy in ctx.alive_enemies() {
                ctx.deal(enemy).damage(t.aoe_dmg).tag(DamageTag::Holy).done();
            }
            // AdvanceTurn self 25% (cap ±50% applicato a emission lato kernel).
            ctx.enqueue(Intent::SelfAdvance { pct: t.self_advance_pct });
        })
        .build()
}
```

### 7.4 `tohakken` — Ult (AoE Holy + DelayTurn enemies 30% + Blessed allies)

```rust
pub fn tohakken() -> Arc<dyn Ability> {
    AbilityBuilder::active("renamon_tohakken")
        .owner("renamon")
        .category(AbilityCategory::Ult)
        .tags(&[DamageTag::Holy])
        .tuning::<TohakkenNumbers>(TOHAKKEN_HANDLE)
        .input(InputShape::AoE { side: Side::Both })
        .cost(|_, _, _| AbilityCost::ult_full())
        .resolve(|ctx, _target, t| {
            for enemy in ctx.alive_enemies() {
                ctx.deal(enemy).damage(t.aoe_dmg).tag(DamageTag::Holy).done();
                ctx.enqueue(Intent::DelayTurn { unit: enemy, pct: t.delay_pct });
            }
            for ally in ctx.alive_allies() {
                ctx.enqueue(Intent::ApplyStatus {
                    target: ally,
                    kind: StatusEffectKind::Blessed,
                    duration: t.blessed_dur as u32,
                    mode: StatusApplyMode::Refresh,
                });
            }
        })
        .build()
}
```

### 7.5 `kitsune_grace` — Passive (listener: ally non-self consumes Ult → AdvanceTurn self 10%)

```rust
pub fn kitsune_grace() -> Arc<dyn Ability> {
    AbilityBuilder::passive("renamon_kitsune_grace")
        .owner("renamon")
        .category(AbilityCategory::Passive)
        .tuning::<KitsuneGraceNumbers>(KITSUNE_GRACE_HANDLE)
        .hooks(AbilityHooks::on_event(
            AbilityEvent::UltConsumed,
            EventFilter::All(vec![
                EventFilter::AllyOfOwner,    // alleato di Renamon
                EventFilter::Not(Box::new(EventFilter::Self_)),  // != Renamon stessa
            ]),
        ))
        .on_event(|_ev, ctx, t| {
            // Trova ogni Renamon viva in party (>1 Renamon non previsto v0, ma robust).
            for renamon in ctx.alive_units_with_owner(BlueprintId::RENAMON) {
                ctx.enqueue(Intent::AdvanceTurn {
                    unit: renamon,
                    pct: t.advance_pct,  // 10
                });
            }
        })
        .build()
}

// Modifier intrinseci di `Blessed` (+15% dmg dealt + Ult charge bonus) — vedi §G4.
```

---

## 8. Gap analysis

Punti dove la stesura ha rivelato un gap rispetto all'API target di `M021-RESEARCH.md` §5. Ogni gap va valutato e o (a) chiuso aggiornando il RESEARCH, o (b) descoped esplicitamente.

> **Nota strutturale post-stesura (D023 + D024).** La prima draft di questa stesura usava `enum OnHit { ApplyStatus, Heal, GainSp, BlueprintSignal, EnqueueFollowUp }` (più gemelli `OnKill`/`OnMiss`/`OnFinalHop`) come record dichiarativo per gli effetti side-chain. La revisione ha rilevato un'**inconsistenza** con D009: i `BlueprintListener` passive sono già fn Rust `(Event, Ctx)→Vec<Intent>`, ma i chain hook erano enum match. D023 promuove **tutti** i chain hook (`on_cast`, `on_hit`, `on_kill`, `on_miss`, `on_final_hop`) a hook fn coerenti col pattern listener. D024 introduce `SkillCtx::Mode { Live, DryRun }` per consentire preview UI/AI delle stesse fn senza side-effect. Tutti i campioni in §2–§7 sono stati riscritti col pattern hook fn. La struttura dei gap restanti (G1, G3–G16) non cambia: continuano a riguardare dati (`StatusApplyMode`, `BuffKind`, `ModifierCondition`, `EventFilter`), non più la forma delle hook.

### G1 — `StatusApplyMode`: Stack vs Refresh vs MaxOf

**Sintomo.** Heated/Chilled si **stackano** (+1 per call, cap 6, decay -1/turno). Status canon `Slowed`/`Paralyzed` si **refreshano** (durata = max(old, new)). `Blessed` idem refresh.
Da memory: «StatusBag.apply oggi fa `max(old, new)` silente». L'`Intent::ApplyStatus { duration }` non distingue le due semantiche.

**Proposta.** Estendere `Intent::ApplyStatus` con `mode: StatusApplyMode { Stack, Refresh, MaxOf }`. Default = `Refresh` (compat con StatusBag attuale). Skill autore dichiara esplicito. Cap (6 Heated, etc.) resta side della `StatusDef` (data, non kernel-hardcoded).

**Decisione.** Da chiudere prima di S05 (Intent canon). Documentare come decisione **D016** se accettata.

---

### G2 — Heated decay -1/turno target-side: chi lo applica?

**Sintomo.** §5 Agumon: «Heated -1 stack su `TurnEnded` del target». Status canon `tick_all` oggi fa `dur -= 1` per turno. Coincide con "decay -1 stack" se mode=Stack e cap-driven. Ma allora il decay è un side-effect generico di Status, non di Heated. OK se la kernel `StatusBag::tick_all` decrementa la **count/duration** per tutti gli status, senza Digimon-specific.

**Decisione.** Nessun gap, ma documentare la semantica: «stacks-as-duration» per status di tipo `Stack`. Heated decay = generico tick. Kernel resta agnostic.

---

### G3 — `Modifier::condition` ricco: source/target/tag/pipeline

**Sintomo.** Twin Core Fire passive richiede `condition = SourceUnit ∧ SourceTag(Fire) ∧ TargetHasStatus(Chilled)`. `Holy Aegis` richiede `condition = TargetIsAlly(patamon) ∧ Pipeline(IncomingDamage)`. `Modifier` originale (§5.12) ha solo `condition: Option<ModifierCondition>` enum chiuso.

**Proposta.** `ModifierCondition` deve supportare almeno: `SourceUnit(UnitId)`, `SourceTag(DamageTag)`, `TargetHasStatus(kind)`, `TargetIsAlly(blueprint_owner)`, `TargetIsEnemy`, `Pipeline(PipelineStep)`, combinator `All(vec)` / `Any(vec)` / `Not(box)`. Lista chiusa, crescibile.

**Decisione.** Chiudere come parte di S07 (AbilityBuilder + modifier API). Documentare come **D017** la lista canon di ModifierCondition.

---

### G4 — `Blessed` non è puro tag: ha effetti intrinseci kernel-visibili

**Sintomo.** Renamon `tohakken` applica `Blessed` 2 turni → bonus +15% dmg dealt + +1 Ult charge per action. Questi sono Modifier kernel-applicati. Domanda: dove vive l'`AbilityModifiers` di Blessed?
- (a) Modifier registrato dal `RenamonPlugin` con condition `TargetHasStatus(Blessed)` — ma allora **se Renamon muore**, il blessed sui sopravvissuti smette di funzionare (Modifier pipeline iter solo su `iter_alive_abilities`).
- (b) Status canon ha **effects intrinsic** registrati con la StatusDef in `StatusBag` (kernel-side data, non Digimon-specific) — Blessed effects = `{+15% dmg dealt, +1 ult charge per action}` definito come data nel kernel.

**Proposta.** Opzione (b) — Status canon ha effects intrinsic registrabili come parte di `StatusDef` (data-driven, non kernel-hardcoded). Eviti il "Renamon muore → Blessed si spegne" paradosso. Pattern simmetrico per Heated (+8% Fire vulnerability/stack) e Chilled. **Implica**: `StatusDef` cresce con un campo `intrinsic_modifiers: Vec<ModifierTemplate>` data-driven.

Conseguenza: il modifier pipeline va consultato anche per **status-intrinsic**, non solo per `Ability::modifiers()`. Aggiungere stage di aggregation: «raccogli modifier da ability viventi + raccogli modifier da status attivi sul target/source».

**Decisione.** Chiudere come **D018** prima di S05. Senza questo, Renamon Blessed non funziona pulito.

---

### G5 — `BuffKind` separato da `StatusEffectKind`

**Sintomo.** Gabumon `fur_cloak` e `blue_cyclone` applicano DR self come Buff tag-pure (presence-only), `value` in stringy map. Tentomon `petit_thunder` idem. Da Gabumon §7 D2: «nuovo componente `Buff_*DR` tag-pure (presence-only) + valore in stringy Buffs map».

**Proposta.** Aggiungere taxonomy `BuffKind { DefenseUp { pct, dur }, AttackUp, SpeedUp, … }` separata da `StatusEffectKind`. `Intent::ApplyBuff { target, kind: BuffKind, stack_mode: BuffStackMode }`. Stack mode: `MaxReplace | ProcOnce | Additive`. **Rationale**: status hanno `stacks/duration` numerici e tick canon; buff hanno `value` mult e durata indipendente. Conflate i due = bug futuri.

**Implicazione kernel.** `Intent` canon cresce a 18 variant. `BuffBag` parallelo a `StatusBag`. `Modifier::source` può essere `ModifierSource::Buff(BuffKind)` o `ModifierSource::Ability(AbilityId)`.

**Decisione.** Chiudere come **D019** prima di S05 (Intent canon). Non è un add-on opzionale: 4/6 Digimon ne hanno bisogno (Gabumon ×2, Tentomon ×2, indirettamente Patamon).

---

### G6 — `on_final_hop` per Bounce: applicare effetto solo all'ultimo hop

**Sintomo.** Tentomon `petit_thunder` Bounce(3) deve applicare `Paralyzed` **solo all'hit finale**, non a ogni hop. Un hook generico `on_hit` viene chiamato per ogni hit del DealDamage; non discrimina "ultimo hop". Da memory: «DamageCurve::PerHop runtime length guard». DamageCurve già sa quale hop è in corso.

**Proposta (post-D023).** Capability slot `on_final_hop: Option<ChainHook<FinalHopEvent>>` come fratello di `on_hit`/`on_kill`/`on_miss` (vedi §0). Il kernel emette un solo `FinalHopEvent` al completamento della chain e route alla closure registrata. Niente enum dichiarativo, niente discriminazione data-driven dentro `on_hit`. La closure capta tuning da snapshot frozen come tutti gli altri hook.

**Decisione.** Chiudere come **D020** prima di S07 (Bounce builder), con il vincolo aggiuntivo che `on_final_hop` è **hook fn coerente** con D023, non un campo enum. Pattern raro (1/24 skill) ma serve, e codifica una semantica "ultimo hop" che AI/UI tooltip già vorranno mostrare via dry-run (D024).

---

### G7 — "Random target" → re-tipizzato come selettore deterministico

**Sintomo.** Tentomon `electrical_discharge` ult: «Paralyzed su 1 random» (canon). Determinismo richiede no-RNG senza seed. La risoluzione `random` va sostituita con un selector deterministico documentato come "lowest-hp-pct, tie-break slot_index asc".

**Proposta.** Non aggiungere `Random` selector. Documentare nel design draft Tentomon (§4) che "random" è retoricato in `LowestHpPctAlive`. Selettore già esistente, nessuna API change.

**Decisione.** Aggiornare `tentomon/03_ult_electrical_discharge.md` con la rettifica, non serve modifier kernel.

---

### G8 — `BuffStackMode::ProcOnce` per block reaction one-shot

**Sintomo.** Tentomon `battery_loop` Path B (Block reaction): quando incoming damage hit Tentomon con SP team ≥3, applicare DR 50% **solo al next IncomingDamage**, poi consumare. Non è durata 1 turno: è "1 proc only".

**Proposta.** `BuffStackMode::ProcOnce` (consume_on_next_pipeline_eval) o capability slot dedicato (intercept_incoming_damage). Preferenza: `ProcOnce` come variant di `BuffStackMode` — pattern modesto, evita capability slot.

**Decisione.** Co-chiude con **G5/D019** (BuffKind canon). Senza, Battery Loop non funziona pulito.

---

### G9 — `EventFilter::Not(Box<EventFilter>)` per kitsune_grace self-Ult gate

**Sintomo.** Renamon `kitsune_grace` deve filtrare "alleato che ha consumato Ult ∧ ≠ self". §5.6 dichiara `EventFilter::{AllyOfOwner, EnemyOfOwner, Self_, Any}` + custom predicate. Mancano combinatori boolean (Not/All/Any).

**Proposta.** Estendere a `EventFilter::{... esistenti ..., All(Vec<EventFilter>), Any(Vec<EventFilter>), Not(Box<EventFilter>), Custom(Arc<dyn Fn(&AbilityEvent) -> bool>)}`. La `Custom` resta escape hatch per casi non coperti dai combinator chiusi.

**Decisione.** Chiudere come **D021** parte di S04 (observer wiring). Pattern usato anche da `fur_cloak` (`EventFilter::SourceIsOwner`, derivabile da `All([Self_, ...])`).

---

### G10 — Predator state lifecycle: Enter/Exit via signal vs system

**Sintomo.** Dorumon `predator_loop` passive emette `Intent::BlueprintSignal { signal: "predator_enter" }` quando enemy HP <50%. Lo state `PredatorLoopState` è `Resource` di owner `"dorumon"`. Chi consuma il signal?
- (a) Observer Bevy interno al `DorumonPlugin`, registrato su `CombatKernelTransition::Blueprint { owner: "dorumon", ... }`. Downcast checked (D006).
- (b) Hook routing del kernel chiama `BlueprintRegistry::get("dorumon").handle_signal(&payload)` direttamente.

§4.3 RESEARCH preferisce (a) "layer double". Funziona. Conferma che `Box<dyn Any>` downcast è la sola arma; debug_assert su mismatch.

**Decisione.** Confermare in S03 con test mismatch panic. Nessun gap API; gap di documentazione (chiarire lifecycle nella roadmap S03 boundary).

---

### G11 — `ctx.adjacents()` e `ctx.adj_lowest_hp_pct()`: posizionale

**Sintomo.** Agumon `baby_burner` splash adj, Gabumon `gabumon_shot` echo adj lowest. `SkillCtx` deve esporre query posizionali su `SlotIndex`. Già esiste `SlotIndex(u8)` (memory: «SlotIndex(u8) inserito post-spawn»). Adj = slot ±1 alive same-team. OK.

**Decisione.** Nessun gap. Documentare API ctx: `ctx.adjacents(primary) -> Vec<UnitId>`, `ctx.adj_lowest_hp_pct(primary) -> Option<UnitId>`.

---

### G12 — `ApplyStatus` con `source: UnitId` per Twin Core consistency

**Sintomo.** Twin Core Ice condition usa `TargetHasStatus(Chilled)` — Chilled può essere applicato solo da Gabumon (per ora). Ma se futura skill non-Gabumon applicasse Chilled (es. nemico Bear-Digimon Ice), Twin Core di Agumon si triggerebbe erroneamente.

**Proposta.** Status canon traccia `applied_by: Option<UnitId>` o `applied_by_owner: Option<BlueprintId>`. `ModifierCondition::TargetHasStatusFrom(kind, owner)` per il future-proof. Pattern minore per v0, ma trascurarlo crea bug silente quando il roster cresce.

**Decisione.** **D022** opzionale, decidere se chiudere in M021 o deferire a M024+. Note: aggiungere il campo `applied_by` allo StatusInstance non rompe l'API (default None). Preferibile aggiungerlo ora che non dopo.

---

### G13 — `FollowUp` ability come ability separata (Dorumon dash_metal_chain)

**Sintomo.** Dorumon `on_kill(|ev, ctx| ctx.enqueue(Intent::EnqueueFollowUp { skill: DASH_METAL_CHAIN, ... }))` richiede una **seconda skill registrata** che faccia il chain damage. Pattern già in canon (`agumon_follow_up` "Spitfire"). OK.

**Decisione.** Documentare nel CONTEXT che FollowUp ability è una `AbilityCategory::FollowUp` distinta, legality kernel-skipped, non in input picker UI. Pattern noto.

---

### G14 — Plugin asimmetria: 3 dir + 3 flat → tutti dir uniformi

**Sintomo.** §1.3 RESEARCH: `gabumon.rs`, `renamon.rs`, `tentomon.rs` flat — non `Plugin`. La stesura assume tutti uniformi. La promozione `flat → dir` va fatta come parte della migration (S10/S12/S13 nella roadmap).

**Decisione.** Confermare in roadmap S10/S12/S13 che il deliverable include la promozione strutturale. Già implicito; documentare esplicito.

---

### G15 — Auto-consume `EnhancedNext` status: meta-flag su StatusDef

**Sintomo.** §5.8 RESEARCH: «meta-flag su `Status` (`consume_on_skill_cast: true`)». Lo riusiamo per `Blessed`? `Blessed` non è auto-consume (dura 2 turni). Quindi flag è per-status, non globale. `EnhancedNext` come status concreto non esiste ancora nel canon v0; lo introdurranno post-M021. Nessuna skill v0 lo usa attualmente — è un pattern futuro.

**Decisione.** Mantenere il flag `consume_on_skill_cast` come campo opzionale di `StatusDef` (default false). Nessuna skill v0 lo attiva. Pattern noto, documentato per future-proof.

---

### G16 — `ctx.is_owner(unit, blueprint_id)` e `ctx.iter_alive_owners()`

**Sintomo.** Modifier conditional Twin Core / Holy Aegis necessitano "chi è il portatore" e "chi è vivo per owner". Query semplici, ma vanno esposti esplicitamente sul Snapshot.

**Decisione.** Nessun gap API; aggiungere alla lista canon delle query Snapshot in §5.10. `Snapshot::is_owner(unit, BlueprintId)`, `Snapshot::iter_alive_owners() -> Iter<BlueprintId>`, `Snapshot::iter_alive_units_with_owner(BlueprintId) -> Iter<UnitId>`.

---

## 9. Sintesi gap

| ID | Severità | Slice di chiusura | Decisione da creare |
|---|---|---|---|
| G1 — StatusApplyMode | **alta** | S05 (Intent canon) | D016 |
| G2 — Heated decay tick-canon | bassa (doc) | — | nessuna |
| G3 — ModifierCondition canon list | **alta** | S07 | D017 |
| G4 — Blessed intrinsic effects | **alta** | S05 | D018 |
| G5 — BuffKind separato | **alta** | S05 | D019 |
| G6 — on_final_hop Bounce (hook fn, vedi D023) | media | S07 | D020 |
| G7 — Random→deterministic selector | bassa (doc) | — | nessuna |
| G8 — BuffStackMode::ProcOnce | media | S05 | (co-chiude G5/D019) |
| G9 — EventFilter combinatori | media | S04 | D021 |
| G10 — Predator lifecycle | bassa (doc) | S03 | nessuna |
| G11 — Adj query API | bassa | S06 | nessuna |
| G12 — Status applied_by | bassa (future-proof) | S05 opzionale | D022 candidata |
| G13 — FollowUp ability category | bassa (doc) | — | nessuna |
| G14 — Plugin dir uniformi | bassa (struct) | S10/S12/S13 | nessuna |
| G15 — consume_on_skill_cast | bassa (future-proof) | — | nessuna |
| G16 — Snapshot owner queries | bassa (doc) | S06 | nessuna |

**Bilancio.** 5 gap ad alta severità + 1 candidata D022 future-proof. Tutti chiudibili dentro S04–S07 senza alterare la struttura della roadmap. Nessuno richiede di smontare slice già pianificate. **Inoltre** la revisione hook ha già chiuso D023 (hook fn vs enum) e D024 (`SkillCtx::Mode { Live, DryRun }`) — questi non sono gap del roster ma decisioni architetturali pre-S01 abilitate dalla stesura.

**Punti positivi confermati dalla stesura.**

1. **Niente kernel pollution.** Tutte le 24 ability scritte vivono sotto `blueprints/<x>/` (più `blueprints/twin_core/`). Zero menzione di Twin Core/Predator/Holy/Battery/Kitsune nel kernel target.
2. **`Intent` canon di §5.7 copre 22/24 ability senza estensioni.** Le 2 estensioni (`ApplyBuff`, `ApplyStatus { mode }`) sono incrementali, non strutturali.
3. **AbilityBuilder ergonomia** regge: ogni skill sta in 10-30 linee, leggibili. Hook fn aggiungono ~3-5 linee per chain vs enum dichiarativo, in cambio di branching libero in-closure (Dorumon Predator-only chain, Agumon detonate condizionale).
4. **Snapshot frozen + `ctx.enqueue` produce ordering deterministico** anche per chain (Gabumon echo, Agumon detonate, Dorumon chain). Le closure catturano per `move` valori già letti dallo snapshot pre-skill (Heated stacks, hp_pct), garantendo invarianza nel resto del cast.
5. **Modifier pipeline è il vero load-bearing**: 3/6 passive sono pure Modifier (`twin_core_fire`, `twin_core_ice`, `holy_aegis`). Senza ModifierCondition ricco (G3) il design crolla.
6. **Hook unification post-D023**: passive listener, per-deal chain hook, on-final-hop sono tutti la stessa forma `(event, ctx) → ctx.enqueue`. Zero match enum nel kernel, zero variant da estendere quando arrivano nuovi side-effect. Preview UI/AI via dry-run (D024) gira gli **stessi** hook su buffer locale, eliminando per costruzione il problema "preview diverge da execute" sui rami condizionali.

---

## 10. Prossimi passi

1. **Fatto** — D023 (hook fn) + D024 (`SkillCtx::Mode`) persistite in `.gsd/DECISIONS.md`. Stesura aggiornata coerentemente.
2. Propagare i **5 gap ad alta severità** (G1/G3/G4/G5 + parziale G8/G9) come Decisioni `D016`–`D021` in `.gsd/DECISIONS.md`, e annotare la lista in `M021-RESEARCH.md` §18. D020 va scritta con il vincolo "hook fn coerente con D023".
3. Aggiornare la roadmap S04–S07 con i requisiti di chiusura gap (extension canon per Intent/Modifier/Buff) **e** per il pattern hook fn (`ChainHooks`, `HitEvent`/`KillEvent`/`MissEvent`/`FinalHopEvent`, `SkillCtxMode`).
4. Validare il fit con utente: l'AbilityBuilder ergonomia post-hook è accettabile o si vuole iterare?
5. (Opzionale) Convertire questa stesura in test fixture sintetiche per S07 (proof AbilityBuilder copertura) — già richiesto dalla roadmap.
